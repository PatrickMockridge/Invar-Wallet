//! Addresses screen — the wallet's receiving address (encryption target), its QR code,
//! and the derived addresses.

use std::collections::HashMap;
use std::sync::{Mutex, OnceLock};

use qrcode::QrCode;

static QR_CACHE: OnceLock<Mutex<HashMap<String, egui::TextureHandle>>> = OnceLock::new();

pub fn show(ui: &mut egui::Ui, default: &str, addresses: &[String]) {
    ui.heading("Addresses");
    ui.separator();

    ui.label("Default (receiving) address:");
    ui.monospace(default);

    if !default.is_empty() {
        if let Some(texture) = qr_texture(ui.ctx(), default) {
            ui.add_space(6.0);
            ui.image((texture.id(), egui::Vec2::splat(200.0)));
        }
    }

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

/// Render (and cache) a QR code for `address` as an egui texture.
fn qr_texture(ctx: &egui::Context, address: &str) -> Option<egui::TextureHandle> {
    let cache = QR_CACHE.get_or_init(|| Mutex::new(HashMap::new()));
    if let Some(t) = cache.lock().unwrap().get(address) {
        return Some(t.clone());
    }

    let code = QrCode::new(address.as_bytes()).ok()?;
    let img = code
        .render::<image::Luma<u8>>()
        .min_dimensions(200, 200)
        .build();
    let size = [img.width() as usize, img.height() as usize];
    let pixels: Vec<egui::Color32> = img
        .pixels()
        .map(|p| egui::Color32::from_rgb(p.0[0], p.0[0], p.0[0]))
        .collect();

    let texture = ctx.load_texture(
        format!("invar-qr:{address}"),
        egui::ColorImage::new(size, pixels),
        egui::TextureOptions::NEAREST,
    );
    cache.lock().unwrap().insert(address.to_string(), texture.clone());
    Some(texture)
}
