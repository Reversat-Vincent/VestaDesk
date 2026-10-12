//! Persistence: OS config directory resolution, atomic writes ([`write_atomic`]) and JSON stores.
//!
//! # Safe saving
//!
//! The app saves every file with [`write_atomic`], so that a crash or power cut during a save
//! leaves either the old file or the new one, never a half-written file. The new contents go to a
//! temporary file in the same folder and are flushed to disk, then a single rename replaces the
//! old file with it. If the save fails, the old file is left untouched. At startup,
//! [`remove_temp_files`] deletes the temporary files of interrupted saves.
//!
//! - On Linux, the folder is also flushed to disk after the rename, so a finished save survives a
//!   power cut. If that flush fails, the app logs a warning.
//! - On Windows, a power cut right after a save can bring back the old file. While another
//!   program, such as an antivirus, briefly holds the file, the rename is retried for about a second.

use std::env;
use std::fs::{self, File};
use std::io::{self, Write};
use std::path::{Path, PathBuf};
use std::process;
use std::sync::atomic::{AtomicU64, Ordering};
use std::thread;
use std::time::Duration;

use crate::error::{AppError, Result};
use crate::platform;

/// Name of VestaDesk's folder inside the user's configuration directory.
const APP_DIR: &str = "VestaDesk";

/// Environment variable naming a folder to use instead of the default settings directory.
const SETTINGS_DIR_ENV: &str = "VESTADESK_SETTINGS_DIR";

/// The end of the name of every temporary file that [`write_atomic`] creates.
const TEMP_SUFFIX: &str = ".vestadesk.tmp";

/// The waits, in milliseconds, between attempts to replace a file that another program holds: 1.27 seconds in all.
const RETRY_DELAYS_MS: [u64; 7] = [10, 20, 40, 80, 160, 320, 640];

/// Returns the directory where VestaDesk stores its settings, chosen by [`settings_dir_from`] from
/// the `VESTADESK_SETTINGS_DIR` environment variable and the user's configuration directory (the
/// application-data folder on Windows, `$XDG_CONFIG_HOME` or `~/.config` on Linux).
///
/// # Errors
///
/// Returns [`AppError::ConfigDirUnavailable`] if the variable is unset or empty and the
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
/// Returns [`AppError::ConfigDirUnavailable`] if there is neither an override nor a
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
/// # Ok::<(), vesta_desk::error::AppError>(())
/// ```
pub fn settings_dir_from(
    override_dir: Option<PathBuf>,
    config_dir: Option<PathBuf>,
) -> Result<PathBuf> {
    match override_dir {
        Some(dir) if !dir.as_os_str().is_empty() => Ok(dir),
        _ => config_dir
            .map(|dir| dir.join(APP_DIR))
            .ok_or(AppError::ConfigDirUnavailable),
    }
}

/// Creates `dir` and any missing parent directories, then returns it; an existing directory is
/// left as it is.
///
/// # Errors
///
/// Returns [`AppError::StorageIo`] if the directory cannot be created.
pub fn ensure_dir(dir: PathBuf) -> Result<PathBuf> {
    match fs::create_dir_all(&dir) {
        Ok(()) => Ok(dir),
        Err(source) => Err(AppError::StorageIo { path: dir, source }),
    }
}

/// Saves `contents` to the file at `path`, creating or replacing it, without ever leaving a half-written file.
///
/// See [Safe saving](crate::storage#safe-saving) for how it works on each system. On Windows, it
/// can take about a second while another program holds the file.
///
/// # Errors
///
/// Returns [`AppError::StorageIo`] if the file cannot be saved; it is then left untouched.
pub fn write_atomic(path: &Path, contents: &[u8]) -> Result<()> {
    let temp = temp_path(path);
    write_new(&temp, contents)
        .and_then(|()| replace(&temp, path))
        .inspect_err(|_| discard_temp_file(&temp))
        .map_err(io_error(path))?;
    // The file is saved at this point, so a folder that cannot be synced is reported, not returned.
    if let Some(dir) = path.parent()
        && let Err(error) = platform::sync_dir(dir)
    {
        error.report();
    }
    Ok(())
}

/// Removes the temporary files that interrupted saves left in `dir`, whose names end with `.vestadesk.tmp`.
///
/// # Errors
///
/// Returns [`AppError::StorageIo`] if `dir` cannot be read or one of these files cannot be removed.
pub fn remove_temp_files(dir: &Path) -> Result<()> {
    for entry in fs::read_dir(dir).map_err(io_error(dir))? {
        let path = entry.map_err(io_error(dir))?.path();
        if is_temp_file(&path) {
            fs::remove_file(&path).map_err(io_error(&path))?;
            log::debug!("removed {}, left by an interrupted save", path.display());
        }
    }
    Ok(())
}

/// Returns whether `path` names a temporary file that [`write_atomic`] creates.
fn is_temp_file(path: &Path) -> bool {
    path.as_os_str()
        .as_encoded_bytes()
        .ends_with(TEMP_SUFFIX.as_bytes())
}

/// Returns a new path, unique among running processes, for the temporary file of a save to `path`.
fn temp_path(path: &Path) -> PathBuf {
    static SAVES: AtomicU64 = AtomicU64::new(0);
    let save = SAVES.fetch_add(1, Ordering::Relaxed);
    let mut temp = path.as_os_str().to_owned();
    temp.push(format!(".{}-{save}{TEMP_SUFFIX}", process::id()));
    temp.into()
}

/// Creates the file at `path`, which must not exist yet, then writes `contents` to it and flushes them to disk.
fn write_new(path: &Path, contents: &[u8]) -> io::Result<()> {
    let mut file = File::create_new(path)?;
    file.write_all(contents)?;
    file.sync_all()
}

/// Renames `temp` to `path`, replacing the file there, and retries while another program holds either file.
fn replace(temp: &Path, path: &Path) -> io::Result<()> {
    for delay in RETRY_DELAYS_MS {
        match fs::rename(temp, path) {
            Err(error) if platform::is_file_in_use(&error) => {
                thread::sleep(Duration::from_millis(delay));
            }
            renamed => return renamed,
        }
    }
    fs::rename(temp, path)
}

/// Deletes `temp`, the temporary file of a failed save, if the save got to create it.
///
/// A file that cannot be deleted is reported as [`AppError::StorageIo`], and deleted at the next startup.
fn discard_temp_file(temp: &Path) {
    if let Err(source) = fs::remove_file(temp)
        && source.kind() != io::ErrorKind::NotFound
    {
        AppError::StorageIo {
            path: temp.to_owned(),
            source,
        }
        .report();
    }
}

/// Returns a function that wraps an I/O error on `path` in [`AppError::StorageIo`].
fn io_error(path: &Path) -> impl FnOnce(io::Error) -> AppError {
    move |source| AppError::StorageIo {
        path: path.to_owned(),
        source,
    }
}
