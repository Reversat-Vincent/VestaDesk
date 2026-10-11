//! The settings directory is the `VestaDesk` folder in the user's configuration directory, and finding it fails if the system has none.

use std::path::PathBuf;

use vesta_desk::error::VestaDeskError;
use vesta_desk::storage::settings_dir_in;

#[test]
fn settings_dir_is_the_vesta_desk_folder_in_the_config_dir() {
    let config_dir = PathBuf::from("config");
    let settings_dir = settings_dir_in(Some(config_dir.clone())).unwrap();
    assert_eq!(settings_dir, config_dir.join("VestaDesk"));
}

#[test]
fn settings_dir_fails_without_a_config_dir() {
    assert!(matches!(
        settings_dir_in(None),
        Err(VestaDeskError::ConfigDirUnavailable)
    ));
}
