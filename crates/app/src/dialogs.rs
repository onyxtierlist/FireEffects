//! Help and project dialogs. Project paths are entered explicitly, so the
//! application keeps working without platform file dialogs; each dialog also
//! offers a native picker for the ordinary case.
//!
//! Nothing here changes project state. The windows report what the user asked for,
//! and `ui.rs` decides what that means — one place where the project mutates.

use eframe::egui;
use fireeffects_core::{Action, BINDINGS, binding_for};

use crate::{panels, theme};

/// A command waiting for the unsaved-changes question to be answered.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Intent {
    /// Start a new project.
    NewProject,
    /// Open a project file.
    OpenProject,
    /// Close the current project.
    CloseProject,
    /// Quit the application.
    Quit,
}

impl Intent {
    /// How the question refers to what the user asked for.
    pub fn description(self) -> &'static str {
        match self {
            Self::NewProject => "starting a new project",
            Self::OpenProject => "opening another project",
            Self::CloseProject => "closing this project",
            Self::Quit => "quitting",
        }
    }
}

/// What the user answered about unsaved changes.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Confirmation {
    /// Save the project, then continue.
    Save,
    /// Continue without saving.
    Discard,
    /// Do nothing.
    Cancel,
}

/// What the open windows produced this frame.
#[derive(Debug, Default)]
pub struct Outcome {
    /// A command a dialog button asked for.
    pub action: Option<Action>,
    /// An answer to the unsaved-changes question.
    pub confirmation: Option<Confirmation>,
}

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
    /// The command waiting on the unsaved-changes question.
    pub confirm: Option<Intent>,
}

/// Draw open dialogs and report what they asked for.
pub fn show(ctx: &egui::Context, dialogs: &mut Dialogs) -> Outcome {
    let mut outcome = Outcome::default();

    if dialogs.new_project {
        let mut open = dialogs.new_project;
        let mut name = dialogs.new_project_name.clone();
        egui::Window::new("New project")
            .open(&mut open)
            .collapsible(false)
            .resizable(false)
            .show(ctx, |ui| {
                let label = ui.label("Project name");
                ui.add(egui::TextEdit::singleline(&mut name))
                    .labelled_by(label.id);
                ui.add_space(8.0);
                if ui.button("Create project").clicked() {
                    outcome.action = Some(Action::CreateProject);
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
                let label = ui.label("Path to a .firefx project");
                ui.horizontal(|ui| {
                    ui.add(egui::TextEdit::singleline(&mut path))
                        .labelled_by(label.id);
                    if ui.button("Browse…").clicked() {
                        outcome.action = Some(Action::PickProjectFile);
                    }
                });
                ui.add_space(8.0);
                if ui.button("Open").clicked() {
                    outcome.action = Some(Action::LoadProject);
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
                let label =
                    ui.label("Destination path (the .firefx extension is added if omitted)");
                ui.horizontal(|ui| {
                    ui.add(egui::TextEdit::singleline(&mut path))
                        .labelled_by(label.id);
                    if ui.button("Browse…").clicked() {
                        outcome.action = Some(Action::PickSaveLocation);
                    }
                });
                ui.add_space(8.0);
                if ui.button("Save project").clicked() {
                    outcome.action = Some(Action::SaveProjectToPath);
                }
            });
        dialogs.save_as = open;
        dialogs.save_project_path = path;
    }

    if let Some(intent) = dialogs.confirm
        && let Some(answer) = confirm(ctx, intent)
    {
        dialogs.confirm = None;
        outcome.confirmation = Some(answer);
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

    outcome
}

/// Asks what to do about unsaved changes.
fn confirm(ctx: &egui::Context, intent: Intent) -> Option<Confirmation> {
    let mut answer = None;
    let mut open = true;

    egui::Window::new("Unsaved changes")
        .open(&mut open)
        .collapsible(false)
        .resizable(false)
        .show(ctx, |ui| {
            ui.label(format!(
                "This project has changes that are not saved. Save them before {}?",
                intent.description()
            ));

            ui.add_space(6.0);
            ui.horizontal(|ui| {
                if ui.button("Save").clicked() {
                    answer = Some(Confirmation::Save);
                }
                if ui.button("Discard changes").clicked() {
                    answer = Some(Confirmation::Discard);
                }
                if ui.button("Cancel").clicked() {
                    answer = Some(Confirmation::Cancel);
                }
            });
        });

    // Dismissing the window is cancel: doing nothing is the safe answer.
    if !open {
        Some(Confirmation::Cancel)
    } else {
        answer
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

    ui.label("Stages 0 to 2 are implemented: projects, media import, and relinking.");
    for stage in [
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
    hint(
        ui,
        "Media is listed without being decoded; preview arrives with Stage 4.",
    );
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
        Action::SaveProjectAs => "Save the project under a new name".to_owned(),
        Action::CreateProject => "Create a project".to_owned(),
        Action::LoadProject => "Open a project file".to_owned(),
        Action::SaveProjectToPath => "Save the project to a selected path".to_owned(),
        Action::CloseProject => "Close the project".to_owned(),
        Action::ImportMedia => "Import media files".to_owned(),
        Action::SelectMedia(_) => "Select a media item".to_owned(),
        Action::RelinkMedia(_) => "Point a missing media item at a new file".to_owned(),
        Action::PickProjectFile => "Choose a project file".to_owned(),
        Action::PickSaveLocation => "Choose where to save".to_owned(),
        Action::ShowShortcuts => "Open this window".to_owned(),
        Action::ShowAbout => "Open the About window".to_owned(),
        Action::Quit => "Close FireEffects".to_owned(),
    }
}

fn hint(ui: &mut egui::Ui, text: &str) {
    ui.label(egui::RichText::new(text).color(theme::TEXT_DIM));
}
