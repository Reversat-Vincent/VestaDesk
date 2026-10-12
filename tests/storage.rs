//! The settings directory is the override if one is given, otherwise the `VestaDesk` folder in the
//! user's configuration directory; finding it fails only without either, and it is created if missing.
//! A save replaces the file in one step and leaves no temporary file, even when it fails, and the
//! temporary files of interrupted saves are removed at startup.
#![cfg(test)]

use std::fs;
use std::path::{Path, PathBuf};

use vesta_desk::error::AppError;
use vesta_desk::storage::{ensure_dir, remove_temp_files, settings_dir_from, write_atomic};

/// The names of the entries in `dir`, sorted.
fn names(dir: &Path) -> Vec<String> {
    let mut names: Vec<String> = fs::read_dir(dir)
        .unwrap()
        .map(|entry| entry.unwrap().file_name().into_string().unwrap())
        .collect();
    names.sort();
    names
}

/// Opens `file` the way some antivirus software does on Windows: other programs can read it, but not replace it.
#[cfg(target_os = "windows")]
fn hold(file: &Path) -> fs::File {
    use std::os::windows::fs::OpenOptionsExt;

    /// The Windows sharing mode that lets other programs only read the file.
    const FILE_SHARE_READ: u32 = 1;
    fs::OpenOptions::new()
        .read(true)
        .share_mode(FILE_SHARE_READ)
        .open(file)
        .unwrap()
}

/// Without an override, the settings directory is the `VestaDesk` folder in the configuration directory.
#[test]
fn settings_dir_is_the_vesta_desk_folder_in_the_config_dir() {
    let config_dir = PathBuf::from("config");
    let settings_dir = settings_dir_from(None, Some(config_dir.clone())).unwrap();
    assert_eq!(settings_dir, config_dir.join("VestaDesk"));
}

/// An empty override, such as an empty `VESTADESK_SETTINGS_DIR`, counts as no override.
#[test]
fn an_empty_override_is_ignored() {
    let config_dir = PathBuf::from("config");
    let settings_dir = settings_dir_from(Some(PathBuf::new()), Some(config_dir.clone())).unwrap();
    assert_eq!(settings_dir, config_dir.join("VestaDesk"));
}

/// A non-empty override becomes the settings directory, with or without a configuration directory.
#[test]
fn an_override_replaces_the_settings_dir() {
    let override_dir = PathBuf::from("custom");
    for config_dir in [Some(PathBuf::from("config")), None] {
        let settings_dir = settings_dir_from(Some(override_dir.clone()), config_dir).unwrap();
        assert_eq!(settings_dir, override_dir);
    }
}

/// When the override is missing or empty and there is no configuration directory, finding the settings directory fails.
#[test]
fn settings_dir_fails_without_an_override_or_a_config_dir() {
    for override_dir in [None, Some(PathBuf::new())] {
        assert!(matches!(
            settings_dir_from(override_dir, None),
            Err(AppError::ConfigDirUnavailable)
        ));
    }
}

#[test]
fn ensure_dir_creates_the_missing_folders_and_keeps_existing_ones() {
    let root = tempfile::tempdir().unwrap();
    let dir = root.path().join("config").join("VestaDesk");
    assert_eq!(ensure_dir(dir.clone()).unwrap(), dir);
    assert!(dir.is_dir());
    assert_eq!(ensure_dir(dir.clone()).unwrap(), dir);
}

#[test]
fn ensure_dir_fails_if_a_file_has_the_folder_name() {
    let root = tempfile::tempdir().unwrap();
    let file = root.path().join("VestaDesk");
    fs::write(&file, "").unwrap();
    assert!(matches!(
        ensure_dir(file.clone()),
        Err(AppError::StorageIo { path, .. }) if path == file
    ));
}

#[test]
fn write_atomic_creates_or_replaces_the_file_and_leaves_no_temporary_file() {
    let dir = tempfile::tempdir().unwrap();
    let file = dir.path().join("settings.json");
    for contents in ["old", "new"] {
        write_atomic(&file, contents.as_bytes()).unwrap();
        assert_eq!(fs::read_to_string(&file).unwrap(), contents);
        assert_eq!(names(dir.path()), ["settings.json"]);
    }
}

/// A save that fails, here because a folder has the file's name, changes nothing and removes its temporary file.
#[test]
fn a_failed_save_changes_nothing_and_leaves_no_temporary_file() {
    let dir = tempfile::tempdir().unwrap();
    let file = dir.path().join("settings.json");
    fs::create_dir(&file).unwrap();
    assert!(matches!(
        write_atomic(&file, b"new"),
        Err(AppError::StorageIo { path, .. }) if path == file
    ));
    assert!(file.is_dir());
    assert_eq!(names(dir.path()), ["settings.json"]);
}

/// On Windows, a file that another program holds for a moment is still replaced.
#[cfg(target_os = "windows")]
#[test]
fn a_file_held_for_a_moment_is_still_replaced() {
    let dir = tempfile::tempdir().unwrap();
    let file = dir.path().join("settings.json");
    fs::write(&file, "old").unwrap();
    let held = hold(&file);
    let release = std::thread::spawn(move || {
        std::thread::sleep(std::time::Duration::from_millis(100));
        drop(held);
    });
    write_atomic(&file, b"new").unwrap();
    release.join().unwrap();
    assert_eq!(fs::read_to_string(&file).unwrap(), "new");
    assert_eq!(names(dir.path()), ["settings.json"]);
}

/// On Windows, a save fails and leaves the file untouched if another program holds it for longer than the retries last.
#[cfg(target_os = "windows")]
#[test]
fn a_file_held_for_too_long_is_left_untouched() {
    let dir = tempfile::tempdir().unwrap();
    let file = dir.path().join("settings.json");
    fs::write(&file, "old").unwrap();
    let held = hold(&file);
    assert!(matches!(
        write_atomic(&file, b"new"),
        Err(AppError::StorageIo { path, .. }) if path == file
    ));
    drop(held);
    assert_eq!(fs::read_to_string(&file).unwrap(), "old");
    assert_eq!(names(dir.path()), ["settings.json"]);
}

/// Only the temporary files of interrupted saves are removed, not other files, even with a `.tmp` extension.
#[test]
fn remove_temp_files_removes_only_the_temporary_files_of_saves() {
    let dir = tempfile::tempdir().unwrap();
    for name in [
        "settings.json",
        "settings.json.1234-0.vestadesk.tmp",
        "notes.tmp",
    ] {
        fs::write(dir.path().join(name), "").unwrap();
    }
    remove_temp_files(dir.path()).unwrap();
    assert_eq!(names(dir.path()), ["notes.tmp", "settings.json"]);
}

/// The app removes the temporary files of interrupted saves from the settings directory when it starts.
#[cfg(target_os = "linux")]
#[test]
fn startup_removes_the_temporary_files_of_interrupted_saves() {
    let settings_dir = tempfile::tempdir().unwrap();
    let leftover = settings_dir
        .path()
        .join("settings.json.1234-0.vestadesk.tmp");
    fs::write(&leftover, "").unwrap();
    // Without a display, the app stops right after starting.
    let output = std::process::Command::new(env!("CARGO_BIN_EXE_vesta-desk"))
        .env_remove("DISPLAY")
        .env_remove("WAYLAND_DISPLAY")
        .env_remove("WAYLAND_SOCKET")
        .env("VESTADESK_SETTINGS_DIR", settings_dir.path())
        .output()
        .unwrap();
    let log = String::from_utf8_lossy(&output.stderr);
    assert!(!leftover.exists(), "{log}");
}
