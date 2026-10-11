//! Persistence: OS config directory resolution, atomic `.tmp` + rename writes and JSON stores.

use std::env;
use std::fs;
use std::path::PathBuf;

use crate::error::{Result, VestaDeskError};

/// Name of VestaDesk's folder inside the user's configuration directory.
const APP_DIR: &str = "VestaDesk";

/// Environment variable naming a folder to use instead of the default settings directory.
const SETTINGS_DIR_ENV: &str = "VESTADESK_SETTINGS_DIR";

/// Returns the directory where VestaDesk stores its settings, chosen by [`settings_dir_from`] from
/// the `VESTADESK_SETTINGS_DIR` environment variable and the user's configuration directory (the
/// application-data folder on Windows, `$XDG_CONFIG_HOME` or `~/.config` on Linux).
///
/// # Errors
///
/// Returns [`VestaDeskError::ConfigDirUnavailable`] if the variable is unset or empty and the
/// system provides no configuration directory.
pub fn settings_dir() -> Result<PathBuf> {
    settings_dir_from(
        env::var_os(SETTINGS_DIR_ENV).map(PathBuf::from),
        dirs::config_dir(),
    )
}

/// Returns the settings directory: `override_dir` if it is a non-empty path, otherwise the
/// `VestaDesk` folder inside `config_dir`, the user's configuration directory.
///
/// # Errors
///
/// Returns [`VestaDeskError::ConfigDirUnavailable`] if there is neither an override nor a
/// configuration directory.
///
/// # Examples
///
/// ```
/// use std::path::{Path, PathBuf};
///
/// use vesta_desk::storage::settings_dir_from;
///
/// let config_dir = PathBuf::from("config");
///
/// // No override: the `VestaDesk` folder in the configuration directory.
/// let settings_dir = settings_dir_from(None, Some(config_dir.clone()))?;
/// assert_eq!(settings_dir, Path::new("config/VestaDesk"));
///
/// // With an override: that folder itself.
/// let settings_dir = settings_dir_from(Some("custom".into()), Some(config_dir))?;
/// assert_eq!(settings_dir, Path::new("custom"));
/// # Ok::<(), vesta_desk::error::VestaDeskError>(())
/// ```
pub fn settings_dir_from(
    override_dir: Option<PathBuf>,
    config_dir: Option<PathBuf>,
) -> Result<PathBuf> {
    match override_dir {
        Some(dir) if !dir.as_os_str().is_empty() => Ok(dir),
        _ => config_dir
            .map(|dir| dir.join(APP_DIR))
            .ok_or(VestaDeskError::ConfigDirUnavailable),
    }
}

/// Creates `dir` and any missing parent directories, then returns it; an existing directory is
/// left as it is.
///
/// # Errors
///
/// Returns [`VestaDeskError::StorageIo`] if the directory cannot be created.
pub fn ensure_dir(dir: PathBuf) -> Result<PathBuf> {
    match fs::create_dir_all(&dir) {
        Ok(()) => Ok(dir),
        Err(source) => Err(VestaDeskError::StorageIo { path: dir, source }),
    }
}
