//! Help windows: the keyboard shortcut reference and About.
//!
//! The shortcut list is generated from `fireeffects_core::BINDINGS`, the same table
//! the window uses to decide what a key press does, so a shortcut cannot be
//! documented without also working.

use eframe::egui;
use fireeffects_core::{Action, BINDINGS, binding_for};

use crate::{panels, theme};

/// Which help windows are open.
#[derive(Debug, Default)]
pub struct Dialogs {
    /// The keyboard shortcut reference.
    pub shortcuts: bool,
    /// The About window.
    pub about: bool,
}

/// Draws any open help window.
pub fn show(ctx: &egui::Context, dialogs: &mut Dialogs) {
    if dialogs.shortcuts {
        egui::Window::new("Keyboard shortcuts")
            .open(&mut dialogs.shortcuts)
            .collapsible(false)
            .resizable(false)
            .show(ctx, shortcuts);
    }

    if dialogs.about {
        egui::Window::new("About FireEffects")
            .open(&mut dialogs.about)
            .collapsible(false)
            .resizable(false)
            .show(ctx, about);
    }
}

/// The chord that opens the shortcut reference, for hints elsewhere in the window.
pub fn shortcut_hint() -> String {
    match binding_for(Action::ShowShortcuts) {
        Some(binding) => format!("{binding} for shortcuts"),
        None => "Shortcuts: see Help menu".to_owned(),
    }
}

fn shortcuts(ui: &mut egui::Ui) {
    egui::Grid::new("shortcut_grid")
        .num_columns(2)
        .spacing([24.0, 6.0])
        .show(ui, |ui| {
            for binding in BINDINGS {
                ui.label(egui::RichText::new(binding.to_string()).strong());
                ui.label(describe(binding.action));
                ui.end_row();
            }
        });

    ui.add_space(8.0);
    ui.separator();
    ui.add_space(4.0);
    hint(ui, "On macOS, Ctrl means Command.");
    hint(ui, "Click a panel header to focus it.");
    hint(
        ui,
        "Panel visibility can also be toggled from the View menu.",
    );
}

fn about(ui: &mut egui::Ui) {
    ui.label(
        egui::RichText::new("FireEffects")
            .color(theme::EMBER)
            .size(22.0)
            .strong(),
    );
    ui.label("An independent desktop video editor.");
    ui.label(egui::RichText::new(format!("Version {}", env!("CARGO_PKG_VERSION"))).weak());

    ui.add_space(8.0);
    ui.separator();
    ui.add_space(4.0);

    ui.label("This build is the Stage 1 editor shell. Not implemented yet:");
    for stage in [
        "Stage 2: project files, media import, saved state",
        "Stage 3: tracks, clips, and timeline editing",
        "Stage 4: preview and playback",
        "Stage 5: compositing and keyframes",
        "Stage 6: effects, titles, and audio",
        "Stage 7: export",
    ] {
        hint(ui, stage);
    }

    ui.add_space(8.0);
    ui.separator();
    ui.add_space(4.0);
    hint(ui, "Not affiliated with Adobe or any other vendor.");
    hint(
        ui,
        "Licence not chosen yet; see docs/ARCHITECTURE.md in the repository.",
    );
}

/// What a command does, in the words used by the shortcut reference.
fn describe(action: Action) -> String {
    match action {
        Action::TogglePanel(panel) => {
            format!("Show or hide the {} panel", panels::title(panel))
        }
        Action::ShowPanel(panel) => {
            format!("Show the {} panel and give it focus", panels::title(panel))
        }
        Action::FocusNext => "Focus the next visible panel".to_owned(),
        Action::FocusPrevious => "Focus the previous visible panel".to_owned(),
        Action::ShowShortcuts => "Open this window".to_owned(),
        Action::ShowAbout => "Open the About window".to_owned(),
        Action::Quit => "Close FireEffects".to_owned(),
    }
}

fn hint(ui: &mut egui::Ui, text: &str) {
    ui.label(egui::RichText::new(text).color(theme::TEXT_DIM));
}
