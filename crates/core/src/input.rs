//! The shell's command model: keys, key chords, and the commands they run.
//!
//! Keeping this table out of the UI code means the shortcuts window, the menus,
//! and the key handling all read the same source of truth, and that conflicts can
//! be caught by tests instead of by a user.

use std::fmt;

use crate::{Panel, TrackKind};

/// Keyboard modifiers, described without reference to any UI toolkit.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct Modifiers {
    /// Control on Windows and Linux; the same physical key group as Command on macOS.
    pub ctrl: bool,
    /// Shift.
    pub shift: bool,
    /// Alt, or Option on macOS.
    pub alt: bool,
}

impl Modifiers {
    /// No modifier held.
    pub const NONE: Self = Self {
        ctrl: false,
        shift: false,
        alt: false,
    };
    /// Control or Command.
    pub const CTRL: Self = Self {
        ctrl: true,
        shift: false,
        alt: false,
    };
    /// Shift alone.
    pub const SHIFT: Self = Self {
        ctrl: false,
        shift: true,
        alt: false,
    };
}

/// A key the shell responds to.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Key {
    /// The `1` key.
    Num1,
    /// The `2` key.
    Num2,
    /// The `3` key.
    Num3,
    /// The `4` key.
    Num4,
    /// The `Q` key.
    Q,
    /// The `F1` key.
    F1,
    /// The `F6` key.
    F6,
    /// The `N` key.
    N,
    /// The `O` key.
    O,
    /// The `S` key.
    S,
    /// The `W` key.
    W,
    /// The `I` key.
    I,
}

impl Key {
    /// The character or name printed on the key.
    pub const fn label(self) -> &'static str {
        match self {
            Self::Num1 => "1",
            Self::Num2 => "2",
            Self::Num3 => "3",
            Self::Num4 => "4",
            Self::Q => "Q",
            Self::F1 => "F1",
            Self::F6 => "F6",
            Self::N => "N",
            Self::O => "O",
            Self::S => "S",
            Self::W => "W",
            Self::I => "I",
        }
    }
}

/// Something the editor shell can do.
///
/// Every variant changes real state today. A command that cannot work yet is not
/// listed here, so nothing in the interface can silently do nothing.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Action {
    /// Hides the panel, or shows it again if it was hidden.
    TogglePanel(Panel),
    /// Shows the panel, if needed, and gives it focus.
    ShowPanel(Panel),
    /// Gives focus to the next visible panel.
    FocusNext,
    /// Gives focus to the previous visible panel.
    FocusPrevious,
    /// Opens the new-project dialog.
    NewProject,
    /// Opens the project-file dialog.
    OpenProject,
    /// Saves the current project to its existing path.
    SaveProject,
    /// Opens the Save As dialog.
    SaveProjectAs,
    /// Creates a project from the new-project dialog.
    CreateProject,
    /// Opens a project from the open-project dialog.
    LoadProject,
    /// Saves the current project to the path in the Save As dialog.
    SaveProjectToPath,
    /// Closes the project, returning to an empty shell.
    CloseProject,
    /// Asks for media files and adds the supported ones to the project.
    ImportMedia,
    /// Creates a timeline track.
    AddTrack(TrackKind),
    /// Adds the selected media item to a compatible timeline track.
    AddSelectedMediaToTimeline,
    /// Moves a clip by a signed number of frames.
    NudgeClip { clip_id: u64, frames: i64 },
    /// Changes a clip duration by a signed number of frames.
    ResizeClip { clip_id: u64, frames: i64 },
    /// Removes a clip from the timeline.
    DeleteClip(u64),
    /// Toggles a track's mute state.
    ToggleTrackMute(u64),
    /// Moves the timeline playhead to a frame.
    SetPlayhead(u64),
    /// Selects a media item in the project panel.
    SelectMedia(u64),
    /// Asks for a new location for a media file the project cannot find.
    RelinkMedia(u64),
    /// Fills the open-project dialog from a native file picker.
    PickProjectFile,
    /// Fills the Save As dialog from a native file picker.
    PickSaveLocation,
    /// Opens the keyboard shortcuts window.
    ShowShortcuts,
    /// Opens the About window.
    ShowAbout,
    /// Closes the application.
    Quit,
}

/// A key chord bound to an [`Action`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Binding {
    /// Modifiers that must be held.
    pub modifiers: Modifiers,
    /// Key that must be pressed.
    pub key: Key,
    /// Command the chord runs.
    pub action: Action,
}

impl fmt::Display for Binding {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.modifiers.ctrl {
            f.write_str("Ctrl+")?;
        }
        if self.modifiers.shift {
            f.write_str("Shift+")?;
        }
        if self.modifiers.alt {
            f.write_str("Alt+")?;
        }
        f.write_str(self.key.label())
    }
}

/// Every keyboard shortcut the shell implements.
pub const BINDINGS: &[Binding] = &[
    Binding {
        modifiers: Modifiers::CTRL,
        key: Key::Num1,
        action: Action::ShowPanel(Panel::Media),
    },
    Binding {
        modifiers: Modifiers::CTRL,
        key: Key::Num2,
        action: Action::ShowPanel(Panel::Preview),
    },
    Binding {
        modifiers: Modifiers::CTRL,
        key: Key::Num3,
        action: Action::ShowPanel(Panel::Timeline),
    },
    Binding {
        modifiers: Modifiers::CTRL,
        key: Key::Num4,
        action: Action::ShowPanel(Panel::Inspector),
    },
    Binding {
        modifiers: Modifiers::NONE,
        key: Key::F6,
        action: Action::FocusNext,
    },
    Binding {
        modifiers: Modifiers::SHIFT,
        key: Key::F6,
        action: Action::FocusPrevious,
    },
    Binding {
        modifiers: Modifiers::CTRL,
        key: Key::N,
        action: Action::NewProject,
    },
    Binding {
        modifiers: Modifiers::CTRL,
        key: Key::O,
        action: Action::OpenProject,
    },
    Binding {
        modifiers: Modifiers::CTRL,
        key: Key::S,
        action: Action::SaveProject,
    },
    Binding {
        modifiers: Modifiers {
            ctrl: true,
            shift: true,
            alt: false,
        },
        key: Key::S,
        action: Action::SaveProjectAs,
    },
    Binding {
        modifiers: Modifiers::CTRL,
        key: Key::W,
        action: Action::CloseProject,
    },
    Binding {
        modifiers: Modifiers::CTRL,
        key: Key::I,
        action: Action::ImportMedia,
    },
    Binding {
        modifiers: Modifiers::NONE,
        key: Key::F1,
        action: Action::ShowShortcuts,
    },
    Binding {
        modifiers: Modifiers::CTRL,
        key: Key::Q,
        action: Action::Quit,
    },
];

/// The command bound to `modifiers` and `key`, if the shell binds it.
pub fn resolve(modifiers: Modifiers, key: Key) -> Option<Action> {
    BINDINGS
        .iter()
        .find(|binding| binding.key == key && binding.modifiers == modifiers)
        .map(|binding| binding.action)
}

/// The chord bound to `action`, if it has one.
///
/// Menu and tooltip text use this so a shortcut is never described differently
/// from the way it behaves.
pub fn binding_for(action: Action) -> Option<Binding> {
    BINDINGS
        .iter()
        .find(|binding| binding.action == action)
        .copied()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_binding_is_reachable_by_its_own_chord() {
        for binding in BINDINGS {
            assert_eq!(
                resolve(binding.modifiers, binding.key),
                Some(binding.action),
                "{binding} is not reachable"
            );
        }
    }

    #[test]
    fn no_two_bindings_share_a_chord() {
        for (index, binding) in BINDINGS.iter().enumerate() {
            for other in &BINDINGS[index + 1..] {
                assert!(
                    binding.key != other.key || binding.modifiers != other.modifiers,
                    "{binding} and {other} share a chord"
                );
            }
        }
    }

    #[test]
    fn every_panel_can_be_shown_from_the_keyboard() {
        for panel in Panel::ALL {
            assert!(
                binding_for(Action::ShowPanel(panel)).is_some(),
                "no shortcut shows the {panel:?} panel"
            );
        }
    }

    #[test]
    fn unbound_chords_resolve_to_nothing() {
        assert_eq!(resolve(Modifiers::NONE, Key::Q), None);
        assert_eq!(resolve(Modifiers::CTRL, Key::F6), None);
        assert_eq!(resolve(Modifiers::SHIFT, Key::Num1), None);
    }

    #[test]
    fn chords_are_described_portably() {
        let describe = |modifiers, key| {
            Binding {
                modifiers,
                key,
                action: Action::ShowAbout,
            }
            .to_string()
        };

        assert_eq!(describe(Modifiers::CTRL, Key::Num1), "Ctrl+1");
        assert_eq!(describe(Modifiers::SHIFT, Key::F6), "Shift+F6");
        assert_eq!(describe(Modifiers::NONE, Key::F1), "F1");
        assert_eq!(describe(Modifiers::CTRL, Key::Q), "Ctrl+Q");
    }

    #[test]
    fn commands_without_a_shortcut_are_reported_as_such() {
        assert_eq!(binding_for(Action::TogglePanel(Panel::Media)), None);
        assert_eq!(binding_for(Action::ShowAbout), None);
    }
}
