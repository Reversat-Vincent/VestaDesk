//! The main window has the app's title and a minimum size, the app starts from the default data,
//! draws with the bundled fonts and the colour schemes, applies the actions sent to it on the next
//! frame and asks for a frame when the next notification goes, and a window that cannot be created
//! is reported as an error.

use std::collections::BTreeSet;
use std::time::Duration;

use eframe::App as _;
use egui::{Context, FontDefinitions, FontFamily, RawInput, Theme, ViewportId};
use vesta_desk::action::AppAction;
use vesta_desk::app::{App, MIN_WINDOW_SIZE, WINDOW_SIZE, native_options};
use vesta_desk::fonts::BUNDLED;
use vesta_desk::i18n::Text;
use vesta_desk::notifications::{DEFAULT_DELAY, Level, Notification};
use vesta_desk::router::Route;
use vesta_desk::state::AppState;
use vesta_desk::theme::ColorScheme;

/// Runs a frame of `app` without a window, as eframe's test frame allows, and returns the delay after which it asks for the next one.
fn run_frame(ctx: &Context, app: &mut App) -> Duration {
    let mut frame = eframe::Frame::_new_kittest();
    let output = ctx.run_ui(RawInput::default(), |ui| {
        app.logic(ui.ctx(), &mut frame);
        app.ui(ui, &mut frame);
    });
    let delay = output.viewport_output[&ViewportId::ROOT].repaint_delay;
    output.drop_without_applying_deltas();
    delay
}

#[test]
fn the_window_has_the_app_title_and_a_minimum_size() {
    let window = native_options().viewport;
    assert_eq!(window.title.as_deref(), Some("VestaDesk"));
    assert_eq!(window.inner_size, Some(WINDOW_SIZE));
    assert_eq!(window.min_inner_size, Some(MIN_WINDOW_SIZE));
}

/// Creating the app opens no window, so tests can check its data.
#[test]
fn the_app_starts_from_the_default_data() {
    assert_eq!(App::new(Context::default()).state(), &AppState::default());
}

/// The app has no font but the bundled ones, which all its text tries in their order: egui's
/// built-in fonts, with their emoji, are left out of it. egui reads every font at the start of a
/// frame, and panics on one that it cannot read.
#[test]
fn the_app_draws_text_with_the_bundled_fonts_only() {
    assert!(
        FontDefinitions::builtin_font_names().is_empty(),
        "egui's built-in fonts are in the app"
    );
    let ctx = Context::default();
    let _app = App::new(ctx.clone());
    ctx.run_ui(RawInput::default(), |_| {})
        .drop_without_applying_deltas();
    let held = ctx.fonts(|fonts| fonts.definitions().clone());
    let bundled: Vec<String> = BUNDLED.iter().map(|font| font.name.to_owned()).collect();
    let names: BTreeSet<&String> = held.font_data.keys().collect();
    assert_eq!(names, bundled.iter().collect());
    for family in [FontFamily::Proportional, FontFamily::Monospace] {
        assert_eq!(held.families.get(&family), Some(&bundled), "{family:?}");
    }
}

/// In both themes, egui's widgets draw with that theme's colour scheme, its computed lines and focus outline included.
#[test]
fn the_app_draws_each_theme_with_its_colour_scheme() {
    let ctx = Context::default();
    let _app = App::new(ctx.clone());
    for theme in [Theme::Light, Theme::Dark] {
        let colors = ColorScheme::of(theme);
        let style = ctx.style_of(theme);
        assert_eq!(style.visuals.panel_fill, colors.bar, "{theme:?}");
        assert_eq!(style.visuals.text_color(), colors.text, "{theme:?}");
        let line = style.visuals.window_stroke.color;
        assert_eq!(line, colors.line(), "{theme:?}");
        let focus = style.visuals.widgets.active.bg_stroke.color;
        assert_eq!(focus, colors.focus(), "{theme:?}");
    }
}

/// An action sent to the app takes effect on the next frame.
#[test]
fn the_app_applies_an_action_sent_to_it_on_the_next_frame() {
    let ctx = Context::default();
    let mut app = App::new(ctx.clone());
    app.sender()
        .send(AppAction::Navigate(Route::About))
        .unwrap();
    run_frame(&ctx, &mut app);
    assert_eq!(app.state().route(), Route::About);
}

/// egui draws no frame without input, so the app asks for one when a notification's time is up, for it to go even if the user does nothing.
#[test]
fn the_app_asks_for_a_frame_when_the_next_notification_goes() {
    let ctx = Context::default();
    let mut app = App::new(ctx.clone());
    let saved = Notification {
        level: Level::Success,
        text: Text::Settings,
        stays_until_closed: false,
    };
    app.sender().send(AppAction::Notify(saved)).unwrap();
    // egui asks for a few frames at once as it starts and when an action is sent; the next frame asked for is the app's.
    let delay = (0..5)
        .map(|_| run_frame(&ctx, &mut app))
        .find(|delay| !delay.is_zero());
    // egui wakes the app slightly early, so as not to be late.
    let earliest = DEFAULT_DELAY.saturating_sub(Duration::from_secs(1));
    assert!(
        delay.is_some_and(|delay| earliest < delay && delay <= DEFAULT_DELAY),
        "{delay:?}"
    );
}

/// Without a display the window cannot be created: the app logs the error and exits with a failure status instead of crashing.
#[cfg(target_os = "linux")]
#[test]
fn without_a_display_the_app_reports_the_error_and_fails() {
    let settings_dir = tempfile::tempdir().unwrap();
    let output = std::process::Command::new(env!("CARGO_BIN_EXE_vesta-desk"))
        .env_remove("DISPLAY")
        .env_remove("WAYLAND_DISPLAY")
        .env_remove("WAYLAND_SOCKET")
        .env_remove("RUST_LOG")
        .env("VESTADESK_SETTINGS_DIR", settings_dir.path())
        .output()
        .unwrap();
    let log = String::from_utf8_lossy(&output.stderr);
    assert_eq!(output.status.code(), Some(1), "{log}");
    assert!(log.contains("the application could not start"), "{log}");
}
