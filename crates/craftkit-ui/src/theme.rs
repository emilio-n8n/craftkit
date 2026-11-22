//! CraftKit's warm charcoal theme.
//!
//! A single dark theme keeps applications visually consistent and avoids a
//! half-implemented light mode. Applications may extend the palette, but the
//! base shell and widgets use these constants.

use eframe::egui::{
    self, Color32, CornerRadius, FontFamily, FontId, Stroke, TextStyle, Vec2, Visuals,
};

/// Application background.
pub const BG: Color32 = Color32::from_rgb(18, 20, 25);
/// Elevated panel background.
pub const PANEL: Color32 = Color32::from_rgb(25, 28, 35);
/// Sunken canvas / input background.
pub const SUNKEN: Color32 = Color32::from_rgb(14, 16, 21);
/// Hover / raised control background.
pub const RAISED: Color32 = Color32::from_rgb(34, 38, 47);
/// Main text.
pub const TEXT: Color32 = Color32::from_rgb(234, 238, 246);
/// Secondary text.
pub const MUTED: Color32 = Color32::from_rgb(140, 149, 165);
/// Dimmest text.
pub const FAINT: Color32 = Color32::from_rgb(94, 102, 117);
/// Borders and separators.
pub const BORDER: Color32 = Color32::from_rgb(44, 49, 60);
/// Stronger border.
pub const BORDER_STRONG: Color32 = Color32::from_rgb(62, 69, 83);
/// Warm accent.
pub const ACCENT: Color32 = Color32::from_rgb(232, 87, 63);
/// Brighter accent for hover.
pub const ACCENT_LIGHT: Color32 = Color32::from_rgb(246, 112, 86);
/// Positive / success.
pub const SUCCESS: Color32 = Color32::from_rgb(84, 209, 132);
/// Warning.
pub const WARNING: Color32 = Color32::from_rgb(238, 180, 86);
/// Error.
pub const ERROR: Color32 = Color32::from_rgb(244, 96, 96);
/// Informational.
pub const INFO: Color32 = Color32::from_rgb(106, 168, 245);

/// Apply the CraftKit look to an egui context.
pub fn apply(ctx: &egui::Context) {
    ctx.set_theme(egui::ThemePreference::Dark);

    let mut visuals = Visuals::dark();
    visuals.panel_fill = BG;
    visuals.window_fill = PANEL;
    visuals.window_stroke = Stroke::new(1.0, BORDER);
    visuals.window_corner_radius = CornerRadius::same(8);
    visuals.menu_corner_radius = CornerRadius::same(8);
    visuals.extreme_bg_color = SUNKEN;
    visuals.faint_bg_color = RAISED;
    visuals.code_bg_color = Color32::from_rgb(30, 34, 42);
    visuals.override_text_color = Some(TEXT);
    visuals.weak_text_color = Some(MUTED);
    visuals.hyperlink_color = ACCENT;
    visuals.warn_fg_color = WARNING;
    visuals.error_fg_color = ERROR;
    visuals.selection.bg_fill = Color32::from_rgba_unmultiplied(232, 87, 63, 90);
    visuals.selection.stroke = Stroke::new(1.0, ACCENT_LIGHT);
    visuals.striped = true;

    visuals.widgets.noninteractive.bg_fill = PANEL;
    visuals.widgets.noninteractive.weak_bg_fill = PANEL;
    visuals.widgets.noninteractive.bg_stroke = Stroke::new(1.0, BORDER);
    visuals.widgets.noninteractive.fg_stroke = Stroke::new(1.0, MUTED);
    visuals.widgets.noninteractive.corner_radius = CornerRadius::same(6);

    visuals.widgets.inactive.bg_fill = RAISED;
    visuals.widgets.inactive.weak_bg_fill = RAISED;
    visuals.widgets.inactive.bg_stroke = Stroke::new(1.0, BORDER);
    visuals.widgets.inactive.fg_stroke = Stroke::new(1.0, TEXT);
    visuals.widgets.inactive.corner_radius = CornerRadius::same(6);

    visuals.widgets.hovered.bg_fill = Color32::from_rgb(43, 48, 59);
    visuals.widgets.hovered.weak_bg_fill = Color32::from_rgb(43, 48, 59);
    visuals.widgets.hovered.bg_stroke = Stroke::new(1.0, BORDER_STRONG);
    visuals.widgets.hovered.fg_stroke = Stroke::new(1.0, TEXT);
    visuals.widgets.hovered.corner_radius = CornerRadius::same(6);

    visuals.widgets.active.bg_fill = Color32::from_rgb(52, 58, 71);
    visuals.widgets.active.weak_bg_fill = Color32::from_rgb(52, 58, 71);
    visuals.widgets.active.bg_stroke = Stroke::new(1.0, ACCENT);
    visuals.widgets.active.fg_stroke = Stroke::new(1.0, TEXT);
    visuals.widgets.active.corner_radius = CornerRadius::same(6);
    visuals.widgets.active.expansion = 0.0;

    visuals.widgets.open = visuals.widgets.inactive;
    ctx.set_visuals(visuals);

    let mut style = (*ctx.style_of(egui::Theme::Dark)).clone();
    style.spacing.item_spacing = Vec2::new(8.0, 6.0);
    style.spacing.button_padding = Vec2::new(10.0, 5.0);
    style.spacing.interact_size = Vec2::new(24.0, 24.0);
    style.spacing.scroll = egui::style::ScrollStyle::solid();
    style.spacing.window_margin = egui::Margin::same(12);
    style.spacing.menu_margin = egui::Margin::same(8);
    style.text_styles.insert(
        TextStyle::Heading,
        FontId::new(22.0, FontFamily::Proportional),
    );
    style
        .text_styles
        .insert(TextStyle::Body, FontId::new(13.5, FontFamily::Proportional));
    style.text_styles.insert(
        TextStyle::Button,
        FontId::new(13.0, FontFamily::Proportional),
    );
    style.text_styles.insert(
        TextStyle::Small,
        FontId::new(11.0, FontFamily::Proportional),
    );
    style.text_styles.insert(
        TextStyle::Monospace,
        FontId::new(12.0, FontFamily::Monospace),
    );
    ctx.set_style_of(egui::Theme::Dark, style);
}
