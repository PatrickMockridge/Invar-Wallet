//! Contracts screen — the standard (genesis) contracts and their on-chain manifests.

use crate::viewmodel::{ContractManifestView, ContractView};

pub fn show(
    ui: &mut egui::Ui,
    contracts: &[ContractView],
    manifests: &[ContractManifestView],
) {
    ui.heading("Contracts");
    ui.label(format!("{} genesis contracts (standard)", contracts.len()));
    ui.separator();

    egui::Grid::new("contracts_grid")
        .num_columns(3)
        .striped(true)
        .spacing([16.0, 4.0])
        .show(ui, |ui| {
            ui.label("name");
            ui.label("trust");
            ui.label("contract id");
            ui.end_row();
            for c in contracts {
                ui.monospace(&c.name);
                ui.monospace(&c.trust);
                ui.monospace(&c.contract_id);
                ui.end_row();
            }
        });

    ui.add_space(12.0);
    ui.label("Manifests:");
    ui.separator();

    if manifests.is_empty() {
        ui.label("No manifests stored yet — initialize the wallet and sync to discover contracts.");
        return;
    }

    egui::ScrollArea::vertical()
        .auto_shrink([false, false])
        .show(ui, |ui| {
            for m in manifests {
                egui::CollapsingHeader::new(format!(
                    "{} [{}] {} v{}",
                    m.name, m.trust, m.category, m.version
                ))
                .show(ui, |ui| {
                    if !m.description.is_empty() {
                        ui.label(&m.description);
                    }
                    if !m.functions.is_empty() {
                        ui.label("Functions:");
                        for f in &m.functions {
                            ui.monospace(format!("  {f}"));
                        }
                    }
                    if !m.capabilities.is_empty() {
                        ui.label("Capabilities:");
                        for c in &m.capabilities {
                            ui.monospace(format!("  {c}"));
                        }
                    }
                    if !m.actions.is_empty() {
                        ui.label("Actions:");
                        for a in &m.actions {
                            ui.monospace(format!("  {a}"));
                        }
                    }
                    if !m.parameters.is_empty() {
                        ui.label("Parameters:");
                        for p in &m.parameters {
                            ui.monospace(format!("  {p}"));
                        }
                    }
                });
            }
        });
}
