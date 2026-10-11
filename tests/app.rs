//! The main window has the app's title and a minimum size, the app starts from the default data,
//! and a window that cannot be created is reported as an error.

use vesta_desk::app::{App, MIN_WINDOW_SIZE, WINDOW_SIZE, native_options};
use vesta_desk::state::AppState;

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
    assert_eq!(App::default().state(), &AppState::default());
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
