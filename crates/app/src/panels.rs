//! The four workspace panels.
//!
//! No panel draws a control that does nothing. While a panel has no features it
//! says which stage brings them, so an empty region is never mistaken for a broken
//! one. Panels hold no state: they read the workspace, the project, and the media
//! library, and they report the one command a click can produce.

use eframe::egui;
use fireeffects_core::{Action, MediaKind, Panel, Project, TrackKind, Workspace};

use crate::library::{Library, MediaEntry, MediaStatus};
use crate::theme;

/// Title of the panel, used in its header and in the View menu.
pub fn title(panel: Panel) -> &'static str {
    match panel {
        Panel::Media => "Media",
        Panel::Preview => "Preview",
        Panel::Timeline => "Timeline",
        Panel::Inspector => "Inspector",
    }
}

/// How a media kind is described in the interface.
pub fn kind_label(kind: MediaKind) -> &'static str {
    match kind {
        MediaKind::Video => "Video",
        MediaKind::Audio => "Audio",
        MediaKind::Image => "Image",
    }
}

/// What a panel shows when it has nothing to show.
fn empty_state(panel: Panel) -> [&'static str; 2] {
    match panel {
        Panel::Media => [
            "No project open",
            "Create or open a project from the File menu.",
        ],
        Panel::Preview => [
            "Nothing to preview",
            "Preview and playback arrive in Stage 4, once media can be decoded.",
        ],
        Panel::Timeline => [
            "No timeline yet",
            "Add tracks and selected media, then move, trim, or delete clips.",
        ],
        Panel::Inspector => [
            "Nothing selected",
            "Select a media item to see its details.",
        ],
    }
}

/// Draws one panel: its header and its contents.
///
/// Clicking the header focuses the panel, which is what the status bar reports and
/// what keyboard commands act on.
pub fn draw(
    ui: &mut egui::Ui,
    panel: Panel,
    workspace: &Workspace,
    project: Option<&Project>,
    library: &Library,
    action: &mut Option<Action>,
) {
    let focused = workspace.focused() == panel;
    let heading = egui::RichText::new(title(panel)).strong();
    let heading = if focused {
        heading.color(theme::EMBER)
    } else {
        heading.color(theme::TEXT)
    };

    if ui.selectable_label(focused, heading).clicked() {
        *action = Some(Action::ShowPanel(panel));
    }

    ui.add_space(2.0);
    ui.separator();
    ui.add_space(6.0);

    match panel {
        Panel::Media => media(ui, project, library, action),
        Panel::Inspector => inspector(ui, project, library, action),
        Panel::Preview => {
            let [headline, detail] = empty_state(panel);
            ui.label(egui::RichText::new(headline).color(theme::TEXT));
            ui.label(egui::RichText::new(detail).color(theme::TEXT_DIM));
        }
        Panel::Timeline => timeline(ui, project, library, action),
    }

    // A resizable panel only follows its handle if something claims the remaining
    // space, so the body claims it.
    ui.take_available_space();
}

/// The first usable timeline editor: tracks, clip placement, move, trim and delete.
fn timeline(
    ui: &mut egui::Ui,
    project: Option<&Project>,
    library: &Library,
    action: &mut Option<Action>,
) {
    let Some(project) = project else {
        ui.label(egui::RichText::new("Open a project to edit its timeline").color(theme::TEXT_DIM));
        return;
    };

    ui.horizontal_wrapped(|ui| {
        if ui.button("+ Video track").clicked() {
            *action = Some(Action::AddTrack(TrackKind::Video));
        }
        if ui.button("+ Audio track").clicked() {
            *action = Some(Action::AddTrack(TrackKind::Audio));
        }
        let selected = library.selected();
        let can_add = selected.is_some_and(|entry| entry.status.is_available());
        if ui.add_enabled(can_add, egui::Button::new("Add selected media")).clicked() {
            *action = Some(Action::AddSelectedMediaToTimeline);
        }
    });

    ui.label(
        egui::RichText::new(format!(
            "{} frames · {}/{} fps",
            project.settings.duration_frames,
            project.settings.frame_rate_num,
            project.settings.frame_rate_den
        ))
        .color(theme::TEXT_DIM)
        .small(),
    );
    ui.separator();

    if project.tracks.is_empty() {
        ui.label("No tracks yet. Add a video or audio track above.");
        return;
    }

    egui::ScrollArea::vertical().show(ui, |ui| {
        for track in &project.tracks {
            ui.group(|ui| {
                ui.horizontal(|ui| {
                    ui.label(egui::RichText::new(&track.name).strong());
                    ui.label(match track.kind {
                        TrackKind::Video => "VIDEO",
                        TrackKind::Audio => "AUDIO",
                    });
                    if ui.small_button(if track.muted { "Unmute" } else { "Mute" }).clicked() {
                        *action = Some(Action::ToggleTrackMute(track.id));
                    }
                });
                if track.clips.is_empty() {
                    ui.label(egui::RichText::new("Empty track").color(theme::TEXT_DIM).small());
                }
                for clip in &track.clips {
                    let media_name = project.media.iter()
                        .find(|asset| asset.id == clip.media_id)
                        .map(|asset| asset.display_name.as_str())
                        .unwrap_or("Missing media");
                    ui.horizontal_wrapped(|ui| {
                        ui.label(format!(
                            "{} · {}f → {}f",
                            media_name,
                            clip.start_frame,
                            clip.start_frame.saturating_add(clip.duration_frames)
                        ));
                        if ui.small_button("← 1s").clicked() {
                            *action = Some(Action::NudgeClip {
                                clip_id: clip.id,
                                frames: -(project.settings.frame_rate_num as i64),
                            });
                        }
                        if ui.small_button("→ 1s").clicked() {
                            *action = Some(Action::NudgeClip {
                                clip_id: clip.id,
                                frames: project.settings.frame_rate_num as i64,
                            });
                        }
                        if ui.small_button("− 1s").clicked() {
                            *action = Some(Action::ResizeClip {
                                clip_id: clip.id,
                                frames: -(project.settings.frame_rate_num as i64),
                            });
                        }
                        if ui.small_button("+ 1s").clicked() {
                            *action = Some(Action::ResizeClip {
                                clip_id: clip.id,
                                frames: project.settings.frame_rate_num as i64,
                            });
                        }
                        if ui.small_button("Delete").clicked() {
                            *action = Some(Action::DeleteClip(clip.id));
                        }
                    });
                }
            });
            ui.add_space(4.0);
        }
    });
}

/// The project panel: the project's format, its media, and how to add more.
fn media(
    ui: &mut egui::Ui,
    project: Option<&Project>,
    library: &Library,
    action: &mut Option<Action>,
) {
    let Some(project) = project else {
        let [headline, detail] = empty_state(Panel::Media);
        ui.label(egui::RichText::new(headline).color(theme::TEXT));
        ui.label(egui::RichText::new(detail).color(theme::TEXT_DIM));
        return;
    };

    ui.label(
        egui::RichText::new(&project.name)
            .strong()
            .color(theme::TEXT),
    );
    ui.label(
        egui::RichText::new(format!(
            "{} × {} · {}/{} fps",
            project.settings.width,
            project.settings.height,
            project.settings.frame_rate_num,
            project.settings.frame_rate_den
        ))
        .color(theme::TEXT_DIM),
    );

    ui.add_space(8.0);
    if ui.button("Import media…").clicked() {
        *action = Some(Action::ImportMedia);
    }

    let entries = library.entries();
    if entries.is_empty() {
        ui.add_space(6.0);
        ui.label(egui::RichText::new("No media yet").color(theme::TEXT));
        ui.label(
            egui::RichText::new("Imported video, audio, and images are listed here.")
                .color(theme::TEXT_DIM),
        );
        return;
    }

    let missing = library.missing();
    ui.add_space(6.0);
    let summary = match missing {
        0 => format!("{} media items", entries.len()),
        count => format!("{} media items, {count} missing", entries.len()),
    };
    ui.label(egui::RichText::new(summary).color(theme::TEXT_DIM));

    ui.add_space(4.0);
    egui::ScrollArea::vertical().show(ui, |ui| {
        for entry in entries {
            media_row(ui, entry, library, action);
            ui.add_space(2.0);
        }
    });
}

fn media_row(
    ui: &mut egui::Ui,
    entry: &MediaEntry,
    library: &Library,
    action: &mut Option<Action>,
) {
    let selected = library
        .selected()
        .is_some_and(|selected| selected.asset.id == entry.asset.id);

    ui.horizontal(|ui| {
        let name = egui::RichText::new(&entry.asset.display_name);
        let name = if entry.status.is_available() {
            name
        } else {
            name.color(theme::WARNING)
        };

        if ui.selectable_label(selected, name).clicked() {
            *action = Some(Action::SelectMedia(entry.asset.id));
        }

        if !entry.status.is_available() && ui.small_button("Locate…").clicked() {
            *action = Some(Action::RelinkMedia(entry.asset.id));
        }
    });

    ui.label(
        egui::RichText::new(format!(
            "{} · {}",
            kind_label(entry.kind()),
            entry.status.summary()
        ))
        .color(theme::TEXT_DIM)
        .small(),
    );
}

/// The inspector: details of whatever is selected.
fn inspector(
    ui: &mut egui::Ui,
    project: Option<&Project>,
    library: &Library,
    action: &mut Option<Action>,
) {
    if project.is_none() {
        let [headline, detail] = empty_state(Panel::Inspector);
        ui.label(egui::RichText::new(headline).color(theme::TEXT));
        ui.label(egui::RichText::new(detail).color(theme::TEXT_DIM));
        return;
    }

    let Some(entry) = library.selected() else {
        let [headline, detail] = empty_state(Panel::Inspector);
        ui.label(egui::RichText::new(headline).color(theme::TEXT));
        ui.label(egui::RichText::new(detail).color(theme::TEXT_DIM));
        return;
    };

    ui.label(
        egui::RichText::new(&entry.asset.display_name)
            .strong()
            .color(theme::TEXT),
    );
    ui.add_space(4.0);

    row(ui, "Type", kind_label(entry.kind()));
    row(ui, "Status", &entry.status.summary());
    row(ui, "Reference", &entry.asset.path);
    row(ui, "Location", &entry.path.display().to_string());

    if let MediaStatus::Available { size } = &entry.status {
        row(ui, "Size", &format!("{size} bytes"));
    }

    if !entry.status.is_available() {
        ui.add_space(6.0);
        ui.label(
            egui::RichText::new(
                "This file is not where the project expects it. Locate it to relink \
                 without losing the project's reference.",
            )
            .color(theme::WARNING),
        );
        if ui.button("Locate…").clicked() {
            *action = Some(Action::RelinkMedia(entry.asset.id));
        }
    }
}

/// One labelled line of details.
fn row(ui: &mut egui::Ui, label: &str, value: &str) {
    ui.horizontal_wrapped(|ui| {
        ui.label(egui::RichText::new(label).color(theme::TEXT_DIM));
        ui.label(egui::RichText::new(value).color(theme::TEXT));
    });
}
