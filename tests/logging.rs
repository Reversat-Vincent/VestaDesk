//! The default log filter is quiet, `RUST_LOG` switches detail on, and installing the logger
//! more than once is harmless.

use log::{Level, Log, Metadata};
use vesta_desk::logging::builder;

/// Whether a logger configured with `spec` shows a message of `level` logged by `target`.
fn shows(spec: Option<&str>, level: Level, target: &str) -> bool {
    builder(spec)
        .build()
        .enabled(&Metadata::builder().level(level).target(target).build())
}

#[test]
fn by_default_only_problems_and_vesta_desk_information_are_shown() {
    assert!(shows(None, Level::Warn, "wgpu"));
    assert!(!shows(None, Level::Info, "wgpu"));
    assert!(shows(None, Level::Info, "vesta_desk::logging"));
    assert!(!shows(None, Level::Debug, "vesta_desk::logging"));
}

#[test]
fn rust_log_switches_detailed_logging_on() {
    assert!(shows(Some("debug"), Level::Debug, "wgpu"));
    assert!(!shows(Some("debug"), Level::Trace, "wgpu"));
    assert!(shows(
        Some("warn,vesta_desk=trace"),
        Level::Trace,
        "vesta_desk::logging"
    ));
    assert!(!shows(Some("warn,vesta_desk=trace"), Level::Debug, "wgpu"));
}

#[test]
fn rust_log_can_hide_what_is_shown_by_default() {
    // By default VestaDesk's information is shown; `RUST_LOG=error` should hide it too.
    assert!(!shows(Some("error"), Level::Info, "vesta_desk::logging"));
}

#[test]
fn init_can_be_called_twice() {
    // A test logger goes first, so that the warning logged by `init` is captured by the harness.
    builder(None).is_test(true).try_init().unwrap();

    vesta_desk::logging::init();
    vesta_desk::logging::init();
}
