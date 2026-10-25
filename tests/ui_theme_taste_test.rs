use iced::widget::button;
use iced::{Background, Color};
use sweepx::ui::Theme;

/// Helper to calculate relative luminance according to WCAG specifications
fn relative_luminance(c: Color) -> f32 {
    let f = |channel: f32| {
        if channel <= 0.03928 {
            channel / 12.92
        } else {
            ((channel + 0.055) / 1.055).powf(2.4)
        }
    };
    0.2126 * f(c.r) + 0.7152 * f(c.g) + 0.0722 * f(c.b)
}

/// Helper to calculate WCAG contrast ratio between two colors
fn contrast_ratio(c1: Color, c2: Color) -> f32 {
    let l1 = relative_luminance(c1);
    let l2 = relative_luminance(c2);
    let (lighter, darker) = if l1 > l2 { (l1, l2) } else { (l2, l1) };
    (lighter + 0.05) / (darker + 0.05)
}


#[test]
#[allow(clippy::assertions_on_constants)]
fn test_palette_color_identity() {
    // 1. Burgundy Canvas & Modal: Red dominant, dark luminance
    assert!(Theme::BURGUNDY_CANVAS.r > Theme::BURGUNDY_CANVAS.g);
    assert!(Theme::BURGUNDY_CANVAS.r > Theme::BURGUNDY_CANVAS.b);
    assert!(relative_luminance(Theme::BURGUNDY_CANVAS) < 0.15, "Burgundy canvas must be dark");

    assert!(Theme::BURGUNDY_MODAL.r > Theme::BURGUNDY_MODAL.g);
    assert!(Theme::BURGUNDY_MODAL.r > Theme::BURGUNDY_MODAL.b);
    assert!(relative_luminance(Theme::BURGUNDY_MODAL) < 0.20, "Burgundy modal must be dark");

    // 2. Result Cards Cream: Warm cream, light luminance
    assert!(relative_luminance(Theme::CARD_CREAM) > 0.65, "Result cards must be light warm cream");
    assert!(Theme::CARD_CREAM.r >= Theme::CARD_CREAM.g);
    assert!(Theme::CARD_CREAM.g >= Theme::CARD_CREAM.b);

    // 3. Pistachio: Green > Red > Blue, vibrant and light
    assert!(Theme::PISTACHIO.g > Theme::PISTACHIO.r);
    assert!(Theme::PISTACHIO.r > Theme::PISTACHIO.b);
    assert!(relative_luminance(Theme::PISTACHIO) > 0.40, "Pistachio must be light and fresh");

    // 4. Danger Red (Purge button exception): Red dominant, high saturation
    assert!(Theme::DANGER.r > 0.65);
    assert!(Theme::DANGER.g < 0.35);
    assert!(Theme::DANGER.b < 0.35);
}

#[test]
fn test_wcag_contrast_ratios_for_taste_and_accessibility() {
    // Text on Cream Result Cards: Deep Burgundy ink on Cream Cards must exceed WCAG AAA (7.0:1)
    let card_text_contrast = contrast_ratio(Theme::CARD_CREAM, Theme::CARD_TEXT_PRIMARY);
    assert!(
        card_text_contrast >= 7.0,
        "Card text contrast ({:.2}) on cream cards must meet WCAG AAA standard (>= 7.0:1)",
        card_text_contrast
    );

    // Secondary text on Cream Result Cards must meet WCAG AA (4.5:1)
    let card_muted_contrast = contrast_ratio(Theme::CARD_CREAM, Theme::CARD_TEXT_MUTED);
    assert!(
        card_muted_contrast >= 4.5,
        "Muted card text contrast ({:.2}) on cream cards must meet WCAG AA standard (>= 4.5:1)",
        card_muted_contrast
    );

    // Text on Pistachio buttons: Dark ink on pistachio must exceed WCAG AA (4.5:1)
    let button_contrast = contrast_ratio(Theme::PISTACHIO, Theme::PISTACHIO_TEXT_DARK);
    assert!(
        button_contrast >= 4.5,
        "Button text contrast ({:.2}) must meet WCAG AA standard (>= 4.5:1)",
        button_contrast
    );

    // Text on Burgundy Canvas & Modals: Warm cream on Burgundy must exceed WCAG AA
    let canvas_contrast = contrast_ratio(Theme::BURGUNDY_CANVAS, Theme::WARM_CREAM);
    assert!(
        canvas_contrast >= 4.5,
        "Canvas text contrast ({:.2}) must meet WCAG AA standard (>= 4.5:1)",
        canvas_contrast
    );

    let modal_contrast = contrast_ratio(Theme::BURGUNDY_MODAL, Theme::WARM_CREAM);
    assert!(
        modal_contrast >= 4.5,
        "Modal text contrast ({:.2}) must meet WCAG AA standard (>= 4.5:1)",
        modal_contrast
    );
}

#[test]
fn test_primary_and_secondary_button_styles_use_pistachio() {
    // Primary Button Normal
    let primary_normal = Theme::primary_button(button::Status::Active);
    assert_eq!(primary_normal.background, Some(Background::Color(Theme::PISTACHIO)));
    assert_eq!(primary_normal.text_color, Theme::PISTACHIO_TEXT_DARK);

    // Primary Button Hover
    let primary_hover = Theme::primary_button(button::Status::Hovered);
    assert_eq!(primary_hover.background, Some(Background::Color(Theme::PISTACHIO_HOVER)));
    assert_eq!(primary_hover.text_color, Theme::PISTACHIO_TEXT_DARK);

    // Secondary Button Hover fills with Pistachio
    let sec_hover = Theme::secondary_button(button::Status::Hovered);
    assert_eq!(sec_hover.background, Some(Background::Color(Theme::PISTACHIO_HOVER)));
    assert_eq!(sec_hover.text_color, Theme::PISTACHIO_TEXT_DARK);
}

#[test]
fn test_danger_button_retains_red_exception_for_purge() {
    let danger_normal = Theme::danger_button(button::Status::Active);
    assert_eq!(danger_normal.background, Some(Background::Color(Theme::DANGER)));
    assert_eq!(danger_normal.text_color, Theme::WARM_CREAM);

    let danger_hover = Theme::danger_button(button::Status::Hovered);
    assert_eq!(danger_hover.background, Some(Background::Color(Theme::DANGER_HOVER)));
    assert_eq!(danger_hover.text_color, Theme::WARM_CREAM);
}

#[test]
fn test_card_and_modal_container_styles() {
    // Result cards must have Cream background
    let card_style = Theme::card_style();
    assert_eq!(card_style.background, Some(Background::Color(Theme::CARD_CREAM)));
    assert_eq!(card_style.text_color, Some(Theme::CARD_TEXT_PRIMARY));

    let active_card = Theme::active_card_style();
    assert_eq!(active_card.background, Some(Background::Color(Theme::CARD_CREAM_ACTIVE)));
    assert_eq!(active_card.border.color, Theme::PISTACHIO);

    // Modals must have Burgundy background
    let modal_style = Theme::modal_style();
    assert_eq!(modal_style.background, Some(Background::Color(Theme::BURGUNDY_MODAL)));
    assert_eq!(modal_style.border.color, Theme::PISTACHIO);
}

#[test]
fn test_ghost_button_active_state_is_pistachio() {
    // When active (e.g. active tab, active sort pill, active filter), must highlight in Pistachio
    let active_pill = Theme::ghost_button(true, button::Status::Active);
    assert_eq!(active_pill.background, Some(Background::Color(Theme::PISTACHIO)));
    assert_eq!(active_pill.text_color, Theme::PISTACHIO_TEXT_DARK);

    // Inactive ghost button has transparent background
    let inactive_pill = Theme::ghost_button(false, button::Status::Active);
    assert_eq!(inactive_pill.background, None);
    assert_eq!(inactive_pill.text_color, Theme::TEXT_MUTED);
}

#[test]
#[allow(clippy::assertions_on_constants)]
fn test_apple_design_depth_and_elevation_tokens() {
    // 1. Card elevation has directional Y offset and diffused blur radius
    assert!(Theme::ELEVATION_CARD.offset.y > 0.0);
    assert!(Theme::ELEVATION_CARD.blur_radius >= 6.0);
    assert!(Theme::card_style().shadow.blur_radius >= 6.0);

    // 2. Active card elevation has greater lift than resting card
    assert!(Theme::ELEVATION_CARD_ACTIVE.offset.y > Theme::ELEVATION_CARD.offset.y);
    assert!(Theme::ELEVATION_CARD_ACTIVE.blur_radius > Theme::ELEVATION_CARD.blur_radius);

    // 3. Header floating functional layer has elevation
    let header = Theme::header_style();
    assert_eq!(header.background, Some(Background::Color(Theme::BURGUNDY_MODAL)));
    assert!(header.shadow.blur_radius >= 8.0);
    assert!(header.border.radius.top_left >= 10.0);

    // 4. Modal dialogue has the deepest elevation sheet shadow
    let modal = Theme::modal_style();
    assert!(modal.shadow.blur_radius >= 20.0);
    assert!(modal.shadow.offset.y >= 10.0);

    // 5. Inset modal container has high contrast for dark modal background
    let modal_inset = Theme::modal_inset_card_style();
    assert_eq!(modal_inset.background, Some(Background::Color(Theme::BURGUNDY_INSET)));
    assert_eq!(modal_inset.text_color, Some(Theme::WARM_CREAM));

    // 6. Buttons have tactile depth shadows
    let btn_normal = Theme::primary_button(button::Status::Active);
    let btn_hover = Theme::primary_button(button::Status::Hovered);
    let btn_pressed = Theme::primary_button(button::Status::Pressed);

    assert!(btn_hover.shadow.offset.y >= btn_normal.shadow.offset.y);
    assert!(btn_pressed.shadow.offset.y < btn_normal.shadow.offset.y);
}

#[test]
fn test_elegant_themed_scrollbar_styles() {
    let dummy_theme = iced::Theme::Dark;

    // 1. Direction and geometry: slim 8px track, 6px scroller pill, 3px margin
    let direction = Theme::scrollbar_direction();
    assert!(matches!(
        direction,
        iced::widget::scrollable::Direction::Vertical(_)
    ));

    // 2. Resting scrollbar on canvas/cards: transparent track, rounded capsule scroller
    let active_status = iced::widget::scrollable::Status::Active {
        is_horizontal_scrollbar_disabled: true,
        is_vertical_scrollbar_disabled: false,
    };
    let active_style = Theme::scrollable_style(&dummy_theme, active_status);
    assert_eq!(active_style.vertical_rail.background, None, "Resting track must be transparent to avoid ugly gray gutters");
    assert!(active_style.vertical_rail.scroller.border.radius.top_left > 10.0, "Scroller thumb must be a capsule pill");

    // 3. Hovered scrollbar: illuminated pistachio thumb
    let hovered_status = iced::widget::scrollable::Status::Hovered {
        is_horizontal_scrollbar_hovered: false,
        is_vertical_scrollbar_hovered: true,
        is_horizontal_scrollbar_disabled: true,
        is_vertical_scrollbar_disabled: false,
    };
    let hovered_style = Theme::scrollable_style(&dummy_theme, hovered_status);
    assert_eq!(
        hovered_style.vertical_rail.scroller.background,
        Background::Color(Theme::PISTACHIO),
        "Hovered scroller must illuminate with pistachio brand accent"
    );

    // 4. Modal scrollbar: subtle cream capsule for dark modal surfaces
    let modal_style = Theme::modal_scrollable_style(&dummy_theme, active_status);
    assert_eq!(modal_style.vertical_rail.background, None);
    assert!(modal_style.vertical_rail.scroller.border.radius.top_left > 10.0);
}

#[test]
fn test_packaging_tier_color_coded_badges() {
    use sweepx::models::InstallMethod;

    // 1. DNF / APT: Emerald Green
    let (dnf_bg, dnf_fg) = Theme::method_badge_colors(InstallMethod::NativeDnf);
    assert_eq!(dnf_fg, Color::WHITE);
    assert_eq!(dnf_bg, Color::from_rgb8(16, 185, 129));

    // 2. Flatpak: Royal Blue
    let (flatpak_bg, flatpak_fg) = Theme::method_badge_colors(InstallMethod::Flatpak);
    assert_eq!(flatpak_fg, Color::WHITE);
    assert_eq!(flatpak_bg, Color::from_rgb8(37, 99, 235));

    // 3. Snap: Canonical Orange
    let (snap_bg, snap_fg) = Theme::method_badge_colors(InstallMethod::Snap);
    assert_eq!(snap_fg, Color::WHITE);
    assert_eq!(snap_bg, Color::from_rgb8(234, 88, 12));

    // 4. Manual (/opt) & Desktop Entry: Cool Slate Gray
    let (opt_bg, opt_fg) = Theme::method_badge_colors(InstallMethod::ManualOpt);
    assert_eq!(opt_fg, Color::WHITE);
    assert_eq!(opt_bg, Color::from_rgb8(71, 85, 105));

    let (desk_bg, desk_fg) = Theme::method_badge_colors(InstallMethod::CustomDesktop);
    assert_eq!(desk_fg, Color::WHITE);
    assert_eq!(desk_bg, Color::from_rgb8(71, 85, 105));

    // 5. AppImage: Violet
    let (appimage_bg, _) = Theme::method_badge_colors(InstallMethod::AppImage);
    assert_eq!(appimage_bg, Color::from_rgb8(139, 92, 246));

    // 6. Bottles: Wine Magenta
    let (bottles_bg, _) = Theme::method_badge_colors(InstallMethod::Bottles);
    assert_eq!(bottles_bg, Color::from_rgb8(190, 24, 93));
}

#[test]
fn test_header_toggle_button_style_and_contrast() {
    use sweepx::ui::theme::Theme;

    // Active state uses BURGUNDY_INSET background and WARM_CREAM text
    let active_style = Theme::header_toggle_button(button::Status::Active);
    assert_eq!(active_style.background, Some(Background::Color(Theme::BURGUNDY_INSET)));
    assert_eq!(active_style.text_color, Theme::WARM_CREAM);
    assert_eq!(active_style.border.color, Theme::BORDER_COLOR);

    // High-contrast check: WARM_CREAM on BURGUNDY_INSET must exceed WCAG AA (>= 4.5:1)
    let contrast = contrast_ratio(Theme::BURGUNDY_INSET, Theme::WARM_CREAM);
    assert!(
        contrast >= 4.5,
        "Header toggle button contrast ({:.2}) on BURGUNDY_INSET must meet WCAG AA (>= 4.5:1)",
        contrast
    );

    // Pistachio text on BURGUNDY_INSET must also exceed WCAG AA (>= 4.5:1)
    let pistachio_contrast = contrast_ratio(Theme::BURGUNDY_INSET, Theme::PISTACHIO);
    assert!(
        pistachio_contrast >= 4.5,
        "Pistachio accent text contrast ({:.2}) on BURGUNDY_INSET must meet WCAG AA (>= 4.5:1)",
        pistachio_contrast
    );

    // Hovered state highlights border with PISTACHIO
    let hovered_style = Theme::header_toggle_button(button::Status::Hovered);
    assert_eq!(hovered_style.border.color, Theme::PISTACHIO);
    assert_eq!(hovered_style.text_color, Theme::WARM_CREAM);

    // Pressed state highlights border with PISTACHIO_PRESSED
    let pressed_style = Theme::header_toggle_button(button::Status::Pressed);
    assert_eq!(pressed_style.border.color, Theme::PISTACHIO_PRESSED);
}

#[test]
fn test_badge_elements_construct_without_panic() {
    use sweepx::models::InstallMethod;
    use sweepx::ui::theme::Theme;

    // Verify all install methods construct badge elements cleanly
    let methods = [
        InstallMethod::NativeDnf,
        InstallMethod::NativeApt,
        InstallMethod::NativePacman,
        InstallMethod::Flatpak,
        InstallMethod::Snap,
        InstallMethod::AppImage,
        InstallMethod::Bottles,
        InstallMethod::ManualOpt,
        InstallMethod::CustomDesktop,
    ];

    for method in methods {
        let _elem: iced::Element<'_, ()> = Theme::method_badge(method);
    }

    // Direct string labels
    let _custom: iced::Element<'_, ()> = Theme::badge_by_label("Manual (/opt)");
}

