//! Each kind of failure has the expected description and a severity, and a failed start gives a failure exit code.

use std::process::ExitCode;

use vesta_desk::error::{Severity, VestaDeskError, exit_code};

#[test]
fn each_failure_has_the_expected_description_and_severity() {
    let parse_error = serde_json::from_str::<u8>("not json").unwrap_err();
    let corrupt = format!("the settings file settings.json is corrupt: {parse_error}");
    let cases = [
        (
            VestaDeskError::ConfigDirUnavailable,
            Severity::Recoverable,
            "the system provides no configuration directory",
        ),
        (
            VestaDeskError::Startup("no window".into()),
            Severity::Fatal,
            "the application could not start: no window",
        ),
        (
            VestaDeskError::StorageIo {
                path: "settings.json".into(),
                source: std::io::Error::other("denied"),
            },
            Severity::Recoverable,
            "cannot access settings.json: denied",
        ),
        (
            VestaDeskError::SettingsCorrupt {
                path: "settings.json".into(),
                source: parse_error,
            },
            Severity::Recoverable,
            &corrupt,
        ),
        (
            VestaDeskError::FontUnavailable("Consolas".into()),
            Severity::Recoverable,
            "the font \"Consolas\" is unavailable",
        ),
        (
            VestaDeskError::UnsupportedLocale("xx".into()),
            Severity::Recoverable,
            "the language \"xx\" is not supported",
        ),
    ];
    for (error, severity, description) in cases {
        assert_eq!(error.severity(), severity, "{error:?}");
        assert_eq!(error.to_string(), description, "{error:?}");
    }
}

#[test]
fn exit_code_follows_the_start_outcome() {
    let failure = VestaDeskError::Startup("no window".into());
    assert_eq!(exit_code(Ok(())), ExitCode::SUCCESS);
    assert_eq!(exit_code(Err(failure)), ExitCode::FAILURE);
}
