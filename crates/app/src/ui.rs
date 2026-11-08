//! The editor shell: window layout, command dispatch, and the status bar.
//!
//! The shell owns no media and no project yet. It owns the workspace layout, which
//! panel has focus, and the help windows. Menus, keyboard shortcuts, and clicks all
//! produce the same [`Action`] values, which are applied in one place, so the
//! interface cannot behave one way from a menu and another way from a key press.

use std::path::PathBuf;

use eframe::egui;
use fireeffects_core::{Action, Panel, Project, Workspace};

use crate::{dialogs, keys, menu, panels, project_io, theme};

/// The application state.
#[derive(Default)]
pub struct FireEffectsApp {
    workspace: Workspace,
    dialogs: dialogs::Dialogs,
    project: Option<Project>,
    project_path: Option<PathBuf>,
    project_dirty: bool,
    /// The last command that could not be carried out, for the status bar.
    notice: Option<String>,
}

impl FireEffectsApp {
    /// Creates the shell and applies the theme. eframe calls this once.
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        theme::apply(&cc.egui_ctx);
        Self::default()
    }

    /// Draws one frame of the shell.
    ///
    /// Kept separate from [`eframe::App`] so the shell can be exercised without a
    /// window, which is how its interactions are tested.
    pub fn draw(&mut self, ui: &mut egui::Ui) {
        let mut action = ui.input(keys::pressed);

        egui::Panel::top("menu_bar")
            .frame(chrome(egui::Margin::symmetric(8, 4)))
            .show(ui, |ui| {
                if let Some(chosen) = menu::bar(ui, &self.workspace) {
                    action = Some(chosen);
                }
            });

        egui::Panel::bottom("status_bar")
            .frame(chrome(egui::Margin::symmetric(8, 4)))
            .show(ui, |ui| self.status_bar(ui));

        self.draw_workspace(ui, &mut action);
        if let Some(dialog_action) = dialogs::show(ui.ctx(), &mut self.dialogs) {
            action = Some(dialog_action);
        }

        if let Some(action) = action {
            self.apply(action, ui.ctx());
        }
    }

    /// The four workspace panels, in the order that gives them their regions.
    fn draw_workspace(&self, ui: &mut egui::Ui, action: &mut Option<Action>) {
        let workspace = &self.workspace;

        if workspace.is_visible(Panel::Media) {
            egui::Panel::left("media_panel")
                .resizable(true)
                .default_size(260.0)
                .min_size(180.0)
                .frame(surface(theme::SURFACE))
                .show(ui, |ui| draw_panel(ui, Panel::Media, workspace, self.project.as_ref(), action));
        }

        if workspace.is_visible(Panel::Inspector) {
            egui::Panel::right("inspector_panel")
                .resizable(true)
                .default_size(280.0)
                .min_size(180.0)
                .frame(surface(theme::SURFACE))
                .show(ui, |ui| {
                    draw_panel(ui, Panel::Inspector, workspace, self.project.as_ref(), action);
                });
        }

        if workspace.is_visible(Panel::Timeline) {
            egui::Panel::bottom("timeline_panel")
                .resizable(true)
                .default_size(240.0)
                .min_size(120.0)
                .frame(surface(theme::SURFACE))
                .show(ui, |ui| {
                    draw_panel(ui, Panel::Timeline, workspace, self.project.as_ref(), action);
                });
        }

        if workspace.is_visible(Panel::Preview) {
            egui::CentralPanel::default()
                .frame(surface(theme::CANVAS))
                .show(ui, |ui| draw_panel(ui, Panel::Preview, workspace, self.project.as_ref(), action));
        }
    }

    fn status_bar(&self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            ui.label(
                egui::RichText::new(format!(
                    "Focus: {}",
                    panels::title(self.workspace.focused())
                ))
                .color(theme::EMBER)
                .strong(),
            );

            if let Some(project) = &self.project {
                let state = if self.project_dirty { " · Unsaved" } else { " · Saved" };
                ui.label(
                    egui::RichText::new(format!("{}{}", project.name, state))
                        .color(theme::TEXT_DIM),
                );
            } else {
                ui.label(egui::RichText::new("No project open").color(theme::TEXT_DIM));
            }

            if let Some(notice) = &self.notice {
                ui.label(egui::RichText::new(notice).color(theme::TEXT_DIM));
            }

            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                ui.label(egui::RichText::new(dialogs::shortcut_hint()).color(theme::TEXT_DIM));
            });
        });
    }

    /// Carries out one command. Every path through the interface ends up here.
    fn apply(&mut self, action: Action, ctx: &egui::Context) {
        self.notice = None;

        match action {
            Action::TogglePanel(panel) => {
                if let Err(error) = self.workspace.toggle(panel) {
                    self.notice = Some(sentence(error.to_string()));
                }
            }
            Action::ShowPanel(panel) => self.workspace.reveal(panel),
            Action::FocusNext => self.workspace.focus_next(),
            Action::FocusPrevious => self.workspace.focus_previous(),
            Action::NewProject => {
                if self.project_dirty {
                    self.notice = Some("Save the current project before creating another".to_owned());
                } else {
                    self.dialogs.new_project_name = "Untitled".to_owned();
                    self.dialogs.new_project = true;
                }
            }
            Action::OpenProject => {
                if self.project_dirty {
                    self.notice = Some("Save the current project before opening another".to_owned());
                } else {
                    self.dialogs.open_project = true;
                }
            }
            Action::SaveProject => self.save_current_project(),
            Action::SaveProjectAs => {
                if self.project.is_some() {
                    self.dialogs.save_project_path = self.project_path.as_ref()
                        .map(|path| path.display().to_string())
                        .unwrap_or_else(|| "Untitled.firefx".to_owned());
                    self.dialogs.save_as = true;
                } else {
                    self.notice = Some("No project is open".to_owned());
                }
            }
            Action::CreateProject => {
                let name = self.dialogs.new_project_name.trim();
                if name.is_empty() {
                    self.notice = Some("Enter a project name".to_owned());
                } else {
                    self.project = Some(Project::new(name));
                    self.project_path = None;
                    self.project_dirty = true;
                    self.dialogs.new_project = false;
                    self.notice = Some("New project created; save it to keep your work".to_owned());
                }
            }
            Action::LoadProject => {
                let path = PathBuf::from(self.dialogs.open_project_path.trim());
                match project_io::read_project(&path) {
                    Ok(project) => {
                        self.project = Some(project);
                        self.project_path = Some(path);
                        self.project_dirty = false;
                        self.dialogs.open_project = false;
                        self.notice = Some("Project opened".to_owned());
                    }
                    Err(error) => self.notice = Some(sentence(error)),
                }
            }
            Action::SaveProjectToPath => self.save_project_as(),
            Action::ShowShortcuts => self.dialogs.shortcuts = true,
            Action::ShowAbout => self.dialogs.about = true,
            Action::Quit => ctx.send_viewport_cmd(egui::ViewportCommand::Close),
        }
    }

    fn save_current_project(&mut self) {
        let Some(path) = self.project_path.clone() else {
            if let Some(project) = &self.project {
                self.dialogs.save_project_path = format!("{}.firefx", project.name);
                self.dialogs.save_as = true;
            } else {
                self.notice = Some("No project is open".to_owned());
            }
            return;
        };

        let Some(project) = self.project.as_ref() else {
            self.notice = Some("No project is open".to_owned());
            return;
        };

        match project_io::write_project(&path, project, true) {
            Ok(()) => {
                self.project_dirty = false;
                self.notice = Some("Project saved".to_owned());
            }
            Err(error) => self.notice = Some(sentence(error)),
        }
    }

    fn save_project_as(&mut self) {
        let Some(project) = self.project.as_ref() else {
            self.notice = Some("No project is open".to_owned());
            return;
        };
        let entered = self.dialogs.save_project_path.trim();
        if entered.is_empty() {
            self.notice = Some("Enter a destination path".to_owned());
            return;
        }
        let path = match project_io::normalize_save_path(&PathBuf::from(entered)) {
            Ok(path) => path,
            Err(error) => {
                self.notice = Some(sentence(error));
                return;
            }
        };
        let overwrite = self.project_path.as_ref() == Some(&path);

        match project_io::write_project(&path, project, overwrite) {
            Ok(()) => {
                self.project_path = Some(path);
                self.project_dirty = false;
                self.dialogs.save_as = false;
                self.notice = Some("Project saved".to_owned());
            }
            Err(error) => self.notice = Some(sentence(error)),
        }
    }
}

impl eframe::App for FireEffectsApp {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        self.draw(ui);
    }
}

fn draw_panel(
    ui: &mut egui::Ui,
    panel: Panel,
    workspace: &Workspace,
    project: Option<&Project>,
    action: &mut Option<Action>,
) {
    panels::draw(ui, panel, workspace, project, action);
}

/// A panel surface: a fill plus the margin that keeps text off the resize handles.
fn surface(fill: egui::Color32) -> egui::Frame {
    egui::Frame::new()
        .fill(fill)
        .inner_margin(egui::Margin::same(10))
}

/// The menu bar and status bar, which stay flat against the window edge.
fn chrome(margin: egui::Margin) -> egui::Frame {
    egui::Frame::new().fill(theme::CHROME).inner_margin(margin)
}

/// Capitalises a message and ends it with a full stop, so a notice reads as a
/// sentence rather than as a log line.
fn sentence(text: String) -> String {
    if text.is_empty() {
        return text;
    }

    let mut text = text;
    if let Some(first) = text.chars().next() {
        let capitalised = first.to_uppercase().collect::<String>();
        text.replace_range(..first.len_utf8(), &capitalised);
    }
    if !text.ends_with('.') {
        text.push('.');
    }
    text
}

#[cfg(test)]
mod tests {
    use super::*;
    use egui_kittest::Harness;
    use egui_kittest::kittest::Queryable;

    /// The modifier flags a real window backend reports while Ctrl is held: it
    /// sets both `ctrl` and the cross-platform `command` flag, which is what the
    /// shell binds. `egui::Modifiers::CTRL` only sets `ctrl`, so it would not
    /// exercise the same path a user's key press takes.
    fn ctrl() -> egui::Modifiers {
        egui::Modifiers {
            command: true,
            ctrl: true,
            ..Default::default()
        }
    }

    /// A headless window running the real shell, with fixed size and Linux key
    /// semantics so results do not depend on the machine.
    fn harness() -> Harness<'static, FireEffectsApp> {
        Harness::builder()
            .with_size(egui::Vec2::new(1280.0, 800.0))
            .with_os(egui::os::OperatingSystem::Nix)
            .build_ui_state(
                |ui: &mut egui::Ui, app: &mut FireEffectsApp| app.draw(ui),
                FireEffectsApp::default(),
            )
    }

    #[test]
    fn opens_with_every_panel_visible() {
        let mut harness = harness();
        harness.run();

        for panel in Panel::ALL {
            let title = panels::title(panel);
            assert!(
                harness.query_by_label(title).is_some(),
                "the {title} panel is missing from the opening window"
            );
        }
    }

    #[test]
    fn clicking_a_panel_header_focuses_it() {
        let mut harness = harness();
        harness.run();

        harness.get_by_label("Inspector").click();
        harness.run();

        assert_eq!(harness.state().workspace.focused(), Panel::Inspector);
    }

    #[test]
    fn the_view_menu_hides_a_panel() {
        let mut harness = harness();
        harness.run();

        harness.get_by_label("View").click();
        harness.run();
        harness.get_by_label("Media panel").click();
        harness.run();

        assert!(!harness.state().workspace.is_visible(Panel::Media));
        assert!(
            harness.query_by_label("Media").is_none(),
            "the Media panel was still drawn after being hidden"
        );
    }

    #[test]
    fn shortcuts_show_a_panel_and_cycle_focus() {
        let mut harness = harness();
        harness.run();

        harness.key_press_modifiers(ctrl(), egui::Key::Num3);
        harness.run();
        assert_eq!(harness.state().workspace.focused(), Panel::Timeline);

        harness.key_press(egui::Key::F6);
        harness.run();
        assert_eq!(harness.state().workspace.focused(), Panel::Inspector);
    }

    #[test]
    fn a_shortcut_reopens_a_hidden_panel() {
        let mut harness = harness();
        harness
            .state_mut()
            .workspace
            .toggle(Panel::Preview)
            .expect("other panels remain");
        harness.run();

        harness.key_press_modifiers(ctrl(), egui::Key::Num2);
        harness.run();

        assert!(harness.state().workspace.is_visible(Panel::Preview));
        assert_eq!(harness.state().workspace.focused(), Panel::Preview);
        assert!(harness.query_by_label("Preview").is_some());
    }

    #[test]
    fn the_help_menu_opens_the_shortcut_reference() {
        let mut harness = harness();
        harness.run();

        harness.get_by_label("Help").click();
        harness.run();
        // The menu shows the chord next to the item, so the label reads
        // "Keyboard shortcuts F1"; match on the prefix.
        harness.get_by_label_contains("Keyboard shortcuts").click();
        harness.run();

        assert!(harness.state().dialogs.shortcuts);
        assert!(
            harness
                .query_by_label("Focus the next visible panel")
                .is_some(),
            "the shortcut reference did not list the focus command"
        );
    }

    #[test]
    fn a_refused_command_is_explained_in_the_status_bar() {
        let mut harness = harness();

        for panel in [Panel::Media, Panel::Preview, Panel::Timeline] {
            harness
                .state_mut()
                .workspace
                .toggle(panel)
                .expect("one panel remains visible");
        }
        harness.run();

        harness.get_by_label("View").click();
        harness.run();
        harness.get_by_label("Inspector panel").click();
        harness.run();

        let notice = harness
            .state()
            .notice
            .clone()
            .expect("a notice explaining the refusal");
        assert!(
            notice.contains("last visible panel"),
            "unexpected notice: {notice}"
        );
        assert!(harness.state().workspace.is_visible(Panel::Inspector));
    }

    #[test]
    fn notices_read_as_sentences() {
        assert_eq!(
            sentence("the last visible panel cannot be hidden".to_owned()),
            "The last visible panel cannot be hidden."
        );
        assert_eq!(sentence(String::new()), String::new());
        assert_eq!(sentence("Already fine.".to_owned()), "Already fine.");
    }
}
