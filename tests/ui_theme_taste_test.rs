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
