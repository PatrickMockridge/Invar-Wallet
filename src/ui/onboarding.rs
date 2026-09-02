//! First-run onboarding wizard: select network, create/restore/open a keys.toml identity,
//! set a wallet password, and produce an [`InvarConfig`].

use std::path::Path;

use dwow_sdk::crypto::keypair::Network;

use crate::config::{self, InvarConfig};

const NETWORKS: &[&str] = &[
    "darkwow-testnet",
    "darkwow-devnet",
    "testnet",
    "localnet",
    "mainnet",
];

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OnboardAction {
    CreateNew,
    RestoreMnemonic,
    RestoreHex,
    OpenExisting,
}

#[derive(Debug, Clone)]
pub struct OnboardingState {
    pub network: String,
    pub action: OnboardAction,
    pub section: String,
    pub password: String,
    pub password_confirm: String,
    pub mnemonic: String,
    pub hex_secret: String,
    pub keys_toml_path: String,
    pub wallet_path: String,
    pub error: Option<String>,
}

impl Default for OnboardingState {
    fn default() -> Self {
        let network = "darkwow-testnet".to_string();
        Self {
            wallet_path: InvarConfig::default_wallet_path(&network),
            keys_toml_path: InvarConfig::default_keys_toml(),
            network,
            action: OnboardAction::CreateNew,
            section: "wallet-1".to_string(),
            password: String::new(),
            password_confirm: String::new(),
            mnemonic: String::new(),
            hex_secret: String::new(),
            error: None,
        }
    }
}

/// Render the onboarding wizard. Returns `Some(config)` once the user successfully
/// creates/restores/opens a wallet (the keys.toml write happens here as a side effect).
pub fn show(ui: &mut egui::Ui, state: &mut OnboardingState) -> Option<InvarConfig> {
    ui.heading("Welcome to Invar");
    ui.label("A fork-able GUI wallet for DarkWow.");
    ui.separator();

    egui::Grid::new("onboarding_grid")
        .num_columns(2)
        .spacing([12.0, 8.0])
        .show(ui, |ui| {
            ui.label("Network");
            egui::ComboBox::from_id_salt("network")
                .selected_text(state.network.as_str())
                .show_ui(ui, |ui| {
                    for n in NETWORKS {
                        ui.selectable_value(&mut state.network, n.to_string(), *n);
                    }
                });
            ui.end_row();

            ui.label("Wallet name");
            ui.text_edit_singleline(&mut state.section);
            ui.end_row();

            ui.label("Password");
            ui.add(egui::TextEdit::singleline(&mut state.password).password(true));
            ui.end_row();

            ui.label("Confirm");
            ui.add(egui::TextEdit::singleline(&mut state.password_confirm).password(true));
            ui.end_row();

            ui.label("keys.toml");
            ui.text_edit_singleline(&mut state.keys_toml_path);
            ui.end_row();

            ui.label("Wallet DB");
            ui.text_edit_singleline(&mut state.wallet_path);
            ui.end_row();
        });

    ui.separator();

    ui.radio_value(&mut state.action, OnboardAction::CreateNew, "Create a new wallet");
    ui.radio_value(
        &mut state.action,
        OnboardAction::RestoreMnemonic,
        "Restore from BIP39 mnemonic",
    );
    ui.radio_value(&mut state.action, OnboardAction::RestoreHex, "Restore from secret hex");
    ui.radio_value(&mut state.action, OnboardAction::OpenExisting, "Open existing wallet");

    ui.separator();

    match state.action {
        OnboardAction::RestoreMnemonic => {
            ui.label("BIP39 mnemonic (12 or 24 words):");
            ui.add(
                egui::TextEdit::multiline(&mut state.mnemonic)
                    .desired_rows(3)
                    .desired_width(f32::INFINITY),
            );
        }
        OnboardAction::RestoreHex => {
            ui.label("Secret hex (64 characters):");
            ui.add(egui::TextEdit::singleline(&mut state.hex_secret).desired_width(f32::INFINITY));
        }
        _ => {}
    }

    if let Some(err) = &state.error {
        ui.add_space(6.0);
        ui.colored_label(egui::Color32::RED, err);
    }

    ui.add_space(8.0);
    let label = match state.action {
        OnboardAction::OpenExisting => "Open wallet",
        _ => "Create wallet",
    };

    if ui.button(label).clicked() {
        match try_build(state) {
            Ok(config) => return Some(config),
            Err(e) => state.error = Some(e),
        }
    }

    None
}

fn network_enum(name: &str) -> Network {
    match name {
        "mainnet" | "localnet" => Network::Mainnet,
        _ => Network::Testnet,
    }
}

fn try_build(state: &OnboardingState) -> Result<InvarConfig, String> {
    if state.section.trim().is_empty() {
        return Err("enter a wallet name (keys.toml section)".into());
    }
    if state.password.is_empty() {
        return Err("enter a wallet password".into());
    }
    if state.password != state.password_confirm {
        return Err("passwords do not match".into());
    }

    let keys_toml = if state.keys_toml_path.trim().is_empty() {
        InvarConfig::default_keys_toml()
    } else {
        state.keys_toml_path.trim().to_string()
    };
    let keys_path = Path::new(&keys_toml);
    let network = network_enum(&state.network);
    let section = state.section.trim().to_string();

    match state.action {
        OnboardAction::CreateNew => {
            config::generate_new_key(keys_path, &section)?;
        }
        OnboardAction::RestoreMnemonic => {
            if state.mnemonic.trim().is_empty() {
                return Err("enter a BIP39 mnemonic".into());
            }
            config::restore_from_mnemonic(keys_path, &section, &state.mnemonic, network)?;
        }
        OnboardAction::RestoreHex => {
            config::restore_from_hex(keys_path, &section, &state.hex_secret)?;
        }
        OnboardAction::OpenExisting => {
            if !keys_path.exists() {
                return Err(format!("keys.toml not found at {}", keys_path.display()));
            }
        }
    }

    Ok(InvarConfig {
        network: state.network.clone(),
        keys_toml,
        section,
        wallet_path: if state.wallet_path.trim().is_empty() {
            InvarConfig::default_wallet_path(&state.network)
        } else {
            state.wallet_path.trim().to_string()
        },
        wallet_pass: state.password.clone(),
        production: false,
        peers: InvarConfig::default_peers(&state.network),
    })
}
