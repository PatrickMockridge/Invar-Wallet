//! WalletService — owns the `dwow_wallet::Dww` and runs the background wallet lifecycle.
//!
//! The service mirrors the wallet CLI's async runtime (`bin/dww/src/main.rs:119-133` +
//! `dispatch_async` Sync::Init): a `smol::Executor` on background threads runs
//! `init_p2p`, `sync_task::run_wallet_sync`, an auto-scan loop, and a snapshot loop that
//! publishes sync state into the shared [`ViewModel`] the egui thread reads each frame.

use std::path::Path;
use std::sync::Arc;

use parking_lot::RwLock;

use dwow_wallet::p2p_wallet::{P2pWalletConfig, SeedAddr};
use dwow_wallet::DwwPtr;

use crate::config::InvarConfig;
use crate::contracts;
use crate::viewmodel::{CapView, ContractView, ViewModel};

pub struct WalletService {
    dww: Option<DwwPtr>,
    vm: Arc<RwLock<ViewModel>>,
    /// Background executor threads (join on shutdown).
    threads: Vec<std::thread::JoinHandle<()>>,
    /// Dropping this sender closes the shutdown channel and stops the executor threads.
    signal: Option<smol::channel::Sender<()>>,
}

impl WalletService {
    /// Open (and initialize) a wallet from config. Blocking — call once at startup/onboarding.
    pub fn open(config: &InvarConfig, vm: Arc<RwLock<ViewModel>>) -> Result<Self, String> {
        let p2p_settings = Some(P2pWalletConfig {
            peers: config
                .peers
                .iter()
                .map(|url| SeedAddr { url: url.clone() })
                .collect(),
            magic_bytes: [68, 82, 75, 87], // "DRKW" — DarkWow testnet/devnet magic
            port: 31340,
            max_peers: 8,
            connect_timeout_secs: 10,
            request_timeout_secs: 30,
            localnet: matches!(
                config.network.as_str(),
                "darkwow-devnet" | "darkwow-testnet" | "localnet"
            ),
            inbound: vec![],
            app_name: None,
            datastore: None,
        });

        let dww = dwow_wallet::Dww::new(
            config.network_enum(),
            Some(Path::new(&config.keys_toml)),
            &config.section,
            config.wallet_path.clone(),
            config.wallet_pass.clone(),
            config.production,
            p2p_settings,
        )
        .map_err(|e| e.to_string())?;

        dww.initialize_wallet().map_err(|e| e.to_string())?;

        Ok(Self {
            dww: Some(dww.into_ptr()),
            vm,
            threads: Vec::new(),
            signal: None,
        })
    }

    /// Start the background sync + scan + snapshot loops. Idempotent.
    pub fn start_sync(&mut self) -> Result<(), String> {
        if self.signal.is_some() {
            return Ok(()); // already running
        }
        let dww = self.dww.clone().ok_or("wallet not open")?;
        let vm = self.vm.clone();

        // 1. Spawn executor threads (the async runtime the P2P sessions run on).
        let ex: Arc<smol::Executor<'static>> = Arc::new(smol::Executor::new());
        let (signal, shutdown) = smol::channel::unbounded::<()>();
        let mut threads = Vec::new();
        for _ in 0..2 {
            let ex = ex.clone();
            let shutdown = shutdown.clone();
            threads.push(std::thread::spawn(move || {
                let _ = smol::future::block_on(ex.run(shutdown.recv()));
            }));
        }
        self.threads = threads;
        self.signal = Some(signal);

        // 2. Snapshot loop — publishes sync state to the ViewModel every 2s.
        {
            let ex = ex.clone();
            let dww = dww.clone();
            let vm = vm.clone();
            ex.spawn(async move {
                loop {
                    smol::Timer::after(std::time::Duration::from_secs(2)).await;
                    let (height, peers, synced) = {
                        let r = dww.read().await;
                        let height = r.chain_height().map(|h| h.get()).unwrap_or(0);
                        let peers = r
                            .p2p
                            .as_ref()
                            .map(|p| p.hosts().peers().len())
                            .unwrap_or(0);
                        let synced = r.is_synced();
                        (height, peers, synced)
                    };
                    let mut w = vm.write();
                    w.sync_height = height;
                    w.peer_count = peers;
                    w.synced = synced;
                    w.snapshot_count += 1;
                }
            })
            .detach();
        }

        // 3. init_p2p, then spawn the sync loop and the auto-scan loop.
        {
            let ex = ex.clone();
            let dww = dww.clone();
            let vm = vm.clone();
            let ex2 = ex.clone();
            ex.spawn(async move {
                {
                    let mut w = dww.write().await;
                    if let Err(e) = w.init_p2p(ex2.clone()).await {
                        vm.write().console_log.push(format!("P2P init failed: {e}"));
                        return;
                    }
                    vm.write().console_log.push("P2P initialized".to_string());
                }

                // Chain sync — pulls GetTip/GetBlocks from configured peers.
                {
                    let tip = dww.read().await.highest_peer_tip.clone();
                    let dww = dww.clone();
                    ex2.spawn(async move {
                        if let Err(e) = dwow_wallet::sync_task::run_wallet_sync(dww, tip).await {
                            tracing::error!("sync task: {e}");
                        }
                    })
                    .detach();
                }

                // Auto-scan — AEAD-decrypts newly synced blocks into held capabilities.
                {
                    let dww = dww.clone();
                    ex2.spawn(async move {
                        loop {
                            smol::Timer::after(std::time::Duration::from_secs(5)).await;
                            let r = dww.read().await;
                            if let Err(e) = r.scan_blocks(&mut vec![], None, &false).await {
                                tracing::warn!("scan: {e}");
                            }
                        }
                    })
                    .detach();
                }
            })
            .detach();
        }

        Ok(())
    }

    /// Stop the background threads (drops the shutdown signal and joins).
    pub fn shutdown(&mut self) {
        self.signal = None;
        for t in self.threads.drain(..) {
            let _ = t.join();
        }
    }

    pub fn is_open(&self) -> bool {
        self.dww.is_some()
    }

    /// True once `init_p2p` has created the P2P instance.
    pub fn p2p_ready(&self) -> bool {
        self.dww
            .as_ref()
            .map(|dww| smol::block_on(dww.read()).p2p.is_some())
            .unwrap_or(false)
    }

    pub fn is_synced(&self) -> bool {
        self.dww
            .as_ref()
            .map(|dww| smol::block_on(dww.read()).is_synced())
            .unwrap_or(false)
    }

    pub fn peer_count(&self) -> usize {
        self.dww
            .as_ref()
            .map(|dww| {
                let r = smol::block_on(dww.read());
                r.p2p.as_ref().map(|p| p.hosts().peers().len()).unwrap_or(0)
            })
            .unwrap_or(0)
    }

    pub fn default_address(&self) -> Result<String, String> {
        let dww = self.dww.as_ref().ok_or("wallet not open")?;
        let addr = smol::block_on(dww.read())
            .default_address()
            .map_err(|e| e.to_string())?;
        Ok(addr.to_string())
    }

    pub fn capability_balance(&self) -> Result<std::collections::HashMap<String, u64>, String> {
        let dww = self.dww.as_ref().ok_or("wallet not open")?;
        smol::block_on(dww.read())
            .capability_balance()
            .map_err(|e| e.to_string())
    }

    pub fn held_capability_count(&self) -> Result<usize, String> {
        let dww = self.dww.as_ref().ok_or("wallet not open")?;
        Ok(smol::block_on(dww.read())
            .get_held_capabilities(Some(false))
            .map_err(|e| e.to_string())?
            .len())
    }

    pub fn chain_height(&self) -> Result<u64, String> {
        let dww = self.dww.as_ref().ok_or("wallet not open")?;
        Ok(smol::block_on(dww.read())
            .chain_height()
            .map_err(|e| e.to_string())?
            .get())
    }

    /// Held capabilities as display-friendly views (typed: name/discriminant/primitives/
    /// barbs/status/trust). Empty until sync + scan discover them.
    pub fn held_capability_views(&self) -> Result<Vec<CapView>, String> {
        let dww = self.dww.as_ref().ok_or("wallet not open")?;
        let caps = smol::block_on(dww.read())
            .get_held_capabilities(Some(false))
            .map_err(|e| e.to_string())?;
        Ok(caps.into_iter().map(cap_view).collect())
    }

    /// All derived addresses (from the declared identity) as display strings.
    pub fn addresses(&self) -> Result<Vec<String>, String> {
        let dww = self.dww.as_ref().ok_or("wallet not open")?;
        let r = smol::block_on(dww.read());
        let network = r.network;
        let addrs = r.addresses().map_err(|e| e.to_string())?;
        Ok(addrs
            .into_iter()
            .map(|(_, public, _, _)| {
                let std = dwow_sdk::crypto::keypair::StandardAddress::from_public(network, public);
                let addr: dwow_sdk::crypto::keypair::Address = std.into();
                addr.to_string()
            })
            .collect())
    }

    /// The nine genesis contracts, configured as `[GENESIS]` (standard).
    pub fn contract_views(&self) -> Vec<ContractView> {
        contracts::genesis_contracts()
            .into_iter()
            .map(|g| ContractView {
                name: g.name.to_string(),
                contract_id: contracts::b58(g.id.to_bytes()),
                trust: "GENESIS".to_string(),
            })
            .collect()
    }
}

/// Build a display-friendly view from a raw capability record.
fn cap_view(cap: dwow_wallet::walletdb::CapRecord) -> CapView {
    CapView {
        cap_id: cap.cap_id,
        value: cap.value,
        asset_id: contracts::b58(cap.asset_id.to_bytes()),
        contract_name: contracts::contract_name(&cap.contract_id).to_string(),
        trust: contracts::trust_tier(&cap.contract_id).to_string(),
        name: cap
            .capability_name
            .unwrap_or_else(|| "(native)".to_string()),
        resource: cap.resource.unwrap_or_default(),
        action: cap.action.unwrap_or_default(),
        primitives: format!("{:?}", cap.primitives),
        barbs: format!("{:?}", cap.barbs),
        status: cap
            .status
            .map(|s| s.as_str().to_string())
            .unwrap_or_else(|| "unspent".to_string()),
        leaf_position: cap.leaf_position,
        created_at_height: cap.created_at_height.get(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config;

    fn test_config(label: &str, peers: Vec<String>) -> InvarConfig {
        let dir = std::env::temp_dir()
            .join("invar_wallet_service_test")
            .join(label);
        std::fs::create_dir_all(&dir).ok();
        let keys = dir.join("keys.toml");
        let db = dir.join("wallet.db");
        let _ = std::fs::remove_file(&keys);
        let _ = std::fs::remove_file(&db);

        config::generate_new_key(&keys, "wallet-1").unwrap();

        InvarConfig {
            network: "darkwow-devnet".to_string(),
            keys_toml: keys.to_string_lossy().to_string(),
            section: "wallet-1".to_string(),
            wallet_path: db.to_string_lossy().to_string(),
            wallet_pass: "test-password".to_string(),
            production: false,
            peers,
        }
    }

    fn vm() -> Arc<RwLock<ViewModel>> {
        Arc::new(RwLock::new(ViewModel::new()))
    }

    #[test]
    fn open_wallet_creates_db_and_derives_address() {
        let cfg = test_config("open", vec![]);
        let w = WalletService::open(&cfg, vm()).unwrap();
        assert!(!w.default_address().unwrap().is_empty());
        assert_eq!(w.held_capability_count().unwrap(), 0);
        assert_eq!(w.chain_height().unwrap(), 0);
    }

    #[test]
    fn start_sync_runs_and_publishes_snapshots() {
        // Install the rustls provider (normally done in main()) before any TLS work.
        let _ = rustls::crypto::ring::default_provider().install_default();

        let cfg = test_config("sync", vec![]); // no peers — sync loops without dialling anything
        let vm = vm();
        let mut w = WalletService::open(&cfg, vm.clone()).unwrap();

        w.start_sync().unwrap();

        // Give init_p2p + a couple of snapshot cycles time to run.
        std::thread::sleep(std::time::Duration::from_secs(4));

        // The snapshot loop ran and published state.
        assert!(vm.read().snapshot_count > 0, "snapshot loop did not run");

        // init_p2p succeeded even with zero configured peers (P2p instance created).
        assert!(w.p2p_ready(), "P2P was not initialized");

        // No chain, no peers, not synced.
        let snap = vm.read();
        assert_eq!(snap.sync_height, 0);
        assert_eq!(snap.peer_count, 0);
        assert!(!snap.synced);

        w.shutdown();
    }

    #[test]
    fn verbs_query_the_wallet() {
        use crate::commands::contracts::ContractsPlugin;
        use crate::commands::wallet::WalletPlugin;
        use crate::commands::{CommandContext, CommandRegistry};

        let cfg = test_config("verbs", vec![]);
        let w = WalletService::open(&cfg, vm()).unwrap();

        let mut reg = CommandRegistry::new();
        reg.register_plugin(&crate::commands::CorePlugin);
        reg.register_plugin(&WalletPlugin);
        reg.register_plugin(&ContractsPlugin);

        let mut ctx = CommandContext::new();
        ctx.wallet = Some(std::sync::Arc::new(w));

        reg.execute_line("/balance", &mut ctx);
        reg.execute_line("/address", &mut ctx);
        reg.execute_line("/contracts", &mut ctx);
        reg.execute_line("/sync", &mut ctx);

        let out = ctx.out.join("\n");
        assert!(out.contains("no native balance"), "balance verb: {out}");
        assert!(!out.contains("error:"), "unexpected error: {out}");
        assert!(out.contains("native_token [GENESIS]"), "contracts verb: {out}");
        assert!(out.contains("chain height: 0"), "sync verb: {out}");
        assert!(out.contains("synced: false"), "sync verb: {out}");
    }
}
