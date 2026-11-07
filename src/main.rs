mod app;
mod project;

use app::FireEffectsApp;

fn main() -> eframe::Result<()> {
    let options = eframe::NativeOptions {
        viewport: eframe::egui::ViewportBuilder::default()
            .with_title("FireEffects")
            .with_inner_size([1440.0, 900.0])
            .with_min_inner_size([900.0, 600.0]),
        ..Default::default()
    };

    eframe::run_native(
        "FireEffects",
        options,
        Box::new(|creation_context| {
            Ok(Box::new(FireEffectsApp::new(creation_context)))
        }),
    )
}
