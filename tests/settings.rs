//! The default settings are saved as readable JSON with their format version, at 100 % zoom on the
//! Home page, and every page is saved and loads back. When the settings are loaded, a missing value
//! takes its default, an out-of-range zoom is corrected, and an unknown page gives the Home page.
//! A zoom that is not a number is corrected to the default zoom.

use std::fs;

use vesta_desk::router::Route;
use vesta_desk::settings::{MAX_ZOOM, MIN_ZOOM, Settings};
use vesta_desk::storage::{Document, load, save};

#[test]
fn the_default_settings_are_saved_as_readable_json_with_their_format_version() {
    let dir = tempfile::tempdir().unwrap();
    save(dir.path(), &Settings::default()).unwrap();
    let json = fs::read_to_string(dir.path().join("settings.json")).unwrap();
    assert_eq!(
        json,
        "{\n  \"version\": 1,\n  \"zoom\": 1.0,\n  \"page\": \"home\"\n}\n"
    );
}

#[test]
fn every_page_is_saved_and_loads_back() {
    let dir = tempfile::tempdir().unwrap();
    for &page in Route::ALL {
        let settings = Settings {
            page,
            ..Settings::default()
        };
        save(dir.path(), &settings).unwrap();
        assert_eq!(load::<Settings>(dir.path()).unwrap(), settings);
    }
}

#[test]
fn a_settings_file_with_only_its_version_loads_the_default_settings() {
    let dir = tempfile::tempdir().unwrap();
    fs::write(dir.path().join("settings.json"), r#"{"version": 1}"#).unwrap();
    let settings = load::<Settings>(dir.path()).unwrap();
    assert_eq!(settings, Settings::default());
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
        let expected = Settings {
            zoom: corrected,
            ..Settings::default()
        };
        assert_eq!(settings, expected, "zoom {zoom}");
    }
}

/// A zoom that is not a number, which no file can hold but a calculation can give, takes the default zoom.
#[test]
fn a_zoom_that_is_not_a_number_is_corrected_to_the_default() {
    let mut settings = Settings {
        zoom: f32::NAN,
        ..Settings::default()
    };
    settings.correct();
    assert_eq!(settings, Settings::default());
}

/// A page that only a newer version has, or a value that is not a page, gives the Home page, and the other settings are kept.
#[test]
fn an_unknown_page_gives_the_home_page() {
    let dir = tempfile::tempdir().unwrap();
    for page in [r#""notes""#, "3", "null"] {
        let json = format!(r#"{{"version": 1, "zoom": 1.5, "page": {page}}}"#);
        fs::write(dir.path().join("settings.json"), json).unwrap();
        let settings = load::<Settings>(dir.path()).unwrap();
        let expected = Settings {
            zoom: 1.5,
            page: Route::Home,
        };
        assert_eq!(settings, expected, "page {page}");
    }
}
