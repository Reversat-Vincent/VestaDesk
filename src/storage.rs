//! Persistence: OS config directory resolution, atomic `.tmp` + rename writes and JSON stores.

use std::path::PathBuf;

use crate::error::{Result, VestaDeskError};

/// Name of VestaDesk's folder inside the user's configuration directory.
const APP_DIR: &str = "VestaDesk";

/// Returns the directory where VestaDesk stores its settings: the `VestaDesk` folder inside the
/// user's configuration directory (the application-data folder on Windows, `$XDG_CONFIG_HOME` or
/// `~/.config` on Linux).
///
/// # Errors
///
/// Returns [`VestaDeskError::ConfigDirUnavailable`] if the system provides no configuration
/// directory.
pub fn settings_dir() -> Result<PathBuf> {
    settings_dir_in(dirs::config_dir())
}

/// Returns VestaDesk's settings directory inside `config_dir`, the user's configuration directory.
///
/// # Errors
///
/// Returns [`VestaDeskError::ConfigDirUnavailable`] if `config_dir` is `None`.
///
/// # Examples
///
/// ```
/// use std::path::{Path, PathBuf};
///
/// use vesta_desk::storage::settings_dir_in;
///
/// let dir = settings_dir_in(Some(PathBuf::from("config")))?;
/// assert_eq!(dir, Path::new("config/VestaDesk"));
/// # Ok::<(), vesta_desk::error::VestaDeskError>(())
/// ```
pub fn settings_dir_in(config_dir: Option<PathBuf>) -> Result<PathBuf> {
    config_dir
        .map(|dir| dir.join(APP_DIR))
        .ok_or(VestaDeskError::ConfigDirUnavailable)
}
