//! The settings directory is the override if one is given, otherwise the `VestaDesk` folder in the
//! user's configuration directory; finding it fails only without either, and it is created if missing.
//! A save replaces the file in one step and leaves no temporary file, even when it fails, and the
//! temporary files of interrupted saves are removed at startup.
//! A document is saved as indented JSON that starts with its format version. It loads back with
//! defaults for a missing file or field, ignores unknown fields, upgrades older formats, and
//! rejects a damaged or oversized file.
//! An auto-save saves a change once it has stayed unchanged for the save delay, or when it is
//! dropped: many quick changes give a single save, an unchanged value none, and a failed save is
//! retried on the next change.
#![cfg(test)]

use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use std::sync::mpsc::{self, Receiver};
use std::time::{Duration, Instant};

use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};
use vesta_desk::error::AppError;
use vesta_desk::storage::{
    AutoSave, Document, MAX_DOCUMENT_SIZE, SAVE_DELAY, ensure_dir, load, remove_temp_files, save,
    settings_dir_from, write_atomic,
};

/// The longest wait for a save: far more than the save delay, so that a slow machine still passes.
const TIMEOUT: Duration = Duration::from_secs(10);

/// A test document in its second format; the first one named `text` what is now `body`.
#[derive(Debug, PartialEq, Serialize, Deserialize)]
#[serde(default)]
struct Note {
    body: String,
    pinned: bool,
}

/// A new note is pinned, so that the tests can tell the note's defaults from those of its field types.
impl Default for Note {
    fn default() -> Self {
        Self {
            body: String::new(),
            pinned: true,
        }
    }
}

impl Document for Note {
    const FILE_NAME: &'static str = "note.json";
    const VERSION: u32 = 2;

    fn migrate(version: u32, fields: &mut Map<String, Value>) {
        if version < 2
            && let Some(text) = fields.remove("text")
        {
            fields.insert("body".to_owned(), text);
        }
    }
}

/// A note with the text `body` and default values otherwise.
fn note(body: &str) -> Note {
    Note {
        body: body.to_owned(),
        ..Note::default()
    }
}

/// Writes `json` as the note file in `dir`.
fn write_note(dir: &Path, json: &str) {
    fs::write(dir.join(Note::FILE_NAME), json).unwrap();
}

/// The names of the entries in `dir`, sorted.
fn names(dir: &Path) -> Vec<String> {
    let mut names: Vec<String> = fs::read_dir(dir)
        .unwrap()
        .map(|entry| entry.unwrap().file_name().into_string().unwrap())
        .collect();
    names.sort();
    names
}

/// Returns an auto-save of numbers that starts from 0, and the channel that receives each number it saves.
fn autosave() -> (AutoSave<u32>, Receiver<u32>) {
    let (sender, saves) = mpsc::channel();
    let autosave = AutoSave::start(0, move |&number| {
        sender.send(number).unwrap();
        Ok(())
    })
    .unwrap();
    (autosave, saves)
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

#[test]
fn a_saved_document_starts_with_its_version_and_loads_back() {
    let dir = tempfile::tempdir().unwrap();
    let unpinned = Note {
        body: "hello".to_owned(),
        pinned: false,
    };
    save(dir.path(), &unpinned).unwrap();
    let json = fs::read_to_string(dir.path().join("note.json")).unwrap();
    assert_eq!(
        json,
        "{\n  \"version\": 2,\n  \"body\": \"hello\",\n  \"pinned\": false\n}\n"
    );
    assert_eq!(load::<Note>(dir.path()).unwrap(), unpinned);
}

#[test]
fn a_missing_file_gives_the_default_document() {
    let dir = tempfile::tempdir().unwrap();
    assert_eq!(load::<Note>(dir.path()).unwrap(), Note::default());
}

#[test]
fn missing_fields_take_their_default_and_unknown_ones_are_ignored() {
    let dir = tempfile::tempdir().unwrap();
    write_note(
        dir.path(),
        r#"{"version": 2, "body": "hello", "color": "red"}"#,
    );
    assert_eq!(load::<Note>(dir.path()).unwrap(), note("hello"));
}

#[test]
fn a_file_in_an_older_format_is_upgraded() {
    let dir = tempfile::tempdir().unwrap();
    write_note(dir.path(), r#"{"version": 1, "text": "hello"}"#);
    assert_eq!(load::<Note>(dir.path()).unwrap(), note("hello"));
}

/// A file from a newer version loads with the fields this version knows, and a file without a version, such as one written by hand, loads too.
#[test]
fn a_file_from_a_newer_version_or_without_a_version_loads() {
    let dir = tempfile::tempdir().unwrap();
    for json in [
        r#"{"version": 3, "body": "hello", "tags": ["a"]}"#,
        r#"{"body": "hello"}"#,
    ] {
        write_note(dir.path(), json);
        let result = load::<Note>(dir.path());
        assert_eq!(
            result.as_ref().ok(),
            Some(&note("hello")),
            "{json:?} -> {result:?}"
        );
    }
}

/// A file that is not a valid document, such as an empty or cut-off one, or one with a value of the wrong type, is rejected as corrupt.
#[test]
fn a_damaged_file_is_rejected_as_corrupt() {
    let dir = tempfile::tempdir().unwrap();
    let file = dir.path().join("note.json");
    for json in [
        "",
        "not json",
        r#"{"body": "hello""#,
        "[1, 2]",
        r#"{"body": 3}"#,
        r#"{"version": "two"}"#,
    ] {
        fs::write(&file, json).unwrap();
        let result = load::<Note>(dir.path());
        assert!(
            matches!(&result, Err(AppError::DocumentCorrupt { path, .. }) if *path == file),
            "{json:?} -> {result:?}"
        );
    }
}

/// A file of exactly the size limit loads, and a larger one is rejected.
#[test]
fn a_file_over_the_size_limit_is_rejected() {
    let dir = tempfile::tempdir().unwrap();
    let file = dir.path().join("note.json");
    let padding = usize::try_from(MAX_DOCUMENT_SIZE).unwrap() - 2;
    fs::write(&file, format!("{{}}{}", " ".repeat(padding))).unwrap();
    assert_eq!(load::<Note>(dir.path()).unwrap(), Note::default());
    fs::File::options()
        .write(true)
        .open(&file)
        .unwrap()
        .set_len(MAX_DOCUMENT_SIZE + 1)
        .unwrap();
    assert!(matches!(
        load::<Note>(dir.path()),
        Err(AppError::DocumentTooLarge { path, size, limit })
            if path == file && size == MAX_DOCUMENT_SIZE + 1 && limit == MAX_DOCUMENT_SIZE
    ));
}

/// Some editors start a UTF-8 file with a byte order mark, which is skipped.
#[test]
fn a_byte_order_mark_is_skipped() {
    let dir = tempfile::tempdir().unwrap();
    write_note(dir.path(), "\u{feff}{\"body\": \"hello\"}");
    assert_eq!(load::<Note>(dir.path()).unwrap(), note("hello"));
}

/// A change is saved once no other change has followed it for the save delay, while the app keeps running.
#[test]
fn a_change_is_saved_once_it_has_stayed_unchanged_for_the_save_delay() {
    let (mut autosave, saves) = autosave();
    autosave.update(&1);
    std::thread::sleep(SAVE_DELAY / 2);
    let changed = Instant::now();
    autosave.update(&2);
    // On a busy machine, 1 may be saved before 2 comes: only the save of 2 is checked.
    while saves.recv_timeout(TIMEOUT).unwrap() != 2 {}
    assert!(changed.elapsed() >= SAVE_DELAY);
}

/// Dropping the auto-save, as the app does when it closes, saves the last of the changes at once.
#[test]
fn closing_saves_only_the_last_change() {
    let (mut autosave, saves) = autosave();
    for number in 1..=10 {
        autosave.update(&number);
    }
    drop(autosave);
    assert_eq!(saves.try_iter().collect::<Vec<_>>(), [10]);
}

#[test]
fn a_value_equal_to_the_saved_one_is_not_saved() {
    let (mut autosave, saves) = autosave();
    autosave.update(&0);
    drop(autosave);
    assert_eq!(saves.try_iter().count(), 0);
}

/// After a failed save, the auto-save keeps running and tries again with the next change.
#[test]
fn a_failed_save_is_retried_on_the_next_change() {
    let (sender, tries) = mpsc::channel();
    let mut autosave = AutoSave::start(0, move |&number| {
        sender.send(number).unwrap();
        if number == 1 {
            Err(AppError::StorageIo {
                path: "settings.json".into(),
                source: io::Error::other("disk full"),
            })
        } else {
            Ok(())
        }
    })
    .unwrap();
    autosave.update(&1);
    assert_eq!(tries.recv_timeout(TIMEOUT).unwrap(), 1);
    autosave.update(&2);
    drop(autosave);
    assert_eq!(tries.try_iter().collect::<Vec<_>>(), [2]);
}
