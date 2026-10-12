//! Typed errors ([`AppError`]) and the crate-wide [`Result`] alias.
//!
//! Every expected failure has its own [`AppError`] variant with a clear description, so the
//! app never has to crash on one. Each variant has a [`Severity`].

use std::io;
use std::path::PathBuf;
use std::process::ExitCode;

/// Crate-wide result type.
pub type Result<T> = std::result::Result<T, AppError>;

/// How serious a failure is, which decides what the app does about it.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Severity {
    /// The app cannot start or continue: it logs the error and exits with an error status.
    Fatal,
    /// The app logs the error and keeps working with a safe fallback; once notifications exist
    /// (VESTA-73), it also tells the user.
    Recoverable,
}

/// An expected failure of the app.
#[derive(Debug, thiserror::Error)]
pub enum AppError {
    /// The operating system gave no per-user configuration directory: the app keeps the settings in memory.
    #[error("the system provides no configuration directory")]
    ConfigDirUnavailable,

    /// The app could not finish starting (window, graphics or another system part).
    #[error("the application could not start: {0}")]
    Startup(String),

    /// A logger was already installed when the app tried to install its own: the app keeps the existing one.
    #[error("a logger is already installed; keeping it")]
    LoggerAlreadyInstalled,

    /// A file or folder could not be created, read, written, moved or removed: the app keeps the settings in memory.
    #[error("cannot access {}: {source}", path.display())]
    StorageIo {
        /// The file or directory involved.
        path: PathBuf,
        /// The underlying system error.
        source: io::Error,
    },

    /// A folder could not be written to disk after a save in it: the file is saved, but a power cut could undo the save.
    #[error(
        "cannot sync the folder {}: {source}; a power cut could undo the last save in it",
        path.display()
    )]
    DirNotSynced {
        /// The folder.
        path: PathBuf,
        /// The underlying system error.
        source: io::Error,
    },

    /// A settings file exists but its content cannot be understood: the app starts from the
    /// default settings.
    #[error("the settings file {} is corrupt: {source}", path.display())]
    SettingsCorrupt {
        /// The settings file.
        path: PathBuf,
        /// The underlying parse error.
        source: serde_json::Error,
    },

    /// A font could not be found or loaded: the app falls back to the bundled fonts.
    #[error("the font \"{0}\" is unavailable")]
    FontUnavailable(String),

    /// A language code is not one of the supported languages: the app falls back to English.
    #[error("the language \"{0}\" is not supported")]
    UnsupportedLocale(String),
}

impl AppError {
    /// Returns how serious this failure is.
    ///
    /// # Examples
    ///
    /// ```
    /// use vesta_desk::error::{AppError, Severity};
    ///
    /// assert_eq!(AppError::Startup("no window".into()).severity(), Severity::Fatal);
    /// assert_eq!(AppError::UnsupportedLocale("xx".into()).severity(), Severity::Recoverable);
    /// ```
    #[must_use]
    pub fn severity(&self) -> Severity {
        match self {
            Self::Startup(_) => Severity::Fatal,
            Self::ConfigDirUnavailable
            | Self::LoggerAlreadyInstalled
            | Self::StorageIo { .. }
            | Self::DirNotSynced { .. }
            | Self::SettingsCorrupt { .. }
            | Self::FontUnavailable(_)
            | Self::UnsupportedLocale(_) => Severity::Recoverable,
        }
    }

    /// Logs the error: at `error` level when fatal, at `warn` level when recoverable.
    pub fn report(&self) {
        match self.severity() {
            Severity::Fatal => log::error!("{self}"),
            Severity::Recoverable => log::warn!("{self}"),
        }
    }
}

/// Returns the process exit code for the outcome of starting the app, and logs the error if
/// starting failed.
///
/// # Examples
///
/// ```
/// use std::process::ExitCode;
///
/// use vesta_desk::error::{AppError, exit_code};
///
/// assert_eq!(exit_code(Ok(())), ExitCode::SUCCESS);
/// assert_eq!(exit_code(Err(AppError::Startup("no window".into()))), ExitCode::FAILURE);
/// ```
#[must_use]
pub fn exit_code(outcome: Result<()>) -> ExitCode {
    match outcome {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            error.report();
            ExitCode::FAILURE
        }
    }
}
