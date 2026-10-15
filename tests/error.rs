//! Each kind of failure has the expected description and a severity, and a failed start gives a failure exit code.

use std::process::ExitCode;

use vesta_desk::error::{AppError, Severity, exit_code};

#[test]
fn each_failure_has_the_expected_description_and_severity() {
    let parse_error = serde_json::from_str::<u8>("not json").unwrap_err();
    let corrupt = format!("the file settings.json is corrupt: {parse_error}");
    let cases = [
        (
            AppError::ConfigDirUnavailable,
            Severity::Recoverable,
            "the system provides no configuration directory",
        ),
        (
            AppError::Startup("no window".into()),
            Severity::Fatal,
            "the application could not start: no window",
        ),
        (
            AppError::AppClosed,
            Severity::Recoverable,
            "the application has closed, so it cannot apply the action",
        ),
        (
            AppError::LoggerAlreadyInstalled,
            Severity::Recoverable,
            "a logger is already installed; keeping it",
        ),
        (
            AppError::StorageIo {
                path: "settings.json".into(),
                source: std::io::Error::other("denied"),
            },
            Severity::Recoverable,
            "cannot access settings.json: denied",
        ),
        (
            AppError::DirNotSynced {
                path: "VestaDesk".into(),
                source: std::io::Error::other("not supported"),
            },
            Severity::Recoverable,
            "cannot sync the folder VestaDesk: not supported; a power cut could undo the last save in it",
        ),
        (
            AppError::DocumentCorrupt {
                path: "settings.json".into(),
                source: parse_error,
            },
            Severity::Recoverable,
            &corrupt,
        ),
        (
            AppError::DocumentTooLarge {
                path: "settings.json".into(),
                size: 2_000_000,
                limit: 1_048_576,
            },
            Severity::Recoverable,
            "the file settings.json is too large: 2000000 bytes, over the limit of 1048576 bytes",
        ),
        (
            AppError::FontUnavailable("Consolas".into()),
            Severity::Recoverable,
            "the font \"Consolas\" is unavailable",
        ),
        (
            AppError::UnsupportedLocale("xx".into()),
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
    let failure = AppError::Startup("no window".into());
    assert_eq!(exit_code(Ok(())), ExitCode::SUCCESS);
    assert_eq!(exit_code(Err(failure)), ExitCode::FAILURE);
}
