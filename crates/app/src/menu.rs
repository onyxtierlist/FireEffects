//! The menu bar.
//!
//! Only commands that work today appear here. Project actions such as New, Open,
//! and Save arrive with the project format in Stage 2, and their absence is
//! explained by the Media panel rather than by a disabled menu item that pretends
//! the feature is one setting away.

use eframe::egui;
use fireeffects_core::{Action, Panel, Workspace, binding_for};

use crate::{panels, theme};

/// Draws the wordmark and the menus, returning the command chosen this frame.
pub fn bar(ui: &mut egui::Ui, workspace: &Workspace) -> Option<Action> {
    let mut action = None;

    ui.horizontal(|ui| {
        ui.label(
            egui::RichText::new("FireEffects")
                .color(theme::EMBER)
                .size(16.0)
                .strong(),
        );

        egui::MenuBar::new().ui(ui, |ui| {
            ui.menu_button("File", |ui| {
                for (label, command_action) in [
                    ("New Project", Action::NewProject),
                    ("Open Project…", Action::OpenProject),
                    ("Save", Action::SaveProject),
                    ("Save As…", Action::SaveProjectAs),
                ] {
                    if command(ui, label, command_action).clicked() {
                        ui.close();
                        action = Some(command_action);
                    }
                }
                ui.separator();
                if command(ui, "Quit", Action::Quit).clicked() {
                    ui.close();
                    action = Some(Action::Quit);
                }
            });

            ui.menu_button("View", |ui| {
                for panel in Panel::ALL {
                    let label = format!("{} panel", panels::title(panel));
                    let entry = egui::Button::new(label).selected(workspace.is_visible(panel));
                    if ui.add(entry).clicked() {
                        ui.close();
                        action = Some(Action::TogglePanel(panel));
                    }
                }
            });

            ui.menu_button("Help", |ui| {
                if command(ui, "Keyboard shortcuts", Action::ShowShortcuts).clicked() {
                    ui.close();
                    action = Some(Action::ShowShortcuts);
                }
                if command(ui, "About FireEffects", Action::ShowAbout).clicked() {
                    ui.close();
                    action = Some(Action::ShowAbout);
                }
            });
        });
    });

    action
}

/// A menu entry that shows the chord bound to `action`, if it has one.
fn command(ui: &mut egui::Ui, label: &str, action: Action) -> egui::Response {
    let entry = egui::Button::new(label);

    match binding_for(action) {
        Some(binding) => ui.add(entry.shortcut_text(binding.to_string())),
        None => ui.add(entry),
    }
}
