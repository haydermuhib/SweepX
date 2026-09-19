use egui::{Color32, Stroke, Visuals};

pub struct Theme;

impl Theme {
    // Backgrounds
    pub const BG_DARK: Color32 = Color32::from_rgb(15, 17, 23);
    pub const BG_CARD: Color32 = Color32::from_rgb(22, 25, 35);
    pub const BG_HOVER: Color32 = Color32::from_rgb(30, 35, 48);
    pub const BG_ACTIVE: Color32 = Color32::from_rgb(40, 46, 64);

    // Accents
    pub const PRIMARY: Color32 = Color32::from_rgb(99, 102, 241); // Vibrant Indigo
    pub const PRIMARY_HOVER: Color32 = Color32::from_rgb(129, 140, 248);
    pub const PRIMARY_MUTED: Color32 = Color32::from_rgb(49, 52, 100);
    pub const ACCENT_SUCCESS: Color32 = Color32::from_rgb(16, 185, 129); // Emerald
    pub const ACCENT_DANGER: Color32 = Color32::from_rgb(239, 68, 68); // Red
    pub const ACCENT_WARNING: Color32 = Color32::from_rgb(245, 158, 11); // Amber
    pub const ACCENT_CYAN: Color32 = Color32::from_rgb(14, 165, 233); // Sky Blue

    // Text
    pub const TEXT_PRIMARY: Color32 = Color32::from_rgb(248, 250, 252);
    pub const TEXT_SECONDARY: Color32 = Color32::from_rgb(203, 213, 225);
    pub const TEXT_MUTED: Color32 = Color32::from_rgb(148, 163, 184);

    // Borders
    pub const BORDER: Color32 = Color32::from_rgb(45, 52, 72);
    pub const BORDER_LIGHT: Color32 = Color32::from_rgb(60, 68, 92);

    pub fn apply(ctx: &egui::Context) {
        let mut visuals = Visuals::dark();
        visuals.override_text_color = Some(Self::TEXT_PRIMARY);
        visuals.panel_fill = Self::BG_DARK;
        visuals.window_fill = Self::BG_CARD;
        visuals.window_stroke = Stroke::new(1.0_f32, Self::BORDER);

        // Selection styling
        visuals.selection.bg_fill = Self::PRIMARY;
        visuals.selection.stroke = Stroke::new(1.0_f32, Self::PRIMARY_HOVER);

        // Noninteractive
        visuals.widgets.noninteractive.bg_fill = Self::BG_CARD;
        visuals.widgets.noninteractive.bg_stroke = Stroke::new(1.0_f32, Self::BORDER);
        visuals.widgets.noninteractive.fg_stroke = Stroke::new(1.0_f32, Self::TEXT_PRIMARY);

        // Inactive
        visuals.widgets.inactive.bg_fill = Self::BG_CARD;
        visuals.widgets.inactive.bg_stroke = Stroke::new(1.0_f32, Self::BORDER);
        visuals.widgets.inactive.fg_stroke = Stroke::new(1.0_f32, Self::TEXT_SECONDARY);

        // Hovered
        visuals.widgets.hovered.bg_fill = Self::BG_HOVER;
        visuals.widgets.hovered.bg_stroke = Stroke::new(1.0_f32, Self::PRIMARY);
        visuals.widgets.hovered.fg_stroke = Stroke::new(1.0_f32, Color32::WHITE);

        // Active
        visuals.widgets.active.bg_fill = Self::PRIMARY;
        visuals.widgets.active.bg_stroke = Stroke::new(1.5_f32, Self::PRIMARY_HOVER);
        visuals.widgets.active.fg_stroke = Stroke::new(1.0_f32, Color32::WHITE);

        // Hyperlink
        visuals.hyperlink_color = Self::PRIMARY_HOVER;

        ctx.set_visuals(visuals);
    }
}
