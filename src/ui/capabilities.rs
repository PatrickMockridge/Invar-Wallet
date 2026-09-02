//! Capability browser — the wallet's held capabilities (typed, not coins).

use crate::viewmodel::CapView;

pub fn show(ui: &mut egui::Ui, caps: &[CapView]) {
    ui.heading("Capabilities");
    ui.label(format!("{} held", caps.len()));
    ui.separator();

    if caps.is_empty() {
        ui.label("No held capabilities yet — sync + scan to discover them.");
        return;
    }

    egui::ScrollArea::vertical()
        .auto_shrink([false, false])
        .show(ui, |ui| {
            for cap in caps {
                ui.monospace(format!("{}  value={}  {}", cap.status, cap.value, cap.name));
                ui.monospace(format!("  contract: {} [{}]", cap.contract_name, cap.trust));
                ui.monospace(format!("  asset: {}", cap.asset_id));
                ui.monospace(format!("  id: {}", cap.cap_id));
                ui.monospace(format!(
                    "  resource={} action={} leaf={} height={}",
                    cap.resource, cap.action, cap.leaf_position, cap.created_at_height
                ));
                ui.monospace(format!("  primitives: {}", cap.primitives));
                ui.monospace(format!("  barbs: {}", cap.barbs));
                ui.separator();
            }
        });
}
