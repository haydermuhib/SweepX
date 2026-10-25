use crate::models::InstallMethod;
use iced::border::Border;
use iced::widget::{button, container, scrollable, text_input};
use iced::{Background, Color, Shadow, Vector};

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

    /// Recessed deep burgundy for inset modal items and panels (#320e17)
    pub const BURGUNDY_INSET: Color = Color::from_rgb(
        0x32 as f32 / 255.0,
        0x0e as f32 / 255.0,
        0x17 as f32 / 255.0,
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
    // Apple-Inspired Elevation & Depth Tokens (HIG Materials)
    // -------------------------------------------------------------

    /// Subtle resting ambient drop shadow for cream cards on wine canvas
    pub const ELEVATION_CARD: Shadow = Shadow {
        color: Color::from_rgba(
            0x12 as f32 / 255.0,
            0x04 as f32 / 255.0,
            0x08 as f32 / 255.0,
            0.45,
        ),
        offset: Vector::new(0.0, 3.0),
        blur_radius: 8.0,
    };

    /// Elevated lift shadow for active / selected cards
    pub const ELEVATION_CARD_ACTIVE: Shadow = Shadow {
        color: Color::from_rgba(
            0x10 as f32 / 255.0,
            0x02 as f32 / 255.0,
            0x06 as f32 / 255.0,
            0.60,
        ),
        offset: Vector::new(0.0, 6.0),
        blur_radius: 16.0,
    };

    /// Floating header navigation bar shadow
    pub const ELEVATION_HEADER: Shadow = Shadow {
        color: Color::from_rgba(
            0x10 as f32 / 255.0,
            0x02 as f32 / 255.0,
            0x06 as f32 / 255.0,
            0.50,
        ),
        offset: Vector::new(0.0, 4.0),
        blur_radius: 12.0,
    };

    /// Deep sheet elevation shadow for modal dialogues
    pub const ELEVATION_MODAL: Shadow = Shadow {
        color: Color::from_rgba(
            0x06 as f32 / 255.0,
            0x01 as f32 / 255.0,
            0x03 as f32 / 255.0,
            0.80,
        ),
        offset: Vector::new(0.0, 12.0),
        blur_radius: 30.0,
    };

    /// Tactile depth shadow for buttons
    pub const ELEVATION_BUTTON: Shadow = Shadow {
        color: Color::from_rgba(
            0x13 as f32 / 255.0,
            0x24 as f32 / 255.0,
            0x10 as f32 / 255.0,
            0.35,
        ),
        offset: Vector::new(0.0, 1.5),
        blur_radius: 3.5,
    };

    /// Tactile depth shadow for button hover
    pub const ELEVATION_BUTTON_HOVER: Shadow = Shadow {
        color: Color::from_rgba(
            0x13 as f32 / 255.0,
            0x24 as f32 / 255.0,
            0x10 as f32 / 255.0,
            0.48,
        ),
        offset: Vector::new(0.0, 2.5),
        blur_radius: 6.0,
    };

    /// Tactile depth shadow for button pressed
    pub const ELEVATION_BUTTON_PRESSED: Shadow = Shadow {
        color: Color::from_rgba(
            0x13 as f32 / 255.0,
            0x24 as f32 / 255.0,
            0x10 as f32 / 255.0,
            0.20,
        ),
        offset: Vector::new(0.0, 0.5),
        blur_radius: 1.5,
    };

    /// Danger button shadow
    pub const ELEVATION_DANGER: Shadow = Shadow {
        color: Color::from_rgba(0.50, 0.08, 0.12, 0.35),
        offset: Vector::new(0.0, 1.5),
        blur_radius: 4.0,
    };

    /// Danger button hover shadow
    pub const ELEVATION_DANGER_HOVER: Shadow = Shadow {
        color: Color::from_rgba(0.50, 0.08, 0.12, 0.45),
        offset: Vector::new(0.0, 2.5),
        blur_radius: 7.0,
    };

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
                radius: 10.0.into(),
            },
            shadow: Self::ELEVATION_CARD,
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
                radius: 10.0.into(),
            },
            shadow: Self::ELEVATION_CARD_ACTIVE,
            ..Default::default()
        }
    }

    pub fn header_style() -> container::Style {
        container::Style {
            background: Some(Background::Color(Self::BURGUNDY_MODAL)),
            text_color: Some(Self::TEXT_PRIMARY),
            border: Border {
                color: Self::BORDER_COLOR,
                width: 1.0,
                radius: 12.0.into(),
            },
            shadow: Self::ELEVATION_HEADER,
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
                radius: 14.0.into(),
            },
            shadow: Self::ELEVATION_MODAL,
            ..Default::default()
        }
    }

    /// Inset sub-card container for modals and dialogs
    pub fn modal_inset_card_style() -> container::Style {
        container::Style {
            background: Some(Background::Color(Self::BURGUNDY_INSET)),
            text_color: Some(Self::WARM_CREAM),
            border: Border {
                color: Self::BORDER_COLOR,
                width: 1.0,
                radius: 8.0.into(),
            },
            shadow: Shadow {
                color: Color::from_rgba(0.0, 0.0, 0.0, 0.25),
                offset: Vector::new(0.0, 1.0),
                blur_radius: 3.0,
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
                    radius: 7.0.into(),
                },
                shadow: Self::ELEVATION_BUTTON_HOVER,
                ..Default::default()
            },
            button::Status::Pressed => button::Style {
                background: Some(Background::Color(Self::PISTACHIO_PRESSED)),
                text_color: Self::PISTACHIO_TEXT_DARK,
                border: Border {
                    color: Self::PISTACHIO_PRESSED,
                    width: 1.0,
                    radius: 7.0.into(),
                },
                shadow: Self::ELEVATION_BUTTON_PRESSED,
                ..Default::default()
            },
            _ => button::Style {
                background: Some(Background::Color(Self::PISTACHIO)),
                text_color: Self::PISTACHIO_TEXT_DARK,
                border: Border {
                    color: Self::PISTACHIO,
                    width: 1.0,
                    radius: 7.0.into(),
                },
                shadow: Self::ELEVATION_BUTTON,
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
                    radius: 7.0.into(),
                },
                shadow: Self::ELEVATION_BUTTON_HOVER,
                ..Default::default()
            },
            button::Status::Pressed => button::Style {
                background: Some(Background::Color(Self::PISTACHIO_PRESSED)),
                text_color: Self::PISTACHIO_TEXT_DARK,
                border: Border {
                    color: Self::PISTACHIO_PRESSED,
                    width: 1.0,
                    radius: 7.0.into(),
                },
                shadow: Self::ELEVATION_BUTTON_PRESSED,
                ..Default::default()
            },
            _ => button::Style {
                background: Some(Background::Color(Self::PISTACHIO)),
                text_color: Self::PISTACHIO_TEXT_DARK,
                border: Border {
                    color: Self::PISTACHIO,
                    width: 1.0,
                    radius: 7.0.into(),
                },
                shadow: Self::ELEVATION_BUTTON,
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
                    radius: 7.0.into(),
                },
                shadow: Self::ELEVATION_DANGER_HOVER,
                ..Default::default()
            },
            _ => button::Style {
                background: Some(Background::Color(Self::DANGER)),
                text_color: Self::WARM_CREAM,
                border: Border {
                    color: Color::from_rgba(0.8, 0.2, 0.25, 0.6),
                    width: 1.0,
                    radius: 7.0.into(),
                },
                shadow: Self::ELEVATION_DANGER,
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

    // -------------------------------------------------------------
    // Scrollable Styling (Apple-design Floating Capsule Scrollbars)
    // -------------------------------------------------------------

    /// Returns standard slim floating vertical scrollbar geometry (8px width, 6px scroller, 3px margin).
    pub fn scrollbar_direction() -> scrollable::Direction {
        scrollable::Direction::Vertical(
            scrollable::Scrollbar::new()
                .width(8.0)
                .scroller_width(6.0)
                .margin(3.0),
        )
    }

    /// Elegant scrollbar style for content on canvas and cream cards.
    /// Features a transparent track and soft burgundy/mauve capsule scroller with pistachio hover illumination.
    pub fn scrollable_style(
        _theme: &iced::Theme,
        status: scrollable::Status,
    ) -> scrollable::Style {
        let (thumb_color, rail_bg) = match status {
            scrollable::Status::Active { .. } => (
                // Soft translucent muted burgundy pill floating on cream cards/canvas
                Color::from_rgba(0.43, 0.32, 0.35, 0.40),
                None,
            ),
            scrollable::Status::Hovered {
                is_vertical_scrollbar_hovered: true,
                ..
            } => (
                // Luminous pistachio accent on hover
                Self::PISTACHIO,
                Some(Background::Color(Color::from_rgba(0.22, 0.07, 0.11, 0.15))),
            ),
            scrollable::Status::Hovered { .. } => (
                Color::from_rgba(0.43, 0.32, 0.35, 0.55),
                None,
            ),
            scrollable::Status::Dragged { .. } => (
                // Active bright pistachio when dragged
                Self::PISTACHIO_HOVER,
                Some(Background::Color(Color::from_rgba(0.22, 0.07, 0.11, 0.22))),
            ),
        };

        let rail = scrollable::Rail {
            background: rail_bg,
            border: Border {
                radius: 999.0.into(),
                width: 0.0,
                color: Color::TRANSPARENT,
            },
            scroller: scrollable::Scroller {
                background: Background::Color(thumb_color),
                border: Border {
                    radius: 999.0.into(),
                    width: 0.0,
                    color: Color::TRANSPARENT,
                },
            },
        };

        scrollable::Style {
            container: container::Style::default(),
            vertical_rail: rail,
            horizontal_rail: rail,
            gap: None,
            auto_scroll: scrollable::AutoScroll {
                background: Background::Color(Color::TRANSPARENT),
                border: Border::default(),
                shadow: Shadow::default(),
                icon: Color::TRANSPARENT,
            },
        }
    }

    /// Elegant scrollbar style for dark modal dialogs.
    /// Features a subtle translucent cream pill with pistachio hover illumination.
    pub fn modal_scrollable_style(
        _theme: &iced::Theme,
        status: scrollable::Status,
    ) -> scrollable::Style {
        let (thumb_color, rail_bg) = match status {
            scrollable::Status::Active { .. } => (
                // Translucent cream capsule on dark burgundy modal surface
                Color::from_rgba(0.93, 0.90, 0.82, 0.28),
                None,
            ),
            scrollable::Status::Hovered {
                is_vertical_scrollbar_hovered: true,
                ..
            } => (
                // Luminous pistachio accent on hover
                Self::PISTACHIO,
                Some(Background::Color(Color::from_rgba(0.0, 0.0, 0.0, 0.25))),
            ),
            scrollable::Status::Hovered { .. } => (
                Color::from_rgba(0.93, 0.90, 0.82, 0.45),
                None,
            ),
            scrollable::Status::Dragged { .. } => (
                Self::PISTACHIO_HOVER,
                Some(Background::Color(Color::from_rgba(0.0, 0.0, 0.0, 0.35))),
            ),
        };

        let rail = scrollable::Rail {
            background: rail_bg,
            border: Border {
                radius: 999.0.into(),
                width: 0.0,
                color: Color::TRANSPARENT,
            },
            scroller: scrollable::Scroller {
                background: Background::Color(thumb_color),
                border: Border {
                    radius: 999.0.into(),
                    width: 0.0,
                    color: Color::TRANSPARENT,
                },
            },
        };

        scrollable::Style {
            container: container::Style::default(),
            vertical_rail: rail,
            horizontal_rail: rail,
            gap: None,
            auto_scroll: scrollable::AutoScroll {
                background: Background::Color(Color::TRANSPARENT),
                border: Border::default(),
                shadow: Shadow::default(),
                icon: Color::TRANSPARENT,
            },
        }
    }

    // -------------------------------------------------------------
    // Packaging Tier Origin Badges (Color-coded pills)
    // -------------------------------------------------------------

    /// Returns the (background_color, text_color) for an application's packaging tier badge by label.
    pub fn badge_colors_by_label(label: &str) -> (Color, Color) {
        let lower = label.to_lowercase();
        let bg = if lower.contains("dnf") || lower.contains("apt") {
            Color::from_rgb8(16, 185, 129) // Emerald Green (#10b981)
        } else if lower.contains("flatpak") {
            Color::from_rgb8(37, 99, 235)  // Royal Blue (#2563eb)
        } else if lower.contains("snap") {
            Color::from_rgb8(234, 88, 12)  // Canonical Orange (#ea580c)
        } else if lower.contains("pacman") {
            Color::from_rgb8(8, 145, 178)  // Arch Cyan (#0891b2)
        } else if lower.contains("appimage") {
            Color::from_rgb8(139, 92, 246) // AppImage Violet (#8b5cf6)
        } else if lower.contains("bottle") {
            Color::from_rgb8(190, 24, 93)  // Wine Magenta (#be185d)
        } else {
            Color::from_rgb8(71, 85, 105)  // Slate Gray (#475569) for Manual (/opt) & Desktop Entry
        };
        (bg, Color::WHITE)
    }

    /// Returns the (background_color, text_color) for an InstallMethod.
    pub fn method_badge_colors(method: InstallMethod) -> (Color, Color) {
        Self::badge_colors_by_label(method.badge_label())
    }

    /// Builds a styled packaging tier badge container element.
    pub fn method_badge<'a, Message: 'a>(method: InstallMethod) -> iced::Element<'a, Message> {
        Self::badge_by_label(method.badge_label())
    }

    /// Builds a styled packaging tier badge container element from a string label.
    pub fn badge_by_label<'a, Message: 'a>(label: &'a str) -> iced::Element<'a, Message> {
        let (bg, fg) = Self::badge_colors_by_label(label);
        container(
            iced::widget::text(label)
                .size(10)
                .color(fg),
        )
        .style(move |_t| container::Style {
            background: Some(Background::Color(bg)),
            border: Border {
                color: Color::TRANSPARENT,
                width: 0.0,
                radius: 4.0.into(),
            },
            ..Default::default()
        })
        .padding([2, 7])
        .into()
    }
}
