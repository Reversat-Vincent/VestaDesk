//! Windows and Linux specifics; the only place for `cfg(target_os)` branches.
//!
//! VestaDesk supports only Windows and Linux: building it for any other system fails with a clear message.

use std::fs::File;
use std::io;
use std::path::Path;

use crate::error::{AppError, Result};

#[cfg(not(any(target_os = "windows", target_os = "linux")))]
compile_error!("VestaDesk supports only Windows and Linux.");

/// The Windows error code for a denied access, also given when another program holds a file being replaced.
const ERROR_ACCESS_DENIED: i32 = 5;

/// The Windows error code for a file that another program holds without sharing it.
const ERROR_SHARING_VIOLATION: i32 = 32;

/// Returns whether `error` can come from another program briefly holding a file, such as an antivirus scanning it.
///
/// Only Windows reports such errors: on Linux, a file can be read or replaced while other programs hold it.
///
/// # Examples
///
/// ```
/// use std::io;
///
/// use vesta_desk::platform::is_file_in_use;
///
/// // On Windows, error 32 is a sharing violation: another program holds the file.
/// let held = io::Error::from_raw_os_error(32);
/// assert_eq!(is_file_in_use(&held), cfg!(target_os = "windows"));
///
/// assert!(!is_file_in_use(&io::Error::from(io::ErrorKind::NotFound)));
/// ```
#[must_use]
pub fn is_file_in_use(error: &io::Error) -> bool {
    cfg!(target_os = "windows")
        && matches!(
            error.raw_os_error(),
            Some(ERROR_ACCESS_DENIED | ERROR_SHARING_VIOLATION)
        )
}

/// Writes the entries of the folder `dir` to disk on Linux, so that a file just renamed in it keeps its new name after a power cut.
///
/// It does nothing on Windows, which documents no way to sync a folder.
///
/// # Errors
///
/// Returns [`AppError::DirNotSynced`] if the folder cannot be written to disk, for example on a file system that cannot sync a folder.
pub fn sync_dir(dir: &Path) -> Result<()> {
    if cfg!(target_os = "linux") {
        File::open(dir)
            .and_then(|folder| folder.sync_all())
            .map_err(|source| AppError::DirNotSynced {
                path: dir.to_owned(),
                source,
            })?;
    }
    Ok(())
}
