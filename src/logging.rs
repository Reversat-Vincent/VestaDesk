//! Diagnostic logging: installs the `log` back end and writes the startup summary.
//!
//! By default the output is quiet: warnings and errors from every crate, plus VestaDesk's own
//! information messages (the startup summary). Set the `RUST_LOG` environment variable to see
//! more, for example `RUST_LOG=debug`; it uses the filter syntax of `env_logger`.

use std::env;
use std::path::Path;

use env_logger::{Builder, DEFAULT_FILTER_ENV};

use crate::error::AppError;

/// The filter used when `RUST_LOG` is not set.
const DEFAULT_FILTER: &str = "warn,vesta_desk=info";

/// Returns the logger configuration for the filter `spec`, written in the `RUST_LOG` syntax, or
/// for the default filter if `spec` is `None`.
///
/// # Examples
///
/// ```
/// use log::{Level, Log, Metadata};
/// use vesta_desk::logging;
///
/// let debug = Metadata::builder().level(Level::Debug).target("vesta_desk").build();
/// assert!(!logging::builder(None).build().enabled(&debug));
/// assert!(logging::builder(Some("debug")).build().enabled(&debug));
/// ```
#[must_use]
pub fn builder(spec: Option<&str>) -> Builder {
    let mut builder = Builder::new();
    builder.parse_filters(spec.unwrap_or(DEFAULT_FILTER));
    builder
}

/// Installs the logger, filtered by the `RUST_LOG` environment variable if it is set.
///
/// If a logger is already installed, it is kept and [`AppError::LoggerAlreadyInstalled`] is reported.
pub fn init() {
    let spec = env::var(DEFAULT_FILTER_ENV).ok();
    match builder(spec.as_deref()).try_init() {
        Ok(()) => log::debug!("log filter: {}", spec.as_deref().unwrap_or(DEFAULT_FILTER)),
        Err(_) => AppError::LoggerAlreadyInstalled.report(),
    }
}

/// Logs, at `info` level, the app version, the operating system and the settings directory, or
/// that the settings are temporary if there is no settings directory.
pub fn log_startup(settings_dir: Option<&Path>) {
    let settings = match settings_dir {
        Some(dir) => format!("settings in {}", dir.display()),
        None => "temporary settings, not saved".to_owned(),
    };
    log::info!(
        "VestaDesk {} on {} {}; {settings}",
        env!("CARGO_PKG_VERSION"),
        env::consts::OS,
        env::consts::ARCH,
    );
}
