//! VestaDesk — a native desktop super-app in Rust, built with egui/eframe for Windows and Linux.
//!
//! All application logic lives in this library crate so that `cargo doc` and the test suite
//! cover it; the `vesta-desk` binary only calls [`run`].
//!
//! # Architecture
//!
//! [`app`] owns the single state container. UI code reads [`state`] and pushes [`action`]s;
//! the app applies them, routes between [`pages`] via [`router`], and persists preferences
//! through [`storage`] and [`settings`].
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

/// Starts the app.
///
/// Placeholder: the eframe bootstrap (VESTA-53) will open the main window here.
fn start() -> error::Result<()> {
    logging::log_startup(&storage::settings_dir()?);
    Ok(())
}
