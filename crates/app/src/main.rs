//! FireEffects desktop entry point.
//!
//! Stage 0 keeps this deliberately small: create a native window, apply the
//! application identity, and hand over to the UI module. There are no documents,
//! media, or editing features yet.

mod ui;

use eframe::egui;

fn main() -> eframe::Result {
    let native_options = eframe::NativeOptions {
        renderer: eframe::Renderer::Wgpu,
        viewport: egui::ViewportBuilder::default()
            .with_title("FireEffects")
            .with_inner_size([1100.0, 720.0])
            .with_min_inner_size([760.0, 480.0]),
        ..Default::default()
    };

    eframe::run_native(
        "FireEffects",
        native_options,
        Box::new(|cc| Ok(Box::new(ui::FireEffectsApp::new(cc)))),
    )
}
