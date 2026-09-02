//! Contracts screen — the standard (genesis) DarkWow contracts, plus a placeholder for
//! manifest-discovered contracts (later milestone).

use crate::viewmodel::ContractView;

pub fn show(ui: &mut egui::Ui, contracts: &[ContractView]) {
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

    ui.add_space(8.0);
    ui.label("Discovered (non-genesis) contracts appear here once scanned (later milestone).");
}
