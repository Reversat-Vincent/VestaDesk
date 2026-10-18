//! The Light and Dark colour schemes ([`ColorScheme`]): the named colours that the app draws with.
//!
//! Each colour is named by what it is for, such as [`error`](ColorScheme::error) or
//! [`background`](ColorScheme::background), and [`ColorScheme::LIGHT`] and [`ColorScheme::DARK`]
//! give its value in each scheme, so re-branding the app means changing them only. [`install`]
//! makes egui's widgets draw with the scheme of the current theme, and a screen that draws by
//! itself takes its colours from [`colors`]: no screen writes a colour value.
//!
//! # Backgrounds
//!
//! To keep to a few colours, each scheme has only four backgrounds: the [`bar`](ColorScheme::bar),
//! the page's [`background`](ColorScheme::background), the [`raised`](ColorScheme::raised)
//! background of what stands out from the page, and the [`highlight`](ColorScheme::highlight) of
//! what is under the pointer or selected. In Dark, each one is lighter than the one before; in
//! Light, the page is the lightest and the others get darker in turn. Badges and buttons can also
//! take the [`primary`](ColorScheme::primary), [`secondary`](ColorScheme::secondary) or
//! [`accent`](ColorScheme::accent) colour as background. Their label is then the
//! [`reversed_text`](ColorScheme::reversed_text) on primary and secondary in Light and on accent in
//! Dark, and the [`text`](ColorScheme::text) otherwise.
//!
//! # Lines
//!
//! [`line`](ColorScheme::line) and [`focus`](ColorScheme::focus) compute the colours of lines and
//! of the keyboard focus outline from the other colours, so that they follow them.
//!
//! # Contrast
//!
//! `tests/theme.rs` checks these minimum WCAG contrast ratios of colours on backgrounds:
//!
//! - in both schemes:
//!   - `text` on `bar`, `background`, `raised` and `highlight`: 7:1;
//!   - `weak_text` on `bar`, `background`, `raised` and `highlight`: 4.5:1;
//!   - `success` and `error` on `background`: 4.5:1;
//!   - `success` and `error` on `bar` and `raised`: 3:1;
//!   - `line()` on `bar`, `background` and `raised`: 3:1;
//!   - `focus()` on `bar`, `background`, `raised` and `highlight`: 3:1;
//! - in Light: `reversed_text` on `primary` and `secondary`, and `text` on `accent`: 4.5:1;
//! - in Dark: `text` on `primary` and `secondary`, and `reversed_text` on `accent`: 4.5:1.

use egui::style::Widgets;
use egui::{Color32, Context, Rgba, Theme, Ui, Visuals};

/// The contrast ratio that lines and the keyboard focus outline have, at least, on the backgrounds next to them.
const LINE_CONTRAST: f32 = 3.0;

/// The colours of a colour scheme, each named by what it is for.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ColorScheme {
    /// The app's own colour, for links and the background of badges and buttons, and the base of the keyboard focus outline.
    pub primary: Color32,
    /// The app's second colour, for the background of badges and buttons set apart from the primary ones.
    pub secondary: Color32,
    /// Attention, for the background of badges and buttons that must catch the eye, such as warnings.
    pub accent: Color32,
    /// Success, for the text and icons of messages about an operation that worked.
    pub success: Color32,
    /// Errors, for the text and icons of messages about an operation that failed.
    pub error: Color32,
    /// The background of the activity bar, the status bar, text fields and the tracks of scroll bars.
    pub bar: Color32,
    /// The background of the page, in the central area.
    pub background: Color32,
    /// The background of what stands out from the page: controls, code, tooltips, menus and windows.
    pub raised: Color32,
    /// The background of a control under the pointer, pressed or open, and of selected items and text.
    pub highlight: Color32,
    /// Text, on the four backgrounds, and labels on accent in Light and on primary and secondary in Dark.
    pub text: Color32,
    /// Dimmed text, such as the hint of an empty text field.
    pub weak_text: Color32,
    /// The other scheme's text colour, for the labels on primary and secondary in Light and on accent in Dark.
    pub reversed_text: Color32,
}

impl ColorScheme {
    /// The Light colour scheme: dark text on light backgrounds.
    pub const LIGHT: Self = Self {
        primary: Color32::from_rgb(0x3F, 0x51, 0xB5),
        secondary: Color32::from_rgb(0x67, 0x3A, 0xB7),
        accent: Color32::from_rgb(0xFF, 0xC1, 0x07),
        success: Color32::from_rgb(0x2E, 0x7D, 0x32),
        error: Color32::from_rgb(0xC6, 0x28, 0x28),
        bar: Color32::from_rgb(0xE7, 0xEA, 0xED),
        background: Color32::from_rgb(0xF7, 0xF8, 0xFB),
        raised: Color32::from_rgb(0xD2, 0xD5, 0xD8),
        highlight: Color32::from_rgb(0xC0, 0xC3, 0xC6),
        text: Color32::from_gray(0x33),
        weak_text: Color32::from_gray(0x4F),
        reversed_text: Color32::from_gray(0xDE),
    };

    /// The Dark colour scheme: light text on dark backgrounds.
    pub const DARK: Self = Self {
        primary: Color32::from_rgb(0x3F, 0x51, 0xB5),
        secondary: Color32::from_rgb(0x67, 0x3A, 0xB7),
        accent: Color32::from_rgb(0xFF, 0xC1, 0x07),
        success: Color32::from_rgb(0x81, 0xC7, 0x84),
        error: Color32::from_rgb(0xE5, 0x73, 0x73),
        bar: Color32::from_rgb(0x1E, 0x21, 0x24),
        background: Color32::from_rgb(0x28, 0x2B, 0x30),
        raised: Color32::from_rgb(0x36, 0x39, 0x3E),
        highlight: Color32::from_rgb(0x42, 0x45, 0x49),
        text: Color32::from_gray(0xDE),
        weak_text: Color32::from_gray(0xB3),
        reversed_text: Color32::from_gray(0x33),
    };

    /// Returns the colour scheme of `theme`.
    ///
    /// # Examples
    ///
    /// ```
    /// use egui::Theme;
    /// use vesta_desk::theme::ColorScheme;
    ///
    /// assert_eq!(ColorScheme::of(Theme::Dark), &ColorScheme::DARK);
    /// ```
    #[must_use]
    pub fn of(theme: Theme) -> &'static Self {
        match theme {
            Theme::Light => &Self::LIGHT,
            Theme::Dark => &Self::DARK,
        }
    }

    /// Returns the colour of lines: the outlines of controls, tooltips, menus and windows, and the lines between areas.
    ///
    /// It starts from the background closest to the text and moves towards the text until it has a
    /// contrast ratio of at least 3:1 on the bar, the background and raised surfaces.
    ///
    /// # Examples
    ///
    /// ```
    /// use vesta_desk::theme::{self, ColorScheme};
    ///
    /// let colors = ColorScheme::LIGHT;
    /// assert!(theme::contrast(colors.line(), colors.raised) >= 3.0);
    /// ```
    #[must_use]
    pub fn line(&self) -> Color32 {
        let backgrounds = [self.bar, self.background, self.raised];
        // The background closest to the text is the one that a line stands out from the least.
        let closest = backgrounds
            .into_iter()
            .min_by(|a, b| contrast(*a, self.text).total_cmp(&contrast(*b, self.text)))
            .unwrap_or(self.background);
        towards(closest, self.text, &backgrounds)
    }

    /// Returns the colour of the keyboard focus outline.
    ///
    /// It starts from the primary colour and moves towards the text until it has a contrast ratio
    /// of at least 3:1 on the four backgrounds. Text fields are outlined in the text colour instead.
    ///
    /// # Examples
    ///
    /// ```
    /// use vesta_desk::theme::{self, ColorScheme};
    ///
    /// let colors = ColorScheme::DARK;
    /// assert!(theme::contrast(colors.focus(), colors.highlight) >= 3.0);
    /// ```
    #[must_use]
    pub fn focus(&self) -> Color32 {
        let backgrounds = [self.bar, self.background, self.raised, self.highlight];
        towards(self.primary, self.text, &backgrounds)
    }
}

/// Returns the colour scheme of the theme that `ui` is drawn in.
///
/// # Examples
///
/// ```
/// use egui::{Context, RawInput, Theme};
/// use vesta_desk::theme::{self, ColorScheme};
///
/// let ctx = Context::default();
/// ctx.set_theme(Theme::Light);
/// ctx.run_ui(RawInput::default(), |ui| assert_eq!(theme::colors(ui), &ColorScheme::LIGHT))
///     .drop_without_applying_deltas();
/// ```
#[must_use]
pub fn colors(ui: &Ui) -> &'static ColorScheme {
    ColorScheme::of(ui.ctx().theme())
}

/// Returns the WCAG contrast ratio of two opaque colours, from 1 for two equal colours to 21 for black and white.
///
/// # Examples
///
/// ```
/// use egui::Color32;
/// use vesta_desk::theme;
///
/// assert!((theme::contrast(Color32::BLACK, Color32::WHITE) - 21.0).abs() < 0.01);
/// ```
#[must_use]
pub fn contrast(a: Color32, b: Color32) -> f32 {
    let luminance = |color: Color32| {
        let linear = Rgba::from(color);
        0.2126 * linear.r() + 0.7152 * linear.g() + 0.0722 * linear.b()
    };
    let (a, b) = (luminance(a), luminance(b));
    (a.max(b) + 0.05) / (a.min(b) + 0.05)
}

/// Makes egui's widgets draw with the Light colour scheme in the Light theme, and with the Dark one in the Dark theme.
pub fn install(ctx: &Context) {
    for theme in [Theme::Light, Theme::Dark] {
        ctx.set_visuals_of(theme, visuals(theme));
    }
}

/// Returns egui's visuals for `theme`, in the colours of its colour scheme except for egui's own shadows and warning colour.
fn visuals(theme: Theme) -> Visuals {
    let colors = ColorScheme::of(theme);
    let line = colors.line();
    let mut visuals = theme.default_visuals();
    let Widgets {
        noninteractive,
        inactive,
        hovered,
        active,
        open,
    } = &mut visuals.widgets;
    // egui outlines the control that has the keyboard focus with the line of the `active` state.
    for (state, fill, outline) in [
        (noninteractive, colors.background, line),
        (inactive, colors.raised, line),
        (hovered, colors.highlight, line),
        (active, colors.highlight, colors.focus()),
        (open, colors.highlight, line),
    ] {
        state.bg_fill = fill;
        state.weak_bg_fill = fill;
        state.bg_stroke.color = outline;
        state.fg_stroke.color = colors.text;
    }
    visuals.weak_text_color = Some(colors.weak_text);
    visuals.selection.bg_fill = colors.highlight;
    // egui also outlines the text field that has the keyboard focus with this colour.
    visuals.selection.stroke.color = colors.text;
    visuals.ime_composition.active_underline_stroke.color = colors.text;
    visuals.ime_composition.inactive_underline_stroke.color = colors.weak_text;
    visuals.hyperlink_color = colors.primary;
    visuals.faint_bg_color = colors.bar;
    visuals.extreme_bg_color = colors.bar;
    visuals.code_bg_color = colors.raised;
    visuals.error_fg_color = colors.error;
    visuals.window_fill = colors.raised;
    visuals.window_stroke.color = line;
    visuals.panel_fill = colors.bar;
    visuals.text_cursor.stroke.color = colors.text;
    visuals
}

/// Returns the first colour from `from` towards `to` with a contrast ratio of at least [`LINE_CONTRAST`] on all of `backgrounds`.
///
/// Returns `to` when none has it.
fn towards(from: Color32, to: Color32, backgrounds: &[Color32]) -> Color32 {
    (0..=u8::MAX)
        .map(|step| from.lerp_to_gamma(to, f32::from(step) / f32::from(u8::MAX)))
        .find(|&color| {
            backgrounds
                .iter()
                .all(|&background| contrast(color, background) >= LINE_CONTRAST)
        })
        .unwrap_or(to)
}
