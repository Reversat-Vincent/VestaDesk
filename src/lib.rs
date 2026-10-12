//! VestaDesk — a native desktop super-app in Rust, built with egui/eframe for Windows and Linux.
//!
//! All application logic lives in this library crate so that `cargo doc` and the test suite
//! cover it; the `vesta-desk` binary only calls [`run`].
//!
//! # Architecture
//!
//! [`app`] opens the main window and owns the single state container. UI code reads [`state`]
//! and pushes [`action`]s; the app applies them, routes between [`pages`] via [`router`], and
//! persists preferences through [`storage`] and [`settings`].
//!
//! Around the pages, [`shell`] draws the activity bar and status bar, [`toast`] shows
//! notifications, and [`commands`] triggers actions from the command palette and keyboard
//! shortcuts. [`i18n`], [`fonts`] and [`theme`] provide the translations, fonts and colours.
//! [`error`] defines the expected failures and how each is handled, [`logging`] records
//! diagnostics, and [`platform`] holds the Windows and Linux specifics.

pub mod action;
pub mod app;
pub mod commands;
pub mod error;
pub mod fonts;
pub mod i18n;
pub mod logging;
pub mod pages;
pub mod platform;
pub mod router;
pub mod settings;
pub mod shell;
pub mod state;
pub mod storage;
pub mod theme;
pub mod toast;

use std::process::ExitCode;

/// Starts VestaDesk and returns the process exit code.
///
/// The logger is installed first. A failure that prevents the app from starting is logged and
/// gives a failure exit code instead of a crash.
#[must_use]
pub fn run() -> ExitCode {
    logging::init();
    error::exit_code(start())
}

/// Starts the app and runs it until its window is closed.
///
/// If the settings directory cannot be found or created, the app logs a warning and keeps
/// working with temporary settings. Otherwise, it removes the temporary files that interrupted
/// saves left there.
fn start() -> error::Result<()> {
    let settings_dir = storage::settings_dir()
        .and_then(storage::ensure_dir)
        .inspect_err(error::AppError::report)
        .ok();
    if let Some(dir) = &settings_dir
        && let Err(error) = storage::remove_temp_files(dir)
    {
        error.report();
    }
    logging::log_startup(settings_dir.as_deref());
    app::run()
}
