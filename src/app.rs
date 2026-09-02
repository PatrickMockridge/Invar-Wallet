//! InvarApp — the top-level eframe application.

use std::sync::Arc;

use parking_lot::RwLock;

use crate::commands::{CommandContext, CommandRegistry};
use crate::config::InvarConfig;
use crate::ui::onboarding::{self, OnboardingState};
use crate::viewmodel::{CapView, ContractManifestView, ContractView, ViewModel};
use crate::wallet_service::WalletService;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Screen {
    Onboarding,
    Main,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum NavScreen {
    Overview,
    Capabilities,
    Addresses,
    Contracts,
    Console,
}

pub struct InvarApp {
    registry: CommandRegistry,
    vm: Arc<RwLock<ViewModel>>,
    command_input: String,
    screen: Screen,
    nav: NavScreen,
    onboarding: OnboardingState,
    wallet: Option<Arc<WalletService>>,
    // Read-only data snapshots refreshed from the wallet.
    summary: Vec<String>,
    default_address: Option<String>,
    caps: Vec<CapView>,
    addresses: Vec<String>,
    contracts: Vec<ContractView>,
    contract_manifests: Vec<ContractManifestView>,
}

impl InvarApp {
    pub fn new(registry: CommandRegistry, vm: Arc<RwLock<ViewModel>>) -> Self {
        Self {
            registry,
            vm,
            command_input: String::new(),
            screen: Screen::Onboarding,
            nav: NavScreen::Overview,
            onboarding: OnboardingState::default(),
            wallet: None,
            summary: Vec::new(),
            default_address: None,
            caps: Vec::new(),
            addresses: Vec::new(),
            contracts: Vec::new(),
            contract_manifests: Vec::new(),
        }
    }

    fn run_command(&mut self) {
        let input = std::mem::take(&mut self.command_input);
        if input.trim().is_empty() {
            return;
        }

        let mut cctx = CommandContext::new();
        cctx.wallet = self.wallet.clone();
        self.registry.execute_line(&input, &mut cctx);

        let mut vm = self.vm.write();
        vm.console_log.push(format!("> {input}"));
        vm.console_log.extend(cctx.out);
    }

    fn open_wallet(&mut self, config: InvarConfig) {
        match WalletService::open(&config, self.vm.clone()) {
            Ok(mut w) => {
                if let Err(e) = w.start_sync() {
                    self.onboarding.error = Some(format!("failed to start sync: {e}"));
                    return;
                }
                self.wallet = Some(Arc::new(w));
                self.screen = Screen::Main;
                self.refresh();
            }
            Err(e) => {
                self.onboarding.error = Some(format!("failed to open wallet: {e}"));
            }
        }
    }

    /// Refresh all read-only snapshots from the wallet.
    fn refresh(&mut self) {
        let mut summary = Vec::new();
        let mut default_address = None;
        let mut caps = Vec::new();
        let mut addresses = Vec::new();
        let mut contracts = Vec::new();
        let mut contract_manifests = Vec::new();

        if let Some(w) = &self.wallet {
            match w.default_address() {
                Ok(a) => {
                    summary.push(format!("address: {a}"));
                    default_address = Some(a);
                }
                Err(e) => summary.push(format!("address: {e}")),
            }
            match w.chain_height() {
                Ok(h) => summary.push(format!("chain height: {h}")),
                Err(e) => summary.push(format!("chain height: {e}")),
            }
            match w.held_capability_count() {
                Ok(c) => summary.push(format!("held capabilities: {c}")),
                Err(e) => summary.push(format!("held capabilities: {e}")),
            }
            match w.capability_balance() {
                Ok(balances) if balances.is_empty() => {
                    summary.push("native balance: (none)".to_string())
                }
                Ok(balances) => {
                    for (asset, amount) in balances {
                        summary.push(format!("native balance: {asset} = {amount}"));
                    }
                }
                Err(e) => summary.push(format!("native balance: {e}")),
            }
            caps = w.held_capability_views().unwrap_or_default();
            addresses = w.addresses().unwrap_or_default();
            contracts = w.contract_views();
            contract_manifests = w.genesis_manifest_views().unwrap_or_default();
        }

        self.summary = summary;
        self.default_address = default_address;
        self.caps = caps;
        self.addresses = addresses;
        self.contracts = contracts;
        self.contract_manifests = contract_manifests;
    }

    fn show_screen(&mut self, ui: &mut egui::Ui) {
        match self.nav {
            NavScreen::Overview => {
                ui.heading("Overview");
                ui.separator();
                {
                    let vm = self.vm.read();
                    ui.monospace(format!(
                        "sync: height={} peers={} synced={}",
                        vm.sync_height, vm.peer_count, vm.synced
                    ));
                }
                for line in &self.summary {
                    ui.monospace(line);
                }
            }
            NavScreen::Capabilities => crate::ui::capabilities::show(ui, &self.caps),
            NavScreen::Addresses => crate::ui::addresses::show(
                ui,
                self.default_address.as_deref().unwrap_or(""),
                &self.addresses,
            ),
            NavScreen::Contracts => {
                crate::ui::contracts::show(ui, &self.contracts, &self.contract_manifests)
            }
            NavScreen::Console => {
                ui.heading("Console");
                ui.label("Type /help for commands. Run /verb <args> or a macro name.");
                ui.separator();
                let vm = self.vm.read();
                egui::ScrollArea::vertical()
                    .auto_shrink([false, false])
                    .show(ui, |ui| {
                        for line in &vm.console_log {
                            ui.monospace(line);
                        }
                    });
            }
        }
    }
}

impl eframe::App for InvarApp {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        match self.screen {
            Screen::Onboarding => {
                egui::CentralPanel::default_margins().show(ui, |ui| {
                    if let Some(config) = onboarding::show(ui, &mut self.onboarding) {
                        self.open_wallet(config);
                    }
                });
            }
            Screen::Main => {
                egui::Panel::top(egui::Id::new("command_bar")).show(ui, |ui| {
                    ui.add_space(4.0);
                    ui.horizontal(|ui| {
                        ui.label("invar>");
                        let resp = ui.add(
                            egui::TextEdit::singleline(&mut self.command_input)
                                .desired_width(480.0)
                                .hint_text("/help"),
                        );
                        let enter =
                            resp.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter));
                        if enter || ui.button("Run").clicked() {
                            self.run_command();
                            resp.request_focus();
                        }
                        if ui.button("Refresh").clicked() {
                            self.refresh();
                        }
                    });
                    ui.add_space(4.0);
                });

                egui::Panel::left(egui::Id::new("nav"))
                    .default_size(140.0)
                    .show(ui, |ui| {
                        ui.add_space(4.0);
                        ui.selectable_value(&mut self.nav, NavScreen::Overview, "Overview");
                        ui.selectable_value(
                            &mut self.nav,
                            NavScreen::Capabilities,
                            "Capabilities",
                        );
                        ui.selectable_value(&mut self.nav, NavScreen::Addresses, "Addresses");
                        ui.selectable_value(&mut self.nav, NavScreen::Contracts, "Contracts");
                        ui.selectable_value(&mut self.nav, NavScreen::Console, "Console");
                    });

                egui::CentralPanel::default_margins().show(ui, |ui| {
                    self.show_screen(ui);
                });
            }
        }
    }
}
