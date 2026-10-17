//! The main window has the app's title and a minimum size, the app starts from the default data and
//! applies the actions sent to it on the next frame, and a window that cannot be created is
//! reported as an error. The app reopens on the page that was open when it closed, and never
//! overwrites a settings file that it cannot read.

use std::fs;

use eframe::App as _;
use egui::{Context, RawInput};
use vesta_desk::action::AppAction;
use vesta_desk::app::{App, MIN_WINDOW_SIZE, WINDOW_SIZE, native_options};
use vesta_desk::router::Route;
use vesta_desk::state::AppState;
use vesta_desk::storage::MAX_DOCUMENT_SIZE;

/// Runs one frame of `app`, whose window context is `ctx`, as eframe does but without a window.
fn run_frame(ctx: &Context, app: &mut App) {
    let mut frame = eframe::Frame::_new_kittest();
    ctx.run_ui(RawInput::default(), |ui| {
        app.logic(ui.ctx(), &mut frame);
        app.ui(ui, &mut frame);
    })
    .drop_without_applying_deltas();
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
    assert_eq!(
        App::new(Context::default(), None).state(),
        &AppState::default()
    );
}

/// An action sent to the app takes effect on the next frame, which eframe's test frame runs without a window.
#[test]
fn the_app_applies_an_action_sent_to_it_on_the_next_frame() {
    let ctx = Context::default();
    let mut app = App::new(ctx.clone(), None);
    app.sender()
        .send(AppAction::Navigate(Route::About))
        .unwrap();
    run_frame(&ctx, &mut app);
    assert_eq!(app.state().route(), Route::About);
}

/// Closing the app right after a change still saves it, so the next start opens the same page.
#[test]
fn the_app_reopens_on_the_page_that_was_open_when_it_closed() {
    let settings_dir = tempfile::tempdir().unwrap();
    let ctx = Context::default();
    let mut app = App::new(ctx.clone(), Some(settings_dir.path().to_owned()));
    app.sender()
        .send(AppAction::Navigate(Route::About))
        .unwrap();
    run_frame(&ctx, &mut app);
    drop(app);
    let reopened = App::new(Context::default(), Some(settings_dir.path().to_owned()));
    assert_eq!(reopened.state().route(), Route::About);
}

/// The app starts from the default settings and keeps its changes in memory, so that the user can still repair the file, whether it is corrupt or too large.
#[test]
fn a_settings_file_that_cannot_be_read_is_never_overwritten() {
    let too_large = " ".repeat(usize::try_from(MAX_DOCUMENT_SIZE).unwrap() + 1);
    for (case, contents) in [("corrupt", "not json"), ("too large", too_large.as_str())] {
        let settings_dir = tempfile::tempdir().unwrap();
        let file = settings_dir.path().join("settings.json");
        fs::write(&file, contents).unwrap();
        let ctx = Context::default();
        let mut app = App::new(ctx.clone(), Some(settings_dir.path().to_owned()));
        assert_eq!(app.state(), &AppState::default(), "{case}");
        app.sender()
            .send(AppAction::Navigate(Route::About))
            .unwrap();
        run_frame(&ctx, &mut app);
        assert_eq!(app.state().route(), Route::About, "{case}");
        drop(app);
        assert!(
            fs::read_to_string(&file).unwrap() == contents,
            "the {case} file was overwritten"
        );
    }
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
