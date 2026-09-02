//! Invar configuration and identity (keys.toml) helpers.

use std::path::{Path, PathBuf};

use dwow_sdk::crypto::keypair::Network;

/// The subset of wallet configuration Invar needs to open a wallet.
///
/// Mirrors `dwow_wallet`'s `WalletConfig` (bin/dww/src/config.rs) but with Invar-owned
/// defaults and no reliance on environment variables (`WALLET_NAME`, `DWOW_WALLET_PASS`).
#[derive(Debug, Clone)]
pub struct InvarConfig {
    /// Network name ("darkwow-testnet", "darkwow-devnet", "mainnet", …).
    pub network: String,
    /// Path to the owner's `keys.toml`.
    pub keys_toml: String,
    /// `keys.toml` section selecting this wallet's identity.
    pub section: String,
    /// Path to the SQLite wallet database.
    pub wallet_path: String,
    /// SQLCipher password for the wallet database.
    pub wallet_pass: String,
    /// Production security checks (password strength, encryption verification).
    pub production: bool,
    /// P2P peer URLs the wallet pulls the chain from (empty = no P2P sync).
    pub peers: Vec<String>,
}

impl InvarConfig {
    pub fn network_enum(&self) -> Network {
        match self.network.as_str() {
            "mainnet" | "localnet" => Network::Mainnet,
            _ => Network::Testnet,
        }
    }

    pub fn default_keys_toml() -> String {
        home_path(".config/invar/keys.toml")
    }

    pub fn default_wallet_path(network: &str) -> String {
        home_path(&format!(".local/share/invar/{network}/wallet.db"))
    }

    /// Default P2P peers for known networks (mirrors dww_config.toml).
    pub fn default_peers(network: &str) -> Vec<String> {
        match network {
            "darkwow-testnet" => vec!["tcp+tls://127.0.0.1:31342".to_string()],
            "testnet" => vec![
                "tcp+tls://lilith0.dark.fi:18340".to_string(),
                "tcp+tls://lilith1.dark.fi:18340".to_string(),
            ],
            _ => vec![],
        }
    }
}

fn home_path(rel: &str) -> String {
    std::env::var("HOME")
        .map(|h| PathBuf::from(h).join(rel))
        .map(|p| p.to_string_lossy().to_string())
        .unwrap_or_else(|_| rel.to_string())
}

/// Generate a fresh random key and write it as `[section]` in `keys.toml`.
/// Returns the secret hex (owner's responsibility to back up).
pub fn generate_new_key(path: &Path, section: &str) -> Result<String, String> {
    let mut mgr = dwow_accounts::AccountManager::empty(Network::Testnet);
    mgr.generate(); // owner-initiated randomness
    let hex = mgr.export_hex(0)?;
    write_keys_toml(path, section, &hex)?;
    Ok(hex)
}

/// Restore from a BIP39 mnemonic (12/24 words), writing the derived key to `keys.toml`.
pub fn restore_from_mnemonic(
    path: &Path,
    section: &str,
    phrase: &str,
    network: Network,
) -> Result<String, String> {
    let mgr = dwow_accounts::AccountManager::from_seed_phrase(phrase.trim(), "", network)?;
    let hex = mgr.export_hex(0)?;
    write_keys_toml(path, section, &hex)?;
    Ok(hex)
}

/// Restore from a raw 64-hex secret, writing it to `keys.toml`.
pub fn restore_from_hex(path: &Path, section: &str, hex: &str) -> Result<String, String> {
    let hex = hex.trim();
    if hex.len() != 64 || !hex.chars().all(|c| c.is_ascii_hexdigit()) {
        return Err("secret must be exactly 64 hex characters".into());
    }
    write_keys_toml(path, section, hex)?;
    Ok(hex.to_string())
}

/// Write (or merge) a `[section] wallet_secret` entry into `keys.toml`, preserving other
/// sections so multiple wallets can share one file.
fn write_keys_toml(path: &Path, section: &str, hex: &str) -> Result<(), String> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).map_err(|e| e.to_string())?;
    }

    let mut root: toml::Value = if path.exists() {
        let text = std::fs::read_to_string(path).map_err(|e| format!("read keys.toml: {e}"))?;
        toml::from_str(&text).unwrap_or_else(|_| toml::Value::Table(Default::default()))
    } else {
        toml::Value::Table(Default::default())
    };

    let table = root
        .as_table_mut()
        .ok_or("keys.toml root must be a TOML table")?;

    let mut section_table = toml::map::Map::new();
    section_table.insert("wallet_secret".to_string(), toml::Value::String(hex.to_string()));
    table.insert(section.to_string(), toml::Value::Table(section_table));

    let text = toml::to_string_pretty(&root).map_err(|e| e.to_string())?;
    std::fs::write(path, text).map_err(|e| format!("write keys.toml: {e}"))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn tmp(name: &str) -> PathBuf {
        std::env::temp_dir().join("invar_test").join(name)
    }

    #[test]
    fn generate_and_read_back() {
        let path = tmp("gen_keys.toml");
        let _ = std::fs::remove_file(&path);
        let hex = generate_new_key(&path, "wallet-1").unwrap();
        assert_eq!(hex.len(), 64);

        let mgr =
            dwow_accounts::AccountManager::open(&path, Network::Testnet, "wallet-1").unwrap();
        assert_eq!(mgr.export_hex(0).unwrap(), hex);
        let _ = std::fs::remove_file(&path);
    }

    #[test]
    fn restore_from_hex_validates() {
        let path = tmp("hex_keys.toml");
        let _ = std::fs::remove_file(&path);
        assert!(restore_from_hex(&path, "w", "not-hex").is_err());
        let hex = "00".repeat(32);
        restore_from_hex(&path, "wallet-1", &hex).unwrap();
        let mgr =
            dwow_accounts::AccountManager::open(&path, Network::Testnet, "wallet-1").unwrap();
        assert_eq!(mgr.export_hex(0).unwrap(), hex);
        let _ = std::fs::remove_file(&path);
    }

    #[test]
    fn merge_preserves_other_sections() {
        let path = tmp("merge_keys.toml");
        std::fs::create_dir_all(path.parent().unwrap()).ok();
        std::fs::write(&path, "[wallet-1]\nwallet_secret = \"00\"\n").unwrap();
        restore_from_hex(&path, "wallet-2", &"11".repeat(32)).unwrap();
        let text = std::fs::read_to_string(&path).unwrap();
        assert!(text.contains("[wallet-1]"));
        assert!(text.contains("[wallet-2]"));
        let _ = std::fs::remove_file(&path);
    }
}
