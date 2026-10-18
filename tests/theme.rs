//! Each colour has at least the WCAG contrast ratio that the `theme` module docs list for it on each background.

use egui::{Color32, Theme};
use vesta_desk::theme::{self, ColorScheme};

/// The themes, each with its colour scheme.
const THEMES: [Theme; 2] = [Theme::Light, Theme::Dark];

/// Returns the bar, background, raised and highlight colours of `colors`, with their names.
fn four_backgrounds(colors: &ColorScheme) -> [(&'static str, Color32); 4] {
    [
        ("bar", colors.bar),
        ("background", colors.background),
        ("raised", colors.raised),
        ("highlight", colors.highlight),
    ]
}

/// Checks that the named colour of `theme` has a contrast ratio of at least `minimum` on each of the named `backgrounds`.
fn assert_contrast(
    theme: Theme,
    (name, color): (&str, Color32),
    backgrounds: &[(&str, Color32)],
    minimum: f32,
) {
    for &(background, fill) in backgrounds {
        let ratio = theme::contrast(color, fill);
        assert!(
            ratio >= minimum,
            "{name} on {background} in {theme:?}: {ratio:.2}:1, under {minimum}:1"
        );
    }
}

/// Black on white is 21:1, a colour on itself 1:1, and #767676, the lightest grey that text on white can use, 4.54:1.
#[test]
fn the_contrast_ratio_matches_known_values() {
    assert!((theme::contrast(Color32::BLACK, Color32::WHITE) - 21.0).abs() < 0.01);
    assert!((theme::contrast(Color32::WHITE, Color32::WHITE) - 1.0).abs() < 0.01);
    assert!((theme::contrast(Color32::from_gray(0x76), Color32::WHITE) - 4.54).abs() < 0.01);
}

#[test]
fn text_on_the_four_backgrounds_is_at_least_7_to_1() {
    for theme in THEMES {
        let colors = ColorScheme::of(theme);
        assert_contrast(theme, ("text", colors.text), &four_backgrounds(colors), 7.0);
    }
}

#[test]
fn weak_text_on_the_four_backgrounds_is_at_least_4_5_to_1() {
    for theme in THEMES {
        let colors = ColorScheme::of(theme);
        assert_contrast(
            theme,
            ("weak_text", colors.weak_text),
            &four_backgrounds(colors),
            4.5,
        );
    }
}

#[test]
fn success_and_error_on_background_are_at_least_4_5_to_1() {
    for theme in THEMES {
        let colors = ColorScheme::of(theme);
        for text in [("success", colors.success), ("error", colors.error)] {
            assert_contrast(theme, text, &[("background", colors.background)], 4.5);
        }
    }
}

#[test]
fn success_and_error_on_bar_and_raised_are_at_least_3_to_1() {
    for theme in THEMES {
        let colors = ColorScheme::of(theme);
        for text in [("success", colors.success), ("error", colors.error)] {
            assert_contrast(
                theme,
                text,
                &[("bar", colors.bar), ("raised", colors.raised)],
                3.0,
            );
        }
    }
}

#[test]
fn line_on_bar_background_and_raised_is_at_least_3_to_1() {
    for theme in THEMES {
        let colors = ColorScheme::of(theme);
        let backgrounds = [
            ("bar", colors.bar),
            ("background", colors.background),
            ("raised", colors.raised),
        ];
        assert_contrast(theme, ("line()", colors.line()), &backgrounds, 3.0);
    }
}

#[test]
fn focus_on_the_four_backgrounds_is_at_least_3_to_1() {
    for theme in THEMES {
        let colors = ColorScheme::of(theme);
        assert_contrast(
            theme,
            ("focus()", colors.focus()),
            &four_backgrounds(colors),
            3.0,
        );
    }
}

#[test]
fn in_light_reversed_text_on_primary_and_secondary_and_text_on_accent_are_at_least_4_5_to_1() {
    let theme = Theme::Light;
    let colors = ColorScheme::of(theme);
    let primary_and_secondary = [("primary", colors.primary), ("secondary", colors.secondary)];
    assert_contrast(
        theme,
        ("reversed_text", colors.reversed_text),
        &primary_and_secondary,
        4.5,
    );
    assert_contrast(
        theme,
        ("text", colors.text),
        &[("accent", colors.accent)],
        4.5,
    );
}

#[test]
fn in_dark_text_on_primary_and_secondary_and_reversed_text_on_accent_are_at_least_4_5_to_1() {
    let theme = Theme::Dark;
    let colors = ColorScheme::of(theme);
    let primary_and_secondary = [("primary", colors.primary), ("secondary", colors.secondary)];
    assert_contrast(theme, ("text", colors.text), &primary_and_secondary, 4.5);
    assert_contrast(
        theme,
        ("reversed_text", colors.reversed_text),
        &[("accent", colors.accent)],
        4.5,
    );
}
