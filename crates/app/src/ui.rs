//! The editor shell: window layout, command dispatch, and the status bar.
//!
//! The shell owns the workspace, the current project, the media library, project
//! dialogs, and help windows. Menus, keyboard shortcuts, and clicks all produce the
//! same [`Action`] values, which are applied in one place so the interface cannot
//! drift between input paths.
//!
//! Two rules shape this module:
//!
//! - **Nothing is discarded silently.** Anything that would replace or close a
//!   project with unsaved changes asks first, including the window's own close
//!   button.
//! - **A failed file operation is reported, not swallowed.** Loading and saving
//!   failures are shown in the status bar and leave the dialogs open, so the path
//!   the user typed is still there to correct.

use std::path::{Path, PathBuf};

use eframe::egui;
use fireeffects_core::{Action, Clip, MediaKind, Panel, Project, Track, TrackKind, Workspace};

use crate::dialogs::{Confirmation, Intent};
use crate::files::{FilePicker, NativePicker};
use crate::library::{ImportReport, Library, reexpress_references};
use crate::{dialogs, keys, menu, panels, project_io, theme};

/// The application state.
pub struct FireEffectsApp {
    workspace: Workspace,
    dialogs: dialogs::Dialogs,
    project: Option<Project>,
    project_path: Option<PathBuf>,
    project_dirty: bool,
    library: Library,
    /// Where the user chooses files. Swapped for a scripted picker in tests.
    picker: Box<dyn FilePicker>,
    /// A command waiting for the unsaved-changes question to be answered.
    pending: Option<Intent>,
    /// The last command that could not be carried out, for the status bar.
    notice: Option<String>,
    /// The title last sent to the window, so it is only set when it changes.
    title: String,
}

impl Default for FireEffectsApp {
    fn default() -> Self {
        Self::with_picker(Box::new(NativePicker))
    }
}

impl FireEffectsApp {
    /// Creates the shell with a specific file picker.
    ///
    /// The window uses [`NativePicker`]; tests use a script so the interface can be
    /// driven without a desktop.
    pub fn with_picker(picker: Box<dyn FilePicker>) -> Self {
        Self {
            workspace: Workspace::new(),
            dialogs: dialogs::Dialogs::default(),
            project: None,
            project_path: None,
            project_dirty: false,
            library: Library::default(),
            picker,
            pending: None,
            notice: None,
            title: String::new(),
        }
    }

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
        self.ask_before_closing(ui.ctx());

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

        let outcome = dialogs::show(ui.ctx(), &mut self.dialogs);
        if let Some(chosen) = outcome.action {
            action = Some(chosen);
        }

        if let Some(action) = action {
            self.apply(action, ui.ctx());
        }

        if let Some(confirmation) = outcome.confirmation {
            self.finish(confirmation, ui.ctx());
        }

        self.sync_title(ui.ctx());
    }

    /// Asks before the window's own close button throws away unsaved work.
    fn ask_before_closing(&mut self, ctx: &egui::Context) {
        let requested = ctx.input(|input| input.viewport().close_requested());

        if requested && self.project_dirty {
            ctx.send_viewport_cmd(egui::ViewportCommand::CancelClose);

            if self.dialogs.confirm.is_none() {
                self.begin(Intent::Quit, ctx);
            }
        }
    }

    /// The four workspace panels, in the order that gives them their regions.
    fn draw_workspace(&self, ui: &mut egui::Ui, action: &mut Option<Action>) {
        let workspace = &self.workspace;
        let project = self.project.as_ref();
        let library = &self.library;

        if workspace.is_visible(Panel::Media) {
            egui::Panel::left("media_panel")
                .resizable(true)
                .default_size(260.0)
                .min_size(180.0)
                .frame(surface(theme::SURFACE))
                .show(ui, |ui| {
                    panels::draw(ui, Panel::Media, workspace, project, library, action)
                });
        }

        if workspace.is_visible(Panel::Inspector) {
            egui::Panel::right("inspector_panel")
                .resizable(true)
                .default_size(280.0)
                .min_size(180.0)
                .frame(surface(theme::SURFACE))
                .show(ui, |ui| {
                    panels::draw(ui, Panel::Inspector, workspace, project, library, action)
                });
        }

        if workspace.is_visible(Panel::Timeline) {
            egui::Panel::bottom("timeline_panel")
                .resizable(true)
                .default_size(240.0)
                .min_size(120.0)
                .frame(surface(theme::SURFACE))
                .show(ui, |ui| {
                    panels::draw(ui, Panel::Timeline, workspace, project, library, action)
                });
        }

        if workspace.is_visible(Panel::Preview) {
            egui::CentralPanel::default()
                .frame(surface(theme::CANVAS))
                .show(ui, |ui| {
                    panels::draw(ui, Panel::Preview, workspace, project, library, action)
                });
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
                let state = if self.project_dirty {
                    " · Unsaved"
                } else {
                    " · Saved"
                };
                ui.label(
                    egui::RichText::new(format!("{}{}", project.name, state))
                        .color(theme::TEXT_DIM),
                );

                let missing = self.library.missing();
                if missing > 0 {
                    ui.label(
                        egui::RichText::new(format!("· {missing} media missing"))
                            .color(theme::WARNING),
                    );
                }
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
            Action::NewProject => self.begin(Intent::NewProject, ctx),
            Action::OpenProject => self.begin(Intent::OpenProject, ctx),
            Action::SaveProject => self.save_current_project(),
            Action::SaveProjectAs => {
                if self.project.is_some() {
                    self.dialogs.save_project_path = self
                        .project_path
                        .as_ref()
                        .map(|path| path.display().to_string())
                        .or_else(|| {
                            self.project
                                .as_ref()
                                .map(|project| format!("{}.firefx", project.name))
                        })
                        .unwrap_or_default();
                    self.dialogs.save_as = true;
                } else {
                    self.notice = Some("No project is open".to_owned());
                }
            }
            Action::CreateProject => self.create_project(),
            Action::LoadProject => self.load_project(),
            Action::SaveProjectToPath => self.save_project_as(),
            Action::CloseProject => self.begin(Intent::CloseProject, ctx),
            Action::ImportMedia => self.import_media(),
            Action::AddTrack(kind) => self.add_track(kind),
            Action::AddSelectedMediaToTimeline => self.add_selected_media_to_timeline(),
            Action::NudgeClip { clip_id, frames } => self.nudge_clip(clip_id, frames),
            Action::ResizeClip { clip_id, frames } => self.resize_clip(clip_id, frames),
            Action::DeleteClip(clip_id) => self.delete_clip(clip_id),
            Action::ToggleTrackMute(track_id) => self.toggle_track_mute(track_id),
            Action::SelectMedia(id) => self.library.select(id),
            Action::RelinkMedia(id) => self.relink_media(id),
            Action::PickProjectFile => {
                if let Some(path) = self.picker.open_project() {
                    self.dialogs.open_project_path = path.display().to_string();
                }
            }
            Action::PickSaveLocation => {
                if let Some(path) = self.picker.save_project(self.project_path.as_deref()) {
                    self.dialogs.save_project_path = path.display().to_string();
                }
            }
            Action::ShowShortcuts => self.dialogs.shortcuts = true,
            Action::ShowAbout => self.dialogs.about = true,
            Action::Quit => self.begin(Intent::Quit, ctx),
        }
    }

    fn add_track(&mut self, kind: TrackKind) {
        let Some(project) = self.project.as_mut() else {
            self.notice = Some("Create or open a project first".to_owned());
            return;
        };
        let id = project
            .tracks
            .iter()
            .map(|track| track.id)
            .max()
            .unwrap_or(0)
            .saturating_add(1);
        let count = project
            .tracks
            .iter()
            .filter(|track| track.kind == kind)
            .count()
            + 1;
        project.tracks.push(Track {
            id,
            name: format!(
                "{} {count}",
                if kind == TrackKind::Video {
                    "Video"
                } else {
                    "Audio"
                }
            ),
            kind,
            muted: false,
            clips: Vec::new(),
        });
        self.project_dirty = true;
        self.notice = Some("Timeline track added".to_owned());
    }

    fn add_selected_media_to_timeline(&mut self) {
        let Some(entry) = self.library.selected().cloned() else {
            self.notice = Some("Select a media item in the Media panel first".to_owned());
            return;
        };
        if !entry.status.is_available() {
            self.notice = Some("Relink this media before adding it to the timeline".to_owned());
            return;
        }
        let Some(project) = self.project.as_mut() else {
            self.notice = Some("Create or open a project first".to_owned());
            return;
        };
        let kind = match entry.kind() {
            MediaKind::Audio => TrackKind::Audio,
            MediaKind::Video | MediaKind::Image => TrackKind::Video,
        };
        if !project.tracks.iter().any(|track| track.kind == kind) {
            let id = project
                .tracks
                .iter()
                .map(|track| track.id)
                .max()
                .unwrap_or(0)
                .saturating_add(1);
            let count = project
                .tracks
                .iter()
                .filter(|track| track.kind == kind)
                .count()
                + 1;
            project.tracks.push(Track {
                id,
                name: format!(
                    "{} {count}",
                    if kind == TrackKind::Video {
                        "Video"
                    } else {
                        "Audio"
                    }
                ),
                kind,
                muted: false,
                clips: Vec::new(),
            });
        }
        let clip_id = project
            .tracks
            .iter()
            .flat_map(|track| &track.clips)
            .map(|clip| clip.id)
            .max()
            .unwrap_or(0)
            .saturating_add(1);
        let track = project
            .tracks
            .iter_mut()
            .find(|track| track.kind == kind)
            .expect("track created");
        let start = track
            .clips
            .iter()
            .map(|clip| clip.start_frame.saturating_add(clip.duration_frames))
            .max()
            .unwrap_or(0);
        let duration = (project.settings.frame_rate_num as u64)
            .saturating_mul(5)
            .max(1);
        let end = start.saturating_add(duration);
        project.settings.duration_frames = project.settings.duration_frames.max(end);
        track.clips.push(Clip {
            id: clip_id,
            media_id: entry.asset.id,
            start_frame: start,
            source_in_frame: 0,
            duration_frames: duration,
        });
        self.project_dirty = true;
        self.notice = Some(format!("{} added to timeline", entry.asset.display_name));
    }

    fn nudge_clip(&mut self, clip_id: u64, frames: i64) {
        let Some(project) = self.project.as_mut() else {
            return;
        };
        let duration = project.settings.duration_frames;
        let Some(clip) = project
            .tracks
            .iter_mut()
            .flat_map(|track| &mut track.clips)
            .find(|clip| clip.id == clip_id)
        else {
            self.notice = Some("Clip no longer exists".to_owned());
            return;
        };
        let proposed = if frames < 0 {
            clip.start_frame.saturating_sub(frames.unsigned_abs())
        } else {
            clip.start_frame.saturating_add(frames as u64)
        };
        clip.start_frame = proposed.min(duration.saturating_sub(clip.duration_frames));
        self.project_dirty = true;
        self.notice = Some("Clip moved".to_owned());
    }

    fn resize_clip(&mut self, clip_id: u64, frames: i64) {
        let Some(project) = self.project.as_mut() else {
            return;
        };
        let project_duration = project.settings.duration_frames;
        let Some(clip) = project
            .tracks
            .iter_mut()
            .flat_map(|track| &mut track.clips)
            .find(|clip| clip.id == clip_id)
        else {
            self.notice = Some("Clip no longer exists".to_owned());
            return;
        };
        let proposed = if frames < 0 {
            clip.duration_frames.saturating_sub(frames.unsigned_abs())
        } else {
            clip.duration_frames.saturating_add(frames as u64)
        }
        .max(1);
        clip.duration_frames =
            proposed.min(project_duration.saturating_sub(clip.start_frame).max(1));
        self.project_dirty = true;
        self.notice = Some("Clip duration changed".to_owned());
    }

    fn delete_clip(&mut self, clip_id: u64) {
        let Some(project) = self.project.as_mut() else {
            return;
        };
        let before: usize = project.tracks.iter().map(|track| track.clips.len()).sum();
        for track in &mut project.tracks {
            track.clips.retain(|clip| clip.id != clip_id);
        }
        let after: usize = project.tracks.iter().map(|track| track.clips.len()).sum();
        if before != after {
            self.project_dirty = true;
            self.notice = Some("Clip deleted".to_owned());
        }
    }

    fn toggle_track_mute(&mut self, track_id: u64) {
        let Some(project) = self.project.as_mut() else {
            return;
        };
        if let Some(track) = project.tracks.iter_mut().find(|track| track.id == track_id) {
            track.muted = !track.muted;
            self.project_dirty = true;
            self.notice = Some(
                if track.muted {
                    "Track muted"
                } else {
                    "Track unmuted"
                }
                .to_owned(),
            );
        }
    }

    /// Runs `intent`, or asks about unsaved changes first.
    fn begin(&mut self, intent: Intent, ctx: &egui::Context) {
        if self.project_dirty && self.project.is_some() {
            self.dialogs.confirm = Some(intent);
            self.pending = Some(intent);
        } else {
            self.run(intent, ctx);
        }
    }

    /// Carries out an intent once any question about it has been answered.
    fn run(&mut self, intent: Intent, ctx: &egui::Context) {
        match intent {
            Intent::NewProject => {
                self.dialogs.new_project_name = "Untitled".to_owned();
                self.dialogs.new_project = true;
            }
            Intent::OpenProject => self.dialogs.open_project = true,
            Intent::CloseProject => {
                self.project = None;
                self.project_path = None;
                self.project_dirty = false;
                self.library.clear();
                self.notice = Some("Project closed".to_owned());
            }
            Intent::Quit => ctx.send_viewport_cmd(egui::ViewportCommand::Close),
        }
    }

    /// Acts on the answer to the unsaved-changes question.
    fn finish(&mut self, confirmation: Confirmation, ctx: &egui::Context) {
        match confirmation {
            Confirmation::Save => {
                let saved = self.save_to_known_path();

                if saved {
                    if let Some(intent) = self.pending.take() {
                        self.run(intent, ctx);
                    }
                } else {
                    // The save failed, or there is nowhere to save yet, so the
                    // request is dropped rather than repeated: the project is still
                    // exactly as it was.
                    self.pending = None;
                }
            }
            Confirmation::Discard => {
                if let Some(intent) = self.pending.take() {
                    self.run(intent, ctx);
                }
            }
            Confirmation::Cancel => self.pending = None,
        }
    }

    fn create_project(&mut self) {
        let name = self.dialogs.new_project_name.trim();
        if name.is_empty() {
            self.notice = Some("Enter a project name".to_owned());
            return;
        }

        self.project = Some(Project::new(name));
        self.project_path = None;
        self.project_dirty = true;
        self.dialogs.new_project = false;
        self.library.clear();
        self.notice = Some("New project created; save it to keep your work".to_owned());
    }

    fn load_project(&mut self) {
        let path = PathBuf::from(self.dialogs.open_project_path.trim());
        match project_io::read_project(&path) {
            Ok(project) => {
                self.library.reload(&project, path.parent());
                self.project = Some(project);
                self.project_path = Some(path);
                self.project_dirty = false;
                self.dialogs.open_project = false;
                self.notice = Some("Project opened".to_owned());
            }
            Err(error) => self.notice = Some(sentence(error)),
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

        // Saving over the project's own file is the normal case, so replacing it
        // is intended rather than accidental.
        self.write_project_to(&path, true);
    }

    fn save_project_as(&mut self) {
        if self.project.is_none() {
            self.notice = Some("No project is open".to_owned());
            return;
        }

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

        // Only the project's own file may be replaced without asking again.
        let overwrite = self.project_path.as_ref() == Some(&path);
        if self.write_project_to(&path, overwrite) {
            self.dialogs.save_as = false;
            self.project_path = Some(path);
        }
    }

    /// Saves to the project's own file, for the unsaved-changes question.
    ///
    /// A project that has never been saved has nowhere to go, so the Save As dialog
    /// opens instead and the request is not repeated: the user can run the command
    /// again once the project has a home.
    fn save_to_known_path(&mut self) -> bool {
        let Some(path) = self.project_path.clone() else {
            if let Some(project) = &self.project {
                self.dialogs.save_project_path = format!("{}.firefx", project.name);
                self.dialogs.save_as = true;
                self.notice = Some("Choose where to save the project first".to_owned());
            }
            return false;
        };

        self.write_project_to(&path, true)
    }

    /// Writes the project to `path`, re-expressing media references for its
    /// directory. Returns whether the file was written.
    fn write_project_to(&mut self, path: &Path, overwrite: bool) -> bool {
        let Some(project) = self.project.as_ref() else {
            self.notice = Some("No project is open".to_owned());
            return false;
        };

        // Paths are rewritten on a copy: if the write fails, the open project must
        // not be left describing a location it was never saved to.
        let mut written = project.clone();
        reexpress_references(&mut written, self.project_dir().as_deref(), path.parent());

        match project_io::write_project(path, &written, overwrite) {
            Ok(()) => {
                self.library.reload(&written, path.parent());
                self.project = Some(written);
                self.project_dirty = false;
                self.notice = Some("Project saved".to_owned());
                true
            }
            Err(error) => {
                self.notice = Some(sentence(error));
                false
            }
        }
    }

    fn import_media(&mut self) {
        if self.project.is_none() {
            self.notice = Some("No project is open".to_owned());
            return;
        }

        let paths = self.picker.import_media();
        if paths.is_empty() {
            return;
        }

        let directory = self.project_dir();
        let Some(project) = self.project.as_mut() else {
            return;
        };

        let report = self.library.import(project, directory.as_deref(), &paths);
        if report.added > 0 {
            self.project_dirty = true;
        }
        self.notice = Some(import_notice(&report));
    }

    fn relink_media(&mut self, id: u64) {
        let Some(name) = self
            .library
            .entries()
            .iter()
            .find(|entry| entry.asset.id == id)
            .map(|entry| entry.asset.display_name.clone())
        else {
            return;
        };

        let Some(path) = self.picker.relink_media(&name) else {
            return;
        };

        let directory = self.project_dir();
        let Some(project) = self.project.as_mut() else {
            return;
        };

        if self
            .library
            .relink(project, directory.as_deref(), id, &path)
        {
            self.project_dirty = true;
            self.notice = Some(format!("Relinked {name}"));
        }
    }

    /// The directory media references resolve against: the project file's own.
    fn project_dir(&self) -> Option<PathBuf> {
        self.project_path
            .as_ref()
            .and_then(|path| path.parent())
            .map(Path::to_path_buf)
    }

    /// Keeps the window title showing what is open and whether it is saved.
    fn sync_title(&mut self, ctx: &egui::Context) {
        let title = match &self.project {
            Some(project) if self.project_dirty => {
                format!("{} (unsaved) — FireEffects", project.name)
            }
            Some(project) => format!("{} — FireEffects", project.name),
            None => "FireEffects".to_owned(),
        };

        if title != self.title {
            self.title = title.clone();
            ctx.send_viewport_cmd(egui::ViewportCommand::Title(title));
        }
    }
}

impl eframe::App for FireEffectsApp {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        self.draw(ui);
    }
}

/// What to say after an import: what was added, what was already there, and what
/// was refused.
fn import_notice(report: &ImportReport) -> String {
    let mut parts = Vec::new();

    if report.added > 0 {
        parts.push(format!(
            "Imported {} media {}",
            report.added,
            if report.added == 1 { "file" } else { "files" }
        ));
    }

    if report.duplicates > 0 {
        parts.push(format!(
            "{} already in the project",
            if report.duplicates == 1 {
                "1 file was".to_owned()
            } else {
                format!("{} files were", report.duplicates)
            }
        ));
    }

    for (name, reason) in &report.rejected {
        parts.push(format!("{name} was skipped ({reason})"));
    }

    if parts.is_empty() {
        "Nothing was imported.".to_owned()
    } else {
        sentence(parts.join(". "))
    }
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
    use crate::files::scripted::ScriptedPicker;
    use crate::library::MediaStatus;
    use egui_kittest::Harness;
    use egui_kittest::kittest::Queryable;
    use std::fs;

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
    fn harness_with(picker: ScriptedPicker) -> Harness<'static, FireEffectsApp> {
        Harness::builder()
            .with_size(egui::Vec2::new(1280.0, 800.0))
            .with_os(egui::os::OperatingSystem::Nix)
            .build_ui_state(
                |ui: &mut egui::Ui, app: &mut FireEffectsApp| app.draw(ui),
                FireEffectsApp::with_picker(Box::new(picker)),
            )
    }

    fn harness() -> Harness<'static, FireEffectsApp> {
        harness_with(ScriptedPicker::default())
    }

    /// Writes a file that stands in for imported media.
    fn write_media(directory: &Path, name: &str) -> PathBuf {
        let path = directory.join(name);
        fs::write(&path, b"stand-in").expect("wrote fixture");
        path
    }

    /// A project that already references one media file, for open-dialog tests.
    fn project_with_media(name: &str, media_path: &str) -> Project {
        let mut project = Project::new(name);
        project.media.push(fireeffects_core::MediaAsset {
            id: 1,
            path: media_path.to_owned(),
            display_name: media_path.to_owned(),
            kind: fireeffects_core::MediaKind::Video,
        });
        project
    }

    /// Starts a project through the real dialog: File → New Project, type a name,
    /// click Create project.
    fn create_project(harness: &mut Harness<'static, FireEffectsApp>, name: &str) {
        harness.get_by_label("File").click();
        harness.run();
        harness.get_by_label_contains("New Project").click();
        harness.run();

        harness.state_mut().dialogs.new_project_name = name.to_owned();
        harness.run();
        harness.get_by_label("Create project").click();
        harness.run();
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

    #[test]
    fn a_project_can_be_created_from_the_new_project_dialog() {
        let mut harness = harness();
        harness.run();

        create_project(&mut harness, "Reel");

        let state = harness.state();
        assert_eq!(state.project.as_ref().expect("open project").name, "Reel");
        assert!(state.project_path.is_none());
        assert!(state.project_dirty, "a new project has unsaved work");
        assert!(harness.query_by_label("No media yet").is_some());
    }

    #[test]
    fn importing_media_adds_it_to_the_project_panel() {
        let directory = tempfile::tempdir().expect("temp dir");
        let video = write_media(directory.path(), "clip.mp4");
        let notes = write_media(directory.path(), "notes.txt");

        let mut harness = harness_with(ScriptedPicker {
            choose_media: std::cell::RefCell::new(vec![video, notes]),
            ..ScriptedPicker::default()
        });

        harness.run();
        create_project(&mut harness, "Reel");

        harness.get_by_label("Import media…").click();
        harness.run();

        let state = harness.state();
        assert_eq!(state.library.entries().len(), 1);
        assert_eq!(state.library.entries()[0].asset.display_name, "clip.mp4");
        assert!(state.project_dirty);
        assert!(harness.query_by_label("clip.mp4").is_some());

        let notice = state.notice.clone().expect("a notice");
        assert!(notice.contains("Imported 1 media file"), "{notice}");
        assert!(notice.contains("notes.txt was skipped"), "{notice}");
        assert!(notice.ends_with('.'), "{notice}");
        assert!(!notice.contains(".."), "one full stop, not two: {notice}");
    }

    #[test]
    fn saving_from_the_save_as_dialog_writes_a_project_that_reopens() {
        let directory = tempfile::tempdir().expect("temp dir");
        let video = write_media(directory.path(), "clip.mp4");
        let project_path = directory.path().join("reel.firefx");

        let mut harness = harness_with(ScriptedPicker {
            choose_media: std::cell::RefCell::new(vec![video]),
            ..ScriptedPicker::default()
        });

        harness.run();
        create_project(&mut harness, "Reel");
        harness.get_by_label("Import media…").click();
        harness.run();

        // Save with nowhere to save yet opens the Save As dialog.
        harness.key_press_modifiers(ctrl(), egui::Key::S);
        harness.run();
        assert!(harness.state().dialogs.save_as);

        harness.state_mut().dialogs.save_project_path = project_path.display().to_string();
        harness.run();
        harness.get_by_label("Save project").click();
        harness.run();

        assert!(!harness.state().project_dirty);
        assert_eq!(
            harness.state().project_path.as_deref(),
            Some(project_path.as_path())
        );

        let written = project_io::read_project(&project_path).expect("readable project");
        assert_eq!(written.media.len(), 1);
        assert_eq!(
            written.media[0].path, "clip.mp4",
            "media should be stored relative to the project file"
        );
    }

    #[test]
    fn opening_a_project_restores_its_media() {
        let directory = tempfile::tempdir().expect("temp dir");
        let video = write_media(directory.path(), "clip.mp4");
        let project_path = directory.path().join("saved.firefx");
        let project = project_with_media("Saved", "clip.mp4");
        project_io::write_project(&project_path, &project, false).expect("fixture project");

        let mut harness = harness();
        harness.run();

        harness.key_press_modifiers(ctrl(), egui::Key::O);
        harness.run();
        harness.state_mut().dialogs.open_project_path = project_path.display().to_string();
        harness.run();
        harness.get_by_label("Open").click();
        harness.run();

        let state = harness.state();
        assert_eq!(state.project.as_ref().expect("open").name, "Saved");
        assert_eq!(state.library.entries().len(), 1);
        assert_eq!(state.library.entries()[0].path, video);
        assert!(!state.project_dirty);
        assert_eq!(state.library.missing(), 0);
    }

    #[test]
    fn the_browse_button_fills_the_open_dialog_from_the_picker() {
        let directory = tempfile::tempdir().expect("temp dir");
        let project_path = directory.path().join("chosen.firefx");

        let mut harness = harness_with(ScriptedPicker {
            choose_open: std::cell::RefCell::new(Some(project_path.clone())),
            ..ScriptedPicker::default()
        });

        harness.run();
        harness.key_press_modifiers(ctrl(), egui::Key::O);
        harness.run();
        harness.get_by_label("Browse…").click();
        harness.run();

        assert_eq!(
            harness.state().dialogs.open_project_path,
            project_path.display().to_string()
        );
    }

    #[test]
    fn opening_a_file_that_is_not_a_project_explains_why() {
        let directory = tempfile::tempdir().expect("temp dir");
        let path = directory.path().join("notes.firefx");
        fs::write(&path, "this is not a project").expect("wrote");

        let mut harness = harness();
        harness.run();
        harness.key_press_modifiers(ctrl(), egui::Key::O);
        harness.run();
        harness.state_mut().dialogs.open_project_path = path.display().to_string();
        harness.run();
        harness.get_by_label("Open").click();
        harness.run();

        let notice = harness.state().notice.clone().expect("a notice");
        assert!(
            notice.to_lowercase().contains("invalid project document"),
            "unexpected notice: {notice}"
        );
        assert!(harness.state().project.is_none());
        assert!(
            harness.state().dialogs.open_project,
            "the dialog should stay open so the path can be corrected"
        );
    }

    #[test]
    fn closing_with_unsaved_changes_asks_first() {
        let mut harness = harness();
        harness.run();
        create_project(&mut harness, "Reel");
        assert!(harness.state().project_dirty);

        harness.key_press_modifiers(ctrl(), egui::Key::W);
        harness.run();

        assert!(
            harness.query_by_label("Unsaved changes").is_some(),
            "closing should ask about unsaved changes"
        );
        assert!(harness.state().project.is_some());

        harness.get_by_label("Cancel").click();
        harness.run();

        assert!(harness.state().project.is_some());
        assert!(harness.query_by_label("Unsaved changes").is_none());
        assert!(harness.state().project_dirty, "cancelling changes nothing");
    }

    #[test]
    fn discarding_unsaved_changes_closes_the_project() {
        let mut harness = harness();
        harness.run();
        create_project(&mut harness, "Reel");

        harness.key_press_modifiers(ctrl(), egui::Key::W);
        harness.run();
        harness.get_by_label("Discard changes").click();
        harness.run();

        assert!(harness.state().project.is_none());
        assert!(harness.state().library.entries().is_empty());
    }

    #[test]
    fn saving_from_the_question_writes_the_file_and_continues() {
        let directory = tempfile::tempdir().expect("temp dir");
        let video = write_media(directory.path(), "clip.mp4");
        let project_path = directory.path().join("reel.firefx");

        let mut harness = harness_with(ScriptedPicker {
            choose_media: std::cell::RefCell::new(vec![video]),
            ..ScriptedPicker::default()
        });

        harness.run();
        create_project(&mut harness, "Reel");

        // Give the project a home first, so the question has somewhere to save.
        harness.state_mut().dialogs.save_project_path = project_path.display().to_string();
        harness.state_mut().dialogs.save_as = true;
        harness.run();
        harness.get_by_label("Save project").click();
        harness.run();
        assert!(!harness.state().project_dirty);

        harness.get_by_label("Import media…").click();
        harness.run();
        assert!(harness.state().project_dirty);

        harness.key_press_modifiers(ctrl(), egui::Key::W);
        harness.run();
        harness.get_by_label("Save").click();
        harness.run();

        assert!(
            harness.state().project.is_none(),
            "the close should go ahead"
        );
        let written = project_io::read_project(&project_path).expect("readable project");
        assert_eq!(written.media.len(), 1, "the import should have been saved");
    }

    #[test]
    fn a_missing_media_file_is_flagged_and_can_be_relinked() {
        let directory = tempfile::tempdir().expect("temp dir");
        let project_path = directory.path().join("reel.firefx");
        project_io::write_project(
            &project_path,
            &project_with_media("Reel", "gone.mp4"),
            false,
        )
        .expect("fixture project");
        let replacement = write_media(directory.path(), "restored.mp4");

        let mut harness = harness_with(ScriptedPicker {
            choose_relink: std::cell::RefCell::new(Some(replacement)),
            ..ScriptedPicker::default()
        });

        harness.run();
        harness.state_mut().dialogs.open_project_path = project_path.display().to_string();
        harness.state_mut().dialogs.open_project = true;
        harness.run();
        harness.get_by_label("Open").click();
        harness.run();

        assert_eq!(harness.state().library.missing(), 1);
        assert!(harness.query_by_label("Locate…").is_some());

        harness.get_by_label("Locate…").click();
        harness.run();

        let state = harness.state();
        assert_eq!(state.library.missing(), 0);
        assert_eq!(
            state.library.entries()[0].status,
            MediaStatus::Available {
                size: "stand-in".len() as u64
            }
        );
        assert!(state.project_dirty, "relinking is a change to the project");
        assert_eq!(
            state.library.entries()[0].asset.display_name,
            "restored.mp4"
        );
    }

    #[test]
    fn import_notices_count_files_and_name_what_was_skipped() {
        let report = ImportReport {
            added: 2,
            duplicates: 1,
            rejected: vec![("notes.txt".to_owned(), "unsupported format")],
        };

        let notice = import_notice(&report);

        assert!(notice.contains("Imported 2 media files"), "{notice}");
        assert!(
            notice.contains("1 file was already in the project"),
            "{notice}"
        );
        assert!(
            notice.contains("notes.txt was skipped (unsupported format)"),
            "{notice}"
        );
        assert!(notice.ends_with('.'), "{notice}");
    }
}
