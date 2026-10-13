//! The default settings are saved as readable JSON with their format version, at 100 % zoom. When
//! the settings are loaded, a missing zoom becomes 100 % and an out-of-range one is corrected.

use std::fs;

use vesta_desk::settings::{MAX_ZOOM, MIN_ZOOM, Settings};
use vesta_desk::storage::{load, save};

#[test]
fn the_default_settings_are_saved_as_readable_json_with_their_format_version() {
    let dir = tempfile::tempdir().unwrap();
    save(dir.path(), &Settings::default()).unwrap();
    let json = fs::read_to_string(dir.path().join("settings.json")).unwrap();
    assert_eq!(json, "{\n  \"version\": 1,\n  \"zoom\": 1.0\n}\n");
}

#[test]
fn a_settings_file_without_a_zoom_loads_at_100_percent() {
    let dir = tempfile::tempdir().unwrap();
    fs::write(dir.path().join("settings.json"), r#"{"version": 1}"#).unwrap();
    let settings = load::<Settings>(dir.path()).unwrap();
    assert_eq!(settings, Settings { zoom: 1.0 });
}

/// A zoom out of range, even an extreme one, is brought back to the nearest limit; a zoom in range is kept.
#[test]
fn an_out_of_range_zoom_is_corrected() {
    let dir = tempfile::tempdir().unwrap();
    for (zoom, corrected) in [
        ("1000", MAX_ZOOM),
        ("1e300", MAX_ZOOM),
        ("0", MIN_ZOOM),
        ("-2", MIN_ZOOM),
        ("1.5", 1.5),
    ] {
        let json = format!(r#"{{"version": 1, "zoom": {zoom}}}"#);
        fs::write(dir.path().join("settings.json"), json).unwrap();
        let settings: Settings = load::<Settings>(dir.path()).unwrap();
        assert_eq!(settings, Settings { zoom: corrected }, "zoom {zoom}");
    }
}
