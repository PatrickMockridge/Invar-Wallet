//! Invar — a fork-able GUI wallet for DarkWow, inspired by Electrum.

mod app;
mod commands;
mod config;
mod contracts;
mod macros;
mod plugin;
mod ui;
mod viewmodel;
mod wallet_service;

use std::sync::Arc;

use parking_lot::RwLock;

use app::InvarApp;
use commands::{CommandRegistry, CorePlugin};
use viewmodel::ViewModel;

fn main() -> eframe::Result<()> {
    // Install the rustls ring crypto provider before any TLS (P2P sync dials TLS peers).
    // Mirrors dwow_wallet's CLI entry (bin/dww/src/main.rs).
    let _ = rustls::crypto::ring::default_provider().install_default();

    // Surface net-layer warnings (connection failures) to the console.
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::WARN)
        .init();

    // Build the command registry from built-in plugins (and later, user plugins).
    let mut registry = CommandRegistry::new();
    registry.register_plugin(&CorePlugin);
    registry.register_plugin(&commands::wallet::WalletPlugin);
    registry.register_plugin(&commands::contracts::ContractsPlugin);

    // Load user macros from ~/.config/invar/macros.toml (named verb sequences).
    if let Ok(home) = std::env::var("HOME") {
        let path = std::path::Path::new(&home).join(".config/invar/macros.toml");
        if let Ok(text) = std::fs::read_to_string(&path) {
            match macros::load_macros(&text) {
                Ok(m) => registry.set_macros(m),
                Err(e) => tracing::warn!("failed to load macros from {path:?}: {e}"),
            }
        }
    }

    let vm = Arc::new(RwLock::new(ViewModel::new()));

    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([1000.0, 700.0])
            .with_title("Invar"),
        ..Default::default()
    };

    eframe::run_native(
        "Invar",
        options,
        Box::new(move |cc| {
            crate::ui::theme::apply(&cc.egui_ctx);
            Ok(Box::new(InvarApp::new(registry, vm)))
        }),
    )
}
