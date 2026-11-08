//! Help and project dialogs. Project paths are entered explicitly so the
//! application does not need a platform-specific file-dialog dependency yet.

use eframe::egui;
use fireeffects_core::{Action, BINDINGS, binding_for};

use crate::{panels, theme};

/// Which windows and project dialogs are open.
#[derive(Debug, Default)]
pub struct Dialogs {
    pub shortcuts: bool,
    pub about: bool,
    pub new_project: bool,
    pub open_project: bool,
    pub save_as: bool,
    pub new_project_name: String,
    pub open_project_path: String,
    pub save_project_path: String,
}

/// Draw open dialogs and return the project command submitted this frame.
pub fn show(ctx: &egui::Context, dialogs: &mut Dialogs) -> Option<Action> {
    let mut action = None;

    if dialogs.new_project {
        let mut open = dialogs.new_project;
        let mut name = dialogs.new_project_name.clone();
        egui::Window::new("New project")
            .open(&mut open)
            .collapsible(false)
            .resizable(false)
            .show(ctx, |ui| {
                ui.label("Project name");
                ui.text_edit_singleline(&mut name);
                ui.add_space(8.0);
                if ui.button("Create project").clicked() {
                    action = Some(Action::CreateProject);
                }
            });
        dialogs.new_project = open;
        dialogs.new_project_name = name;
    }

    if dialogs.open_project {
        let mut open = dialogs.open_project;
        let mut path = dialogs.open_project_path.clone();
        egui::Window::new("Open project")
            .open(&mut open)
            .collapsible(false)
            .resizable(false)
            .show(ctx, |ui| {
                ui.label("Path to a .firefx project");
                ui.text_edit_singleline(&mut path);
                ui.add_space(8.0);
                if ui.button("Open").clicked() {
                    action = Some(Action::LoadProject);
                }
            });
        dialogs.open_project = open;
        dialogs.open_project_path = path;
    }

    if dialogs.save_as {
        let mut open = dialogs.save_as;
        let mut path = dialogs.save_project_path.clone();
        egui::Window::new("Save project as")
            .open(&mut open)
            .collapsible(false)
            .resizable(false)
            .show(ctx, |ui| {
                ui.label("Destination path (the .firefx extension is added if omitted)");
                ui.text_edit_singleline(&mut path);
                ui.add_space(8.0);
                if ui.button("Save project").clicked() {
                    action = Some(Action::SaveProjectToPath);
                }
            });
        dialogs.save_as = open;
        dialogs.save_project_path = path;
    }

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

    action
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
    ui.label("Stage 2: native project creation and file persistence are implemented.");
    for stage in [
        "Media import and metadata are the next Stage 2 work.",
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
        Action::TogglePanel(panel) => format!("Show or hide the {} panel", panels::title(panel)),
        Action::ShowPanel(panel) => {
            format!("Show the {} panel and give it focus", panels::title(panel))
        }
        Action::FocusNext => "Focus the next visible panel".to_owned(),
        Action::FocusPrevious => "Focus the previous visible panel".to_owned(),
        Action::NewProject => "Open the new-project dialog".to_owned(),
        Action::OpenProject => "Open the project-file dialog".to_owned(),
        Action::SaveProject => "Save the current project".to_owned(),
        Action::SaveProjectAs => "Open the Save As dialog".to_owned(),
        Action::CreateProject => "Create a project".to_owned(),
        Action::LoadProject => "Open a project file".to_owned(),
        Action::SaveProjectToPath => "Save the project to a selected path".to_owned(),
        Action::ShowShortcuts => "Open this window".to_owned(),
        Action::ShowAbout => "Open the About window".to_owned(),
        Action::Quit => "Close FireEffects".to_owned(),
    }
}

fn hint(ui: &mut egui::Ui, text: &str) {
    ui.label(egui::RichText::new(text).color(theme::TEXT_DIM));
}
