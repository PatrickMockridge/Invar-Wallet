//! Shared UI state snapshot that the egui thread reads each frame.

/// Snapshot of wallet state that the egui thread reads each frame.
///
/// The background wallet service owns the real `dwow_wallet::Dww` and writes a fresh
/// snapshot here after each operation; the UI never touches `Dww` directly.
#[derive(Debug, Default)]
pub struct ViewModel {
    /// Console output log (REPL + verb results).
    pub console_log: Vec<String>,
    /// Local chain height.
    pub sync_height: u64,
    /// Number of connected peers.
    pub peer_count: usize,
    /// Whether the wallet considers itself synced to the network tip.
    pub synced: bool,
    /// Number of snapshots published by the background sync loop (progress marker).
    pub snapshot_count: u64,
}

impl ViewModel {
    pub fn new() -> Self {
        Self::default()
    }
}

/// Display-friendly view of a held capability (built from `dwow_wallet::walletdb::CapRecord`).
///
/// All crypto values are pre-formatted to strings so the UI never touches raw dwow types.
#[derive(Debug, Clone)]
pub struct CapView {
    pub cap_id: String,
    pub value: u64,
    pub asset_id: String,
    pub contract_name: String,
    pub trust: String,
    pub name: String,
    pub resource: String,
    pub action: String,
    pub primitives: String,
    pub barbs: String,
    pub status: String,
    pub leaf_position: u64,
    pub created_at_height: u64,
}

/// Display-friendly view of a contract (the nine genesis contracts in milestone 4).
#[derive(Debug, Clone)]
pub struct ContractView {
    pub name: String,
    pub contract_id: String,
    pub trust: String,
}

/// Display-friendly view of a contract's on-chain manifest (milestone 6).
///
/// `functions`/`capabilities`/`actions`/`parameters` are pre-formatted strings so the UI
/// never touches raw manifest types.
#[derive(Debug, Clone)]
pub struct ContractManifestView {
    pub name: String,
    pub category: String,
    pub description: String,
    pub version: String,
    pub trust: String,
    pub functions: Vec<String>,
    pub capabilities: Vec<String>,
    pub actions: Vec<String>,
    pub parameters: Vec<String>,
}
