//! The FireEffects visual identity.
//!
//! Colours live in one place so the window reads consistently: a warm ember
//! accent over near-black charcoal, carried by the wordmark, the focused panel
//! title, and selection.

use eframe::egui::{self, Color32, Stroke};

/// Ember accent: the wordmark, the focused panel title, and interactive highlights.
pub const EMBER: Color32 = Color32::from_rgb(232, 122, 61);
/// Deep ember, used where full-strength ember would be too loud behind text.
pub const EMBER_DEEP: Color32 = Color32::from_rgb(122, 62, 32);
/// Window background, darker than the panels so they read as raised surfaces.
pub const CANVAS: Color32 = Color32::from_rgb(16, 17, 21);
/// Standard panel surface.
pub const SURFACE: Color32 = Color32::from_rgb(26, 28, 34);
/// Chrome surfaces: menu bar and status bar.
pub const CHROME: Color32 = Color32::from_rgb(12, 13, 16);
/// Primary text.
pub const TEXT: Color32 = Color32::from_rgb(233, 228, 220);
/// Secondary text: hints, empty states, unfocused panel titles.
pub const TEXT_DIM: Color32 = Color32::from_rgb(143, 141, 135);

/// Applies the FireEffects theme to a context. Called once, at start-up.
pub fn apply(ctx: &egui::Context) {
    ctx.set_theme(egui::ThemePreference::Dark);
    ctx.all_styles_mut(|style| {
        style.visuals = visuals();
        style.spacing.item_spacing = egui::vec2(8.0, 6.0);
        style.spacing.button_padding = egui::vec2(8.0, 4.0);
    });
}

fn visuals() -> egui::Visuals {
    let mut visuals = egui::Visuals::dark();

    visuals.dark_mode = true;
    visuals.panel_fill = SURFACE;
    visuals.window_fill = SURFACE;
    visuals.extreme_bg_color = CANVAS;
    visuals.faint_bg_color = CHROME;
    visuals.override_text_color = Some(TEXT);
    visuals.hyperlink_color = EMBER;

    visuals.selection.bg_fill = EMBER_DEEP;
    visuals.selection.stroke = Stroke::new(1.0, TEXT);

    visuals.widgets.noninteractive.bg_stroke = Stroke::new(1.0, CHROME);
    visuals.widgets.inactive.fg_stroke = Stroke::new(1.0, TEXT);
    visuals.widgets.inactive.weak_bg_fill = SURFACE;
    visuals.widgets.hovered.weak_bg_fill = Color32::from_rgb(42, 45, 53);
    visuals.widgets.active.weak_bg_fill = EMBER_DEEP;

    visuals
}
