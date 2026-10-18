//! The main window has the app's title and a minimum size, the app starts from the default data,
//! draws with the colour schemes and applies the actions sent to it on the next frame, and a window
//! that cannot be created is reported as an error.

use eframe::App as _;
use egui::{Context, RawInput, Theme};
use vesta_desk::action::AppAction;
use vesta_desk::app::{App, MIN_WINDOW_SIZE, WINDOW_SIZE, native_options};
use vesta_desk::router::Route;
use vesta_desk::state::AppState;
use vesta_desk::theme::ColorScheme;

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

/// An action sent to the app takes effect on the next frame, which eframe's test frame runs without a window.
#[test]
fn the_app_applies_an_action_sent_to_it_on_the_next_frame() {
    let ctx = Context::default();
    let mut app = App::new(ctx.clone());
    app.sender()
        .send(AppAction::Navigate(Route::About))
        .unwrap();
    let mut frame = eframe::Frame::_new_kittest();
    ctx.run_ui(RawInput::default(), |ui| {
        app.logic(ui.ctx(), &mut frame);
        app.ui(ui, &mut frame);
    })
    .drop_without_applying_deltas();
    assert_eq!(app.state().route(), Route::About);
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
