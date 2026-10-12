//! Syncing a folder succeeds on an existing folder and, on Linux, fails with a clear error on a missing one.

use vesta_desk::error::AppError;
use vesta_desk::platform::sync_dir;

/// On Linux, a missing folder cannot be synced; Windows syncs no folder, so nothing fails there.
#[test]
fn sync_dir_fails_on_a_missing_folder_only_on_linux() {
    let root = tempfile::tempdir().unwrap();
    sync_dir(root.path()).unwrap();
    let missing = root.path().join("missing");
    let synced = sync_dir(&missing);
    if cfg!(target_os = "linux") {
        assert!(matches!(synced, Err(AppError::DirNotSynced { path, .. }) if path == missing));
    } else {
        assert!(synced.is_ok());
    }
}
