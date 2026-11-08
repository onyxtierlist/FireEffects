//! FireEffects desktop entry point.
//!
//! This stays deliberately small: create the editor window with the configured
//! renderer, apply the application identity, and hand over to the shell. Project
//! files, media, and their dialogs live in the shell and the modules beside it.

mod dialogs;
mod files;
mod keys;
mod library;
mod menu;
mod panels;
mod project_io;
mod theme;
mod ui;

use eframe::egui;

fn main() -> eframe::Result {
    let native_options = eframe::NativeOptions {
        renderer: eframe::Renderer::Wgpu,
        viewport: egui::ViewportBuilder::default()
            .with_title("FireEffects")
            .with_inner_size([1280.0, 800.0])
            .with_min_inner_size([960.0, 600.0]),
        ..Default::default()
    };

    eframe::run_native(
        "FireEffects",
        native_options,
        Box::new(|cc| Ok(Box::new(ui::FireEffectsApp::new(cc)))),
    )
}
