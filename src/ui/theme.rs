use iced::border::Border;
use iced::widget::{button, container, text_input};
use iced::{Background, Color};

pub struct Theme;

impl Theme {
    // -------------------------------------------------------------
    // Core Palette (Burgundy Canvas/Modals, Cream Cards, Pistachio Buttons)
    // -------------------------------------------------------------

    /// Rich, deep burgundy wine canvas background (#38121c)
    pub const BURGUNDY_CANVAS: Color = Color::from_rgb(
        0x38 as f32 / 255.0,
        0x12 as f32 / 255.0,
        0x1c as f32 / 255.0,
    );

    /// Elevated burgundy for modal surfaces (#421622)
    pub const BURGUNDY_MODAL: Color = Color::from_rgb(
        0x42 as f32 / 255.0,
        0x16 as f32 / 255.0,
        0x22 as f32 / 255.0,
    );

    /// Warm, elegant cream for result cards (#ede5d2)
    pub const CARD_CREAM: Color = Color::from_rgb(
        0xed as f32 / 255.0,
        0xe5 as f32 / 255.0,
        0xd2 as f32 / 255.0,
    );

    /// Hover state for cream result cards (#f6efe1)
    pub const CARD_CREAM_HOVER: Color = Color::from_rgb(
        0xf6 as f32 / 255.0,
        0xef as f32 / 255.0,
        0xe1 as f32 / 255.0,
    );

    /// Active state for cream result cards (#fbf7ee)
    pub const CARD_CREAM_ACTIVE: Color = Color::from_rgb(
        0xfb as f32 / 255.0,
        0xf7 as f32 / 255.0,
        0xee as f32 / 255.0,
    );

    /// Deep burgundy-black ink for ultra-clear text on cream cards (#2a0c14)
    pub const CARD_TEXT_PRIMARY: Color = Color::from_rgb(
        0x2a as f32 / 255.0,
        0x0c as f32 / 255.0,
        0x14 as f32 / 255.0,
    );

    /// Muted burgundy gray for secondary labels on cream cards (#6d525a)
    pub const CARD_TEXT_MUTED: Color = Color::from_rgb(
        0x6d as f32 / 255.0,
        0x52 as f32 / 255.0,
        0x5a as f32 / 255.0,
    );

    /// Fine warm border for cream cards (#dcd1bb)
    pub const CARD_BORDER: Color = Color::from_rgba(
        0xdc as f32 / 255.0,
        0xd1 as f32 / 255.0,
        0xbb as f32 / 255.0,
        0.90,
    );

    /// Fresh pistachio green for buttons & interactive elements (#9ecb82)
    pub const PISTACHIO: Color = Color::from_rgb(
        0x9e as f32 / 255.0,
        0xcb as f32 / 255.0,
        0x82 as f32 / 255.0,
    );

    /// Lighter pistachio for hover state (#b4dd98)
    pub const PISTACHIO_HOVER: Color = Color::from_rgb(
        0xb4 as f32 / 255.0,
        0xdd as f32 / 255.0,
        0x98 as f32 / 255.0,
    );

    /// Darker pistachio for pressed state (#88b56e)
    pub const PISTACHIO_PRESSED: Color = Color::from_rgb(
        0x88 as f32 / 255.0,
        0xb5 as f32 / 255.0,
        0x6e as f32 / 255.0,
    );

    /// Dark ink text for high contrast on light pistachio surfaces (#132410)
    pub const PISTACHIO_TEXT_DARK: Color = Color::from_rgb(
        0x13 as f32 / 255.0,
        0x24 as f32 / 255.0,
        0x10 as f32 / 255.0,
    );

    /// Translucent pistachio tint for subtle button backgrounds
    pub const PISTACHIO_TINT: Color = Color::from_rgba(
        0x9e as f32 / 255.0,
        0xcb as f32 / 255.0,
        0x82 as f32 / 255.0,
        0.20,
    );

    // Compatibility aliases
    pub const MAROON_WINE: Color = Self::BURGUNDY_CANVAS;
    pub const OLIVE_SAGE: Color = Self::PISTACHIO;
    pub const CARD_GREEN: Color = Self::CARD_CREAM;

    // Background aliases
    pub const BG_CANVAS: Color = Self::BURGUNDY_CANVAS;
    pub const BG_SURFACE: Color = Self::CARD_CREAM;
    pub const BG_SURFACE_HOVER: Color = Self::CARD_CREAM_HOVER;

    // Canvas Typography
    pub const WARM_CREAM: Color = Color::from_rgb(
        0xf4 as f32 / 255.0,
        0xef as f32 / 255.0,
        0xe4 as f32 / 255.0,
    );
    pub const TEXT_PRIMARY: Color = Self::WARM_CREAM;
    pub const TEXT_MUTED: Color = Color::from_rgb(
        0xa7 as f32 / 255.0,
        0xbf as f32 / 255.0,
        0xad as f32 / 255.0,
    );

    // Borders
    pub const BORDER_COLOR: Color = Color::from_rgba(
        0x9e as f32 / 255.0,
        0xcb as f32 / 255.0,
        0x82 as f32 / 255.0,
        0.28,
    );
    pub const BORDER_ACCENT: Color = Self::PISTACHIO;

    // Purge Danger Red (Exception: Purge button stays red)
    pub const DANGER: Color = Color::from_rgb(
        0xcc as f32 / 255.0,
        0x33 as f32 / 255.0,
        0x44 as f32 / 255.0,
    );
    pub const DANGER_HOVER: Color = Color::from_rgb(
        0xe2 as f32 / 255.0,
        0x44 as f32 / 255.0,
        0x56 as f32 / 255.0,
    );

    // -------------------------------------------------------------
    // Styles
    // -------------------------------------------------------------

    pub fn card_style() -> container::Style {
        container::Style {
            background: Some(Background::Color(Self::CARD_CREAM)),
            text_color: Some(Self::CARD_TEXT_PRIMARY),
            border: Border {
                color: Self::CARD_BORDER,
                width: 1.0,
                radius: 8.0.into(),
            },
            ..Default::default()
        }
    }

    pub fn active_card_style() -> container::Style {
        container::Style {
            background: Some(Background::Color(Self::CARD_CREAM_ACTIVE)),
            text_color: Some(Self::CARD_TEXT_PRIMARY),
            border: Border {
                color: Self::PISTACHIO,
                width: 1.5,
                radius: 8.0.into(),
            },
            ..Default::default()
        }
    }

    pub fn modal_style() -> container::Style {
        container::Style {
            background: Some(Background::Color(Self::BURGUNDY_MODAL)),
            text_color: Some(Self::TEXT_PRIMARY),
            border: Border {
                color: Self::PISTACHIO,
                width: 1.5,
                radius: 12.0.into(),
            },
            ..Default::default()
        }
    }

    pub fn primary_button(status: button::Status) -> button::Style {
        match status {
            button::Status::Hovered => button::Style {
                background: Some(Background::Color(Self::PISTACHIO_HOVER)),
                text_color: Self::PISTACHIO_TEXT_DARK,
                border: Border {
                    color: Self::PISTACHIO_HOVER,
                    width: 1.0,
                    radius: 6.0.into(),
                },
                ..Default::default()
            },
            button::Status::Pressed => button::Style {
                background: Some(Background::Color(Self::PISTACHIO_PRESSED)),
                text_color: Self::PISTACHIO_TEXT_DARK,
                border: Border {
                    color: Self::PISTACHIO_PRESSED,
                    width: 1.0,
                    radius: 6.0.into(),
                },
                ..Default::default()
            },
            _ => button::Style {
                background: Some(Background::Color(Self::PISTACHIO)),
                text_color: Self::PISTACHIO_TEXT_DARK,
                border: Border {
                    color: Self::PISTACHIO,
                    width: 1.0,
                    radius: 6.0.into(),
                },
                ..Default::default()
            },
        }
    }

    pub fn secondary_button(status: button::Status) -> button::Style {
        match status {
            button::Status::Hovered => button::Style {
                background: Some(Background::Color(Self::PISTACHIO_HOVER)),
                text_color: Self::PISTACHIO_TEXT_DARK,
                border: Border {
                    color: Self::PISTACHIO_HOVER,
                    width: 1.0,
                    radius: 6.0.into(),
                },
                ..Default::default()
            },
            button::Status::Pressed => button::Style {
                background: Some(Background::Color(Self::PISTACHIO_PRESSED)),
                text_color: Self::PISTACHIO_TEXT_DARK,
                border: Border {
                    color: Self::PISTACHIO_PRESSED,
                    width: 1.0,
                    radius: 6.0.into(),
                },
                ..Default::default()
            },
            _ => button::Style {
                background: Some(Background::Color(Self::PISTACHIO)),
                text_color: Self::PISTACHIO_TEXT_DARK,
                border: Border {
                    color: Self::PISTACHIO,
                    width: 1.0,
                    radius: 6.0.into(),
                },
                ..Default::default()
            },
        }
    }

    pub fn danger_button(status: button::Status) -> button::Style {
        match status {
            button::Status::Hovered => button::Style {
                background: Some(Background::Color(Self::DANGER_HOVER)),
                text_color: Self::WARM_CREAM,
                border: Border {
                    color: Self::DANGER_HOVER,
                    width: 1.0,
                    radius: 6.0.into(),
                },
                ..Default::default()
            },
            _ => button::Style {
                background: Some(Background::Color(Self::DANGER)),
                text_color: Self::WARM_CREAM,
                border: Border {
                    color: Color::from_rgba(0.8, 0.2, 0.25, 0.6),
                    width: 1.0,
                    radius: 6.0.into(),
                },
                ..Default::default()
            },
        }
    }

    pub fn ghost_button(is_active: bool, status: button::Status) -> button::Style {
        if is_active {
            button::Style {
                background: Some(Background::Color(Self::PISTACHIO)),
                text_color: Self::PISTACHIO_TEXT_DARK,
                border: Border {
                    color: Self::PISTACHIO,
                    width: 1.0,
                    radius: 6.0.into(),
                },
                ..Default::default()
            }
        } else {
            match status {
                button::Status::Hovered => button::Style {
                    background: Some(Background::Color(Self::PISTACHIO_TINT)),
                    text_color: Self::WARM_CREAM,
                    border: Border {
                        color: Self::BORDER_COLOR,
                        width: 1.0,
                        radius: 6.0.into(),
                    },
                    ..Default::default()
                },
                _ => button::Style {
                    background: None,
                    text_color: Self::TEXT_MUTED,
                    border: Border {
                        color: Color::TRANSPARENT,
                        width: 0.0,
                        radius: 6.0.into(),
                    },
                    ..Default::default()
                },
            }
        }
    }

    pub fn card_ghost_button(status: button::Status) -> button::Style {
        match status {
            button::Status::Hovered => button::Style {
                background: Some(Background::Color(Self::PISTACHIO)),
                text_color: Self::PISTACHIO_TEXT_DARK,
                border: Border {
                    color: Self::PISTACHIO,
                    width: 1.0,
                    radius: 4.0.into(),
                },
                ..Default::default()
            },
            _ => button::Style {
                background: Some(Background::Color(Color::from_rgba(0.0, 0.0, 0.0, 0.06))),
                text_color: Self::CARD_TEXT_MUTED,
                border: Border {
                    color: Color::from_rgba(0.0, 0.0, 0.0, 0.15),
                    width: 1.0,
                    radius: 4.0.into(),
                },
                ..Default::default()
            },
        }
    }

    pub fn search_input(status: text_input::Status) -> text_input::Style {
        match status {
            text_input::Status::Focused { .. } => text_input::Style {
                background: Background::Color(Self::BURGUNDY_MODAL),
                border: Border {
                    color: Self::PISTACHIO,
                    width: 1.5,
                    radius: 8.0.into(),
                },
                icon: Self::PISTACHIO,
                placeholder: Self::TEXT_MUTED,
                value: Self::WARM_CREAM,
                selection: Self::BURGUNDY_CANVAS,
            },
            _ => text_input::Style {
                background: Background::Color(Self::BURGUNDY_MODAL),
                border: Border {
                    color: Self::BORDER_COLOR,
                    width: 1.0,
                    radius: 8.0.into(),
                },
                icon: Self::TEXT_MUTED,
                placeholder: Self::TEXT_MUTED,
                value: Self::WARM_CREAM,
                selection: Self::BURGUNDY_CANVAS,
            },
        }
    }

    pub fn selectable_text(status: text_input::Status) -> text_input::Style {
        match status {
            text_input::Status::Focused { .. } => text_input::Style {
                background: Background::Color(Color::from_rgba(0.0, 0.0, 0.0, 0.06)),
                border: Border {
                    color: Self::PISTACHIO,
                    width: 1.0,
                    radius: 4.0.into(),
                },
                icon: Self::PISTACHIO,
                placeholder: Self::CARD_TEXT_MUTED,
                value: Self::CARD_TEXT_PRIMARY,
                selection: Self::PISTACHIO,
            },
            _ => text_input::Style {
                background: Background::Color(Color::TRANSPARENT),
                border: Border {
                    color: Color::TRANSPARENT,
                    width: 0.0,
                    radius: 4.0.into(),
                },
                icon: Self::CARD_TEXT_MUTED,
                placeholder: Self::CARD_TEXT_MUTED,
                value: Self::CARD_TEXT_MUTED,
                selection: Self::PISTACHIO,
            },
        }
    }
}
