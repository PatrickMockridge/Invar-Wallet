//! Invar theme — the single place to customise colours and visuals.
//!
//! To retheme a fork, edit `apply` (or replace it with your own palette) — every
//! screen inherits these visuals automatically.

/// Apply the Invar look (dark background, warm-gold accent evoking Electrum).
pub fn apply(ctx: &egui::Context) {
    let mut visuals = egui::Visuals::dark();

    visuals.panel_fill = egui::Color32::from_rgb(18, 18, 24);
    visuals.window_fill = egui::Color32::from_rgb(24, 24, 32);
    visuals.extreme_bg_color = egui::Color32::from_rgb(12, 12, 16);

    let accent = egui::Color32::from_rgb(212, 175, 55);
    let accent_bright = egui::Color32::from_rgb(230, 195, 80);
    visuals.selection.bg_fill = accent;
    visuals.selection.stroke = egui::Stroke::new(1.0, accent_bright);
    visuals.hyperlink_color = accent_bright;

    ctx.set_visuals(visuals);
}
