//! The application window shown by the Stage 0 foundation build.
//!
//! Everything here is a read-out of what the repository actually implements. No
//! control is drawn unless it does something, which is why this build has no
//! menus, panels, or buttons yet.

use eframe::egui;
use fireeffects_core::FrameRate;

/// Ember accent used for the FireEffects wordmark.
const EMBER: egui::Color32 = egui::Color32::from_rgb(232, 122, 61);

/// Capabilities that exist in this build, paired with what the user can expect.
const IMPLEMENTED: &[(&str, &str)] = &[
    (
        "Application window",
        "A native window opens, resizes, and keeps repainting.",
    ),
    (
        "Core crate",
        "Platform-independent types build and test without a display.",
    ),
];

/// Roadmap stages that are not implemented yet, so nothing here is mistaken for a feature.
const PLANNED: &[(&str, &str)] = &[
    (
        "Stage 1 — Editor shell",
        "Panels, menus, selection, keyboard behaviour.",
    ),
    (
        "Stage 2 — Projects and media",
        "Project files, import, media metadata.",
    ),
    (
        "Stage 3 — Timeline editing",
        "Tracks, clips, trimming, undo/redo.",
    ),
    (
        "Stage 4 — Preview and playback",
        "Playhead preview, play, pause, seek.",
    ),
    (
        "Stage 5 — Compositing",
        "Layers, transform properties, keyframes.",
    ),
    (
        "Stage 6 — Effects, titles, audio",
        "Effect stack, text, audio tracks.",
    ),
    (
        "Stage 7 — Export",
        "Export settings, progress, cancellation.",
    ),
];

/// The application state owned by the UI loop.
pub struct FireEffectsApp;

impl FireEffectsApp {
    /// Creates the application. No project or media state exists yet.
    pub fn new(_cc: &eframe::CreationContext<'_>) -> Self {
        Self
    }
}

impl eframe::App for FireEffectsApp {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ui, |ui| {
            egui::ScrollArea::vertical().show(ui, |ui| {
                ui.add_space(8.0);
                ui.label(
                    egui::RichText::new("FireEffects")
                        .color(EMBER)
                        .size(28.0)
                        .strong(),
                );
                ui.label(egui::RichText::new("Stage 0 — Foundation").weak());
                ui.label(format!("Version {}", env!("CARGO_PKG_VERSION")));

                ui.add_space(4.0);
                ui.separator();
                ui.add_space(4.0);

                ui.label(egui::RichText::new("Implemented in this build").strong());
                for (name, detail) in IMPLEMENTED {
                    ui.label(format!("{name} — {detail}"));
                }

                ui.add_space(12.0);
                ui.label(egui::RichText::new("Not implemented yet").strong());
                for (stage, detail) in PLANNED {
                    ui.label(egui::RichText::new(format!("{stage} — {detail}")).weak());
                }

                ui.add_space(12.0);
                ui.separator();
                ui.add_space(4.0);
                ui.label(format!(
                    "Frame rates understood by the core time model: {}",
                    supported_frame_rates()
                ));
                ui.label(egui::RichText::new("Roadmap: README section 2.").weak());
            });
        });
    }
}

/// Lists the rational frame rates the core crate defines, for the status read-out.
fn supported_frame_rates() -> String {
    const RATES: [FrameRate; 8] = [
        FrameRate::FPS_23_976,
        FrameRate::FPS_24,
        FrameRate::FPS_25,
        FrameRate::FPS_29_97,
        FrameRate::FPS_30,
        FrameRate::FPS_50,
        FrameRate::FPS_59_94,
        FrameRate::FPS_60,
    ];

    RATES
        .iter()
        .map(ToString::to_string)
        .collect::<Vec<_>>()
        .join(", ")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn status_readout_lists_every_defined_rate() {
        let rates = supported_frame_rates();
        assert_eq!(rates.split(", ").count(), 8);
        assert!(
            rates.contains("23.976 fps"),
            "missing NTSC film rate: {rates}"
        );
        assert!(
            rates.contains("59.94 fps"),
            "missing NTSC high rate: {rates}"
        );
        assert!(rates.contains("25 fps"), "missing PAL rate: {rates}");
    }

    #[test]
    fn planned_stages_are_all_labelled() {
        assert!(PLANNED.iter().all(|(stage, _)| stage.starts_with("Stage ")));
        assert_eq!(PLANNED.len(), 7);
    }
}
