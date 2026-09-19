use egui::{Color32, Stroke, Visuals};

pub struct Theme;

impl Theme {
    // Backgrounds
    pub const BG_DARK: Color32 = Color32::from_rgb(18, 20, 26);
    pub const BG_CARD: Color32 = Color32::from_rgb(26, 29, 38);
    pub const BG_HOVER: Color32 = Color32::from_rgb(34, 38, 50);
    pub const BG_ACTIVE: Color32 = Color32::from_rgb(42, 47, 62);

    // Accents
    pub const PRIMARY: Color32 = Color32::from_rgb(99, 102, 241); // Indigo
    pub const PRIMARY_HOVER: Color32 = Color32::from_rgb(129, 140, 248);
    pub const ACCENT_SUCCESS: Color32 = Color32::from_rgb(16, 185, 129); // Emerald
    pub const ACCENT_DANGER: Color32 = Color32::from_rgb(239, 68, 68); // Red
    pub const ACCENT_WARNING: Color32 = Color32::from_rgb(245, 158, 11); // Amber
    pub const ACCENT_CYAN: Color32 = Color32::from_rgb(6, 182, 212); // Cyan

    // Text
    pub const TEXT_PRIMARY: Color32 = Color32::from_rgb(243, 244, 246);
    pub const TEXT_SECONDARY: Color32 = Color32::from_rgb(156, 163, 175);
    pub const TEXT_MUTED: Color32 = Color32::from_rgb(107, 114, 128);

    // Borders
    pub const BORDER: Color32 = Color32::from_rgb(45, 50, 65);

    pub fn apply(ctx: &egui::Context) {
        let mut visuals = Visuals::dark();
        visuals.override_text_color = Some(Self::TEXT_PRIMARY);
        visuals.panel_fill = Self::BG_DARK;
        visuals.window_fill = Self::BG_CARD;
        visuals.window_stroke = Stroke::new(1.0_f32, Self::BORDER);

        visuals.widgets.noninteractive.bg_fill = Self::BG_CARD;
        visuals.widgets.noninteractive.bg_stroke = Stroke::new(1.0_f32, Self::BORDER);
        visuals.widgets.noninteractive.fg_stroke = Stroke::new(1.0_f32, Self::TEXT_PRIMARY);

        visuals.widgets.inactive.bg_fill = Self::BG_CARD;
        visuals.widgets.inactive.bg_stroke = Stroke::new(1.0_f32, Self::BORDER);
        visuals.widgets.inactive.fg_stroke = Stroke::new(1.0_f32, Self::TEXT_SECONDARY);

        visuals.widgets.hovered.bg_fill = Self::BG_HOVER;
        visuals.widgets.hovered.bg_stroke = Stroke::new(1.0_f32, Self::PRIMARY);
        visuals.widgets.hovered.fg_stroke = Stroke::new(1.0_f32, Self::TEXT_PRIMARY);

        visuals.widgets.active.bg_fill = Self::BG_ACTIVE;
        visuals.widgets.active.bg_stroke = Stroke::new(1.5_f32, Self::PRIMARY);
        visuals.widgets.active.fg_stroke = Stroke::new(1.0_f32, Self::TEXT_PRIMARY);

        ctx.set_visuals(visuals);
    }
}
