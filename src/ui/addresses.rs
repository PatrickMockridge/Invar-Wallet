//! Addresses screen — the wallet's receiving address (encryption target) and derived keys.

pub fn show(ui: &mut egui::Ui, default: &str, addresses: &[String]) {
    ui.heading("Addresses");
    ui.separator();

    ui.label("Default (receiving) address:");
    ui.monospace(default);

    ui.add_space(8.0);
    ui.label(format!("{} derived addresses:", addresses.len()));
    egui::ScrollArea::vertical()
        .auto_shrink([false, false])
        .show(ui, |ui| {
            for (i, a) in addresses.iter().enumerate() {
                ui.monospace(format!("{i}: {a}"));
            }
        });
}
