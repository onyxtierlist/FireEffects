//! Workspace layout state: which panels are visible and which one has focus.
//!
//! The rules are deliberately small and explicit. Menus, keyboard shortcuts, and
//! clicks in the window all route through this type, so the shell cannot behave
//! one way from a menu and another way from a key press.

use crate::{Error, Result};

/// A region of the editor workspace.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum Panel {
    /// Project and media browser, on the left.
    Media,
    /// Programme monitor, in the centre.
    Preview,
    /// Timeline, below the monitor.
    Timeline,
    /// Properties inspector, on the right.
    Inspector,
}

impl Panel {
    /// Every panel, in the order focus moves through them.
    pub const ALL: [Self; 4] = [Self::Media, Self::Preview, Self::Timeline, Self::Inspector];

    const fn index(self) -> usize {
        match self {
            Self::Media => 0,
            Self::Preview => 1,
            Self::Timeline => 2,
            Self::Inspector => 3,
        }
    }
}

/// Which panels are visible, and which one holds focus.
///
/// Focus is the panel that keyboard commands and menu actions apply to. A hidden
/// panel can never hold focus, and at least one panel stays visible.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Workspace {
    visible: [bool; Panel::ALL.len()],
    focused: Panel,
}

impl Default for Workspace {
    fn default() -> Self {
        Self::new()
    }
}

impl Workspace {
    /// A workspace with every panel visible and the media panel focused.
    pub const fn new() -> Self {
        Self {
            visible: [true; Panel::ALL.len()],
            focused: Panel::Media,
        }
    }

    /// The panel that currently holds focus.
    pub const fn focused(&self) -> Panel {
        self.focused
    }

    /// Whether `panel` is visible.
    pub const fn is_visible(&self, panel: Panel) -> bool {
        self.visible[panel.index()]
    }

    /// The visible panels, in focus order.
    pub fn visible(&self) -> impl Iterator<Item = Panel> + '_ {
        Panel::ALL
            .into_iter()
            .filter(|panel| self.is_visible(*panel))
    }

    /// How many panels are visible.
    pub fn visible_count(&self) -> usize {
        self.visible.iter().filter(|visible| **visible).count()
    }

    /// Shows or hides `panel`.
    ///
    /// Hiding the focused panel moves focus to the next visible panel. Hiding the
    /// last visible panel is refused, because the workspace would have nowhere to
    /// put focus.
    pub fn set_visible(&mut self, panel: Panel, visible: bool) -> Result<()> {
        if !visible {
            if !self.is_visible(panel) {
                return Ok(());
            }
            if self.visible_count() == 1 {
                return Err(Error::LastVisiblePanel);
            }
            if self.focused == panel {
                self.focused = self
                    .next_visible_after(panel, Direction::Forward)
                    .ok_or(Error::LastVisiblePanel)?;
            }
        }

        self.visible[panel.index()] = visible;
        Ok(())
    }

    /// Shows `panel` if it was hidden, hides it if it was visible.
    pub fn toggle(&mut self, panel: Panel) -> Result<()> {
        self.set_visible(panel, !self.is_visible(panel))
    }

    /// Gives focus to `panel`.
    pub fn focus(&mut self, panel: Panel) -> Result<()> {
        if !self.is_visible(panel) {
            return Err(Error::PanelNotVisible);
        }
        self.focused = panel;
        Ok(())
    }

    /// Shows `panel` and gives it focus.
    pub fn reveal(&mut self, panel: Panel) {
        self.visible[panel.index()] = true;
        self.focused = panel;
    }

    /// Moves focus to the next visible panel, wrapping around.
    pub fn focus_next(&mut self) {
        if let Some(panel) = self.next_visible_after(self.focused, Direction::Forward) {
            self.focused = panel;
        }
    }

    /// Moves focus to the previous visible panel, wrapping around.
    pub fn focus_previous(&mut self) {
        if let Some(panel) = self.next_visible_after(self.focused, Direction::Backward) {
            self.focused = panel;
        }
    }

    fn next_visible_after(&self, panel: Panel, direction: Direction) -> Option<Panel> {
        let count = Panel::ALL.len();
        let start = panel.index();

        (1..=count).find_map(|step| {
            let offset = start + count - step;
            let index = match direction {
                Direction::Forward => (start + step) % count,
                Direction::Backward => offset % count,
            };
            let candidate = Panel::ALL[index];
            (candidate != panel && self.is_visible(candidate)).then_some(candidate)
        })
    }
}

#[derive(Debug, Clone, Copy)]
enum Direction {
    Forward,
    Backward,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_with_every_panel_visible() {
        let workspace = Workspace::new();

        assert_eq!(workspace.visible_count(), Panel::ALL.len());
        assert_eq!(workspace.visible().collect::<Vec<_>>(), Panel::ALL);
        assert_eq!(workspace.focused(), Panel::Media);
    }

    #[test]
    fn hiding_the_focused_panel_moves_focus_forward() {
        let mut workspace = Workspace::new();
        workspace.focus(Panel::Preview).expect("visible panel");

        workspace
            .toggle(Panel::Preview)
            .expect("other panels remain");

        assert!(!workspace.is_visible(Panel::Preview));
        assert_eq!(workspace.focused(), Panel::Timeline);
    }

    #[test]
    fn hiding_the_focused_panel_wraps_around() {
        let mut workspace = Workspace::new();
        workspace.focus(Panel::Inspector).expect("visible panel");

        workspace
            .toggle(Panel::Inspector)
            .expect("other panels remain");

        assert_eq!(workspace.focused(), Panel::Media);
    }

    #[test]
    fn hiding_an_unfocused_panel_keeps_focus() {
        let mut workspace = Workspace::new();
        workspace.focus(Panel::Timeline).expect("visible panel");

        workspace.toggle(Panel::Media).expect("other panels remain");

        assert_eq!(workspace.focused(), Panel::Timeline);
    }

    #[test]
    fn refuses_to_hide_the_last_visible_panel() {
        let mut workspace = Workspace::new();
        for panel in [Panel::Media, Panel::Preview, Panel::Timeline] {
            workspace.toggle(panel).expect("one panel remains visible");
        }

        assert_eq!(workspace.visible_count(), 1);
        assert_eq!(
            workspace.toggle(Panel::Inspector),
            Err(Error::LastVisiblePanel)
        );
        assert!(workspace.is_visible(Panel::Inspector));
    }

    #[test]
    fn refuses_to_focus_a_hidden_panel() {
        let mut workspace = Workspace::new();
        workspace
            .toggle(Panel::Inspector)
            .expect("other panels remain");

        assert_eq!(
            workspace.focus(Panel::Inspector),
            Err(Error::PanelNotVisible)
        );
        assert_eq!(workspace.focused(), Panel::Media);
    }

    #[test]
    fn reveal_shows_and_focuses_a_hidden_panel() {
        let mut workspace = Workspace::new();
        workspace
            .toggle(Panel::Timeline)
            .expect("other panels remain");

        workspace.reveal(Panel::Timeline);

        assert!(workspace.is_visible(Panel::Timeline));
        assert_eq!(workspace.focused(), Panel::Timeline);
    }

    #[test]
    fn focus_cycles_through_visible_panels_only() {
        let mut workspace = Workspace::new();
        workspace
            .toggle(Panel::Preview)
            .expect("other panels remain");
        workspace.focus(Panel::Media).expect("visible panel");

        workspace.focus_next();
        assert_eq!(workspace.focused(), Panel::Timeline);

        workspace.focus_previous();
        assert_eq!(workspace.focused(), Panel::Media);
    }

    #[test]
    fn focus_cycling_wraps_in_both_directions() {
        let mut workspace = Workspace::new();

        workspace.focus_previous();
        assert_eq!(workspace.focused(), Panel::Inspector);

        workspace.focus_next();
        assert_eq!(workspace.focused(), Panel::Media);
    }

    #[test]
    fn cycling_with_one_visible_panel_stays_put() {
        let mut workspace = Workspace::new();
        for panel in [Panel::Media, Panel::Preview, Panel::Timeline] {
            workspace.toggle(panel).expect("one panel remains visible");
        }

        workspace.focus_next();
        workspace.focus_previous();

        assert_eq!(workspace.focused(), Panel::Inspector);
    }
}
