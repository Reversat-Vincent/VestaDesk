//! Persistence: the settings directory ([`settings_dir`]), atomic writes ([`write_atomic`]), versioned JSON documents ([`Document`]) and saving in the background ([`AutoSave`]).
//!
//! # Safe saving
//!
//! The app saves every file with [`write_atomic`], so that a crash or power cut during a save
//! leaves either the old file or the new one, never a half-written file. The new contents go to a
//! temporary file in the same folder and are flushed to disk, then a single rename replaces the
//! old file with it. If the save fails, the old file is left untouched. At startup,
//! [`remove_temp_files`] deletes the temporary files of interrupted saves.
//!
#![doc = include_str!("../docs/assets/safe-saving.svg")]
//!
//! - On Linux, the folder is also flushed to disk after the rename, so a finished save survives a
//!   power cut. If that flush fails, the app logs a warning.
//! - On Windows, a power cut right after a save can bring back the old file. While another
//!   program, such as an antivirus, briefly holds the file, the rename is retried for about a second.
//!
//! # Documents
//!
//! A [`Document`], such as the [`Settings`](crate::settings::Settings), is saved by [`save`] in the
//! settings directory as indented JSON that starts with its format version. [`load`] reads it back:
//!
//! - A missing file gives the default document, and a missing field its default value.
//! - Unknown fields are ignored, so a file from a newer version still loads.
//! - [`Document::migrate`] upgrades a file in an older format, then [`Document::correct`] fixes out-of-range values.
//! - A file over [`MAX_DOCUMENT_SIZE`], or whose content is not a valid document, is rejected.
//!
#![doc = include_str!("../docs/assets/loading-a-document.svg")]
//!
//! # Saving automatically
//!
//! An [`AutoSave`] saves a value, such as the [`Settings`](crate::settings::Settings),
//! each time it changes. It saves on a thread of its own, so that a slow disk or a file
//! held by another program never slows down the app. It waits until the value has stayed
//! unchanged for [`SAVE_DELAY`], so that many quick changes in a row give a single save,
//! but saves the last change at once when the app closes. A failed save is reported,
//! and the next change tries again.
//!
#![doc = include_str!("../docs/assets/saving-automatically.svg")]

use std::env;
use std::fs::{self, File};
use std::io::{self, Read, Write};
use std::path::{Path, PathBuf};
use std::process;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::mpsc::{self, Receiver, Sender};
use std::thread::{self, JoinHandle};
use std::time::Duration;

use serde::Serialize;
use serde::de::DeserializeOwned;
use serde_json::{Map, Value};

use crate::error::{AppError, Result};
use crate::platform;

/// The size of the largest file that [`load`] reads, in bytes: 1 MiB.
pub const MAX_DOCUMENT_SIZE: u64 = 1024 * 1024;

/// How long a value must stay unchanged before [`AutoSave`] saves it: half a second.
pub const SAVE_DELAY: Duration = Duration::from_millis(500);

/// Name of VestaDesk's folder inside the user's configuration directory.
const APP_DIR: &str = "VestaDesk";

/// Environment variable naming a folder to use instead of the default settings directory.
const SETTINGS_DIR_ENV: &str = "VESTADESK_SETTINGS_DIR";

/// The end of the name of every temporary file that [`write_atomic`] creates.
const TEMP_SUFFIX: &str = ".vestadesk.tmp";

/// The waits, in milliseconds, between attempts to replace a file that another program holds: 1.27 seconds in all.
const RETRY_DELAYS_MS: [u64; 7] = [10, 20, 40, 80, 160, 320, 640];

/// The byte order mark that some editors write at the start of a UTF-8 file, which JSON does not allow.
const UTF8_BOM: &[u8] = b"\xEF\xBB\xBF";

/// A file of preferences that [`save`] writes and [`load`] reads, as described in [Documents](crate::storage#documents).
///
/// Implement it on a struct marked `#[serde(default)]`, so that a field missing from the file takes
/// its default value. The field name `version` is reserved for the format version.
pub trait Document: Default + Serialize + DeserializeOwned {
    /// The name of the file in the settings directory, such as `settings.json`.
    const FILE_NAME: &'static str;

    /// The format version saved in the file; increase it when older files need [`migrate`](Document::migrate) to be read correctly.
    const VERSION: u32;

    /// Upgrades `fields`, read from a file in the older format `version`, to the current format; the default does nothing.
    fn migrate(_version: u32, _fields: &mut Map<String, Value>) {}

    /// Corrects out-of-range values once [`load`] has read the document; the default does nothing.
    fn correct(&mut self) {}
}

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

/// Reads the document `T` from its file in `dir`, or returns the default document if the file does not exist.
///
/// See [Documents](crate::storage#documents) for how the file is read.
///
/// # Errors
///
/// Returns [`AppError::DocumentTooLarge`] if the file is over [`MAX_DOCUMENT_SIZE`],
/// [`AppError::DocumentCorrupt`] if its content is not a valid document, and
/// [`AppError::StorageIo`] if it cannot be read.
pub fn load<T: Document>(dir: &Path) -> Result<T> {
    let path = dir.join(T::FILE_NAME);
    let Some(contents) = read_document(&path)? else {
        log::debug!("{} does not exist; using the defaults", path.display());
        return Ok(T::default());
    };
    let json = contents.strip_prefix(UTF8_BOM).unwrap_or(&contents);
    let mut fields: Map<String, Value> = serde_json::from_slice(json).map_err(corrupt(&path))?;
    // A file without a version, such as one written by hand, is read in the current format.
    let version = match fields.remove("version") {
        Some(version) => serde_json::from_value(version).map_err(corrupt(&path))?,
        None => T::VERSION,
    };
    if version < T::VERSION {
        T::migrate(version, &mut fields);
    } else if version > T::VERSION {
        log::info!(
            "{} comes from a newer version of VestaDesk: the preferences this version does not know are ignored",
            path.display()
        );
    }
    let mut document: T = serde_json::from_value(Value::Object(fields)).map_err(corrupt(&path))?;
    document.correct();
    Ok(document)
}

/// Saves `document` to its file in `dir` with [`write_atomic`], as indented JSON that starts with its format version.
///
/// # Errors
///
/// Returns [`AppError::StorageIo`] if the file cannot be saved; it is then left untouched.
pub fn save<T: Document>(dir: &Path, document: &T) -> Result<()> {
    let path = dir.join(T::FILE_NAME);
    let versioned = Versioned {
        version: T::VERSION,
        document,
    };
    let mut json = serde_json::to_vec_pretty(&versioned)
        .map_err(io::Error::from)
        .map_err(io_error(&path))?;
    json.push(b'\n');
    write_atomic(&path, &json)?;
    log::debug!("saved {}", path.display());
    Ok(())
}

/// Saves a value each time it changes, on a thread of its own, as described in [Saving automatically](crate::storage#saving-automatically).
///
/// Dropping it, as the app does when it closes, saves the last change at once and waits until it is saved.
#[derive(Debug)]
pub struct AutoSave<T> {
    /// The last value given to [`AutoSave::update`], or the saved value it started from.
    latest: T,
    /// The channel to the thread that saves, and that thread; `None` once the thread has stopped.
    thread: Option<(Sender<T>, JoinHandle<()>)>,
}

impl<T: Clone + PartialEq + Send + 'static> AutoSave<T> {
    /// Starts the thread that calls `save` with each change that [`update`](AutoSave::update) gives; `saved` is the value already saved, which needs no save.
    ///
    /// # Errors
    ///
    /// Returns [`AppError::AutoSaveUnavailable`] if the thread cannot start.
    pub fn start(saved: T, save: impl FnMut(&T) -> Result<()> + Send + 'static) -> Result<Self> {
        let (sender, changes) = mpsc::channel();
        let thread = thread::Builder::new()
            .name("autosave".to_owned())
            .spawn(move || save_changes(&changes, save))
            .map_err(AppError::AutoSaveUnavailable)?;
        Ok(Self {
            latest: saved,
            thread: Some((sender, thread)),
        })
    }

    /// Saves `value` if it differs from the previous one, once no other change has followed it for [`SAVE_DELAY`].
    pub fn update(&mut self, value: &T) {
        if *value == self.latest {
            return;
        }
        self.latest = value.clone();
        if let Some((changes, _)) = &self.thread
            && changes.send(value.clone()).is_err()
        {
            // The thread stops before its channel closes only if it panicked.
            self.stop();
        }
    }
}

impl<T> AutoSave<T> {
    /// Closes the channel, so that the thread saves the last change at once, then waits for the thread to end.
    ///
    /// A thread that panicked is reported as [`AppError::AutoSaveStopped`].
    fn stop(&mut self) {
        if let Some((changes, thread)) = self.thread.take() {
            drop(changes);
            if thread.join().is_err() {
                AppError::AutoSaveStopped.report();
            }
        }
    }
}

impl<T> Drop for AutoSave<T> {
    fn drop(&mut self) {
        self.stop();
    }
}

/// Saves with `save` each value from `changes` once no newer one has come for [`SAVE_DELAY`], or at once if the channel closes, until it is closed and empty.
fn save_changes<T>(changes: &Receiver<T>, mut save: impl FnMut(&T) -> Result<()>) {
    while let Ok(mut value) = changes.recv() {
        while let Ok(newer) = changes.recv_timeout(SAVE_DELAY) {
            value = newer;
        }
        if let Err(error) = save(&value) {
            error.report();
        }
    }
}

/// A document as [`save`] writes it: its format version, then its fields.
#[derive(Serialize)]
struct Versioned<'a, T> {
    /// The format version of the document.
    version: u32,
    /// The document, whose fields follow the version.
    #[serde(flatten)]
    document: &'a T,
}

/// Reads the file at `path`, which must not be over [`MAX_DOCUMENT_SIZE`], or returns `None` if it does not exist.
fn read_document(path: &Path) -> Result<Option<Vec<u8>>> {
    let file = match File::open(path) {
        Ok(file) => file,
        Err(error) if error.kind() == io::ErrorKind::NotFound => return Ok(None),
        Err(error) => return Err(io_error(path)(error)),
    };
    let size = file.metadata().map_err(io_error(path))?.len();
    if size > MAX_DOCUMENT_SIZE {
        return Err(AppError::DocumentTooLarge {
            path: path.to_owned(),
            size,
            limit: MAX_DOCUMENT_SIZE,
        });
    }
    let mut contents = Vec::new();
    // The limit also holds if the file grew after its size was checked.
    file.take(MAX_DOCUMENT_SIZE)
        .read_to_end(&mut contents)
        .map_err(io_error(path))?;
    Ok(Some(contents))
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

/// Returns a function that wraps an error in the content of the document file at `path` in [`AppError::DocumentCorrupt`].
fn corrupt(path: &Path) -> impl FnOnce(serde_json::Error) -> AppError {
    move |source| AppError::DocumentCorrupt {
        path: path.to_owned(),
        source,
    }
}
