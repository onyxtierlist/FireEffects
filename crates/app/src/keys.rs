//! Translates window input into the shell's command model.
//!
//! Which chord does what is decided in `fireeffects_core`; this module only knows
//! how to ask egui which keys were pressed this frame and how its modifier flags
//! correspond to the shell's own.

use eframe::egui;
use fireeffects_core::{Action, Key, Modifiers, resolve};

/// The keys the window listens for, and the shell key each one stands for.
const WATCHED: [(Key, egui::Key); 12] = [
    (Key::Num1, egui::Key::Num1),
    (Key::Num2, egui::Key::Num2),
    (Key::Num3, egui::Key::Num3),
    (Key::Num4, egui::Key::Num4),
    (Key::Q, egui::Key::Q),
    (Key::F1, egui::Key::F1),
    (Key::F6, egui::Key::F6),
    (Key::N, egui::Key::N),
    (Key::O, egui::Key::O),
    (Key::S, egui::Key::S),
    (Key::W, egui::Key::W),
    (Key::I, egui::Key::I),
];

/// The command for a key pressed this frame, if the shell binds it.
///
/// `command` is Control on Windows and Linux and Command on macOS, which is what
/// makes one binding table usable on every platform.
pub fn pressed(input: &egui::InputState) -> Option<Action> {
    let modifiers = Modifiers {
        ctrl: input.modifiers.command,
        shift: input.modifiers.shift,
        alt: input.modifiers.alt,
    };

    WATCHED.iter().find_map(|(key, egui_key)| {
        input
            .key_pressed(*egui_key)
            .then(|| resolve(modifiers, *key))
            .flatten()
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use fireeffects_core::BINDINGS;

    #[test]
    fn every_binding_has_a_key_the_window_listens_for() {
        for binding in BINDINGS {
            assert!(
                WATCHED.iter().any(|(key, _)| *key == binding.key),
                "{binding} is bound but the window never looks for its key"
            );
        }
    }
}
