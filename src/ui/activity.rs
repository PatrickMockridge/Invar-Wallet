//! Activity screen — the capability spend-state lifecycle
//! (Unspent → Pending → Processing → Spent). The Rust wallet has no transaction-history
//! table, so "activity" is the capability lifecycle, not a tx list.

use crate::viewmodel::CapView;

pub fn show(ui: &mut egui::Ui, caps: &[CapView]) {
    ui.heading("Activity");
    ui.label("Capability spend-state lifecycle (Unspent → Pending → Processing → Spent).");
    ui.separator();

    if caps.is_empty() {
        ui.label("No held capabilities yet — sync + scan to discover them.");
        return;
    }

    let mut unspent: Vec<&CapView> = Vec::new();
    let mut pending: Vec<&CapView> = Vec::new();
    let mut processing: Vec<&CapView> = Vec::new();
    let mut spent: Vec<&CapView> = Vec::new();
    for c in caps {
        match c.status.as_str() {
            "pending" => pending.push(c),
            "processing" => processing.push(c),
            "spent" => spent.push(c),
            _ => unspent.push(c),
        }
    }

    egui::ScrollArea::vertical()
        .auto_shrink([false, false])
        .show(ui, |ui| {
            section(ui, "Unspent", &unspent);
            section(ui, "Pending", &pending);
            section(ui, "Processing", &processing);
            section(ui, "Spent", &spent);
        });
}

fn section(ui: &mut egui::Ui, title: &str, caps: &[&CapView]) {
    ui.add_space(6.0);
    ui.label(format!("{title} ({})", caps.len()));
    ui.separator();
    for c in caps {
        let status_h = c
            .status_height
            .map(|h| h.to_string())
            .unwrap_or_else(|| "-".into());
        let revoked_h = c
            .revoked_at_height
            .map(|h| h.to_string())
            .unwrap_or_else(|| "-".into());
        ui.monospace(format!(
            "{} value={} {} [{}]  created={} status@{} revoked@{}",
            c.name, c.value, c.contract_name, c.trust, c.created_at_height, status_h, revoked_h
        ));
        ui.monospace(format!("  id: {}", c.cap_id));
    }
}
