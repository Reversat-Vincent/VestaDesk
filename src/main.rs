//! VestaDesk binary entry point; all logic lives in the `vesta_desk` library.

// Release builds on Windows use the GUI subsystem, so no console window opens; debug builds keep
// the console to show diagnostic output.
#![cfg_attr(all(windows, not(debug_assertions)), windows_subsystem = "windows")]

fn main() -> std::process::ExitCode {
    vesta_desk::run()
}
