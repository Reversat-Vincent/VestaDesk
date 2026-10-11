//! The settings directory is the override if one is given, otherwise the `VestaDesk` folder in the
//! user's configuration directory; finding it fails only without either, and it is created if missing.

use std::fs;
use std::path::PathBuf;

use vesta_desk::error::AppError;
use vesta_desk::storage::{ensure_dir, settings_dir_from};

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
