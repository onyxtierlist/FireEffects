//! The four workspace panels.
//!
//! No panel draws a control that does nothing. While a panel has no features it
//! says which stage brings them, so an empty region is never mistaken for a broken
//! one. Nothing here holds state: the panels are a view of `fireeffects_core`'s
//! [`Workspace`] and they report the one command a click can produce.

use eframe::egui;
use fireeffects_core::{Action, Panel, Project, Workspace};

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

/// What the panel shows today: a headline, and which stage fills it in.
fn empty_state(panel: Panel) -> [&'static str; 2] {
    match panel {
        Panel::Media => [
            "No project open",
            "Project files, media import, and metadata arrive in Stage 2.",
        ],
        Panel::Preview => [
            "Nothing to preview",
            "Preview and playback arrive in Stage 4, once media can be imported and edited.",
        ],
        Panel::Timeline => [
            "No timeline yet",
            "Tracks, clips, and the playhead arrive in Stage 3.",
        ],
        Panel::Inspector => [
            "Nothing selected",
            "Properties appear once clips and layers exist, in Stages 3 to 5.",
        ],
    }
}

/// Draws one panel: its header and what it has to show.
///
/// Clicking the header focuses the panel, which is what the status bar reports and
/// what keyboard commands act on.
pub fn draw(
    ui: &mut egui::Ui,
    panel: Panel,
    workspace: &Workspace,
    project: Option<&Project>,
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

    if panel == Panel::Media {
        if let Some(project) = project {
            ui.label(egui::RichText::new(&project.name).strong().color(theme::TEXT));
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
            ui.label(format!("{} media items", project.media.len()));
            ui.add_space(8.0);
            ui.label(
                egui::RichText::new("Media import arrives in the next Stage 2 step.")
                    .color(theme::TEXT_DIM),
            );
        } else {
            let [headline, detail] = empty_state(panel);
            ui.label(egui::RichText::new(headline).color(theme::TEXT));
            ui.label(egui::RichText::new(detail).color(theme::TEXT_DIM));
        }
    } else {
        let [headline, detail] = empty_state(panel);
        ui.label(egui::RichText::new(headline).color(theme::TEXT));
        ui.label(egui::RichText::new(detail).color(theme::TEXT_DIM));
    }

    // A resizable panel only follows its handle if something claims the remaining
    // space, so the empty body claims it.
    ui.take_available_space();
}
