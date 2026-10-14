# VestaDesk

VestaDesk is a native desktop super-app in Rust, built with [egui/eframe](https://github.com/emilk/egui) for Windows and Linux.

> Early development: the app opens a window with an empty left menu, central area and status bar. Progress is tracked in the Linear project "VestaDesk".

## Requirements

VestaDesk runs on Windows and Linux only; building it for any other system fails with a clear message.

It needs a graphics driver with OpenGL 2.0 or later.

On Linux, VestaDesk needs a Wayland or X11 desktop session and these libraries, which most desktop distributions already provide.

## Build

Requires Rust 1.95 (pinned by `rust-toolchain.toml`, installed automatically by rustup).

```sh
cargo run                  # build and run
cargo test                 # run the tests
cargo doc --no-deps --open # build and open the developer documentation
```

## Settings

VestaDesk stores its settings in `%APPDATA%\VestaDesk` on Windows and `~/.config/VestaDesk` on Linux. Set `VESTADESK_SETTINGS_DIR` to use another folder.

## Logging

By default VestaDesk logs a startup summary (version, operating system and settings folder), warnings and errors. Release builds on Windows have no console window, so their log output is not visible.
Set `RUST_LOG` to see more:
- In a Linux terminal: `RUST_LOG=debug cargo run`.
- In PowerShell: `$env:RUST_LOG = "debug"; cargo run`.

## Contributing

See [CONTRIBUTING.md](CONTRIBUTING.md) for the workflow, the checks to run before pushing, and the lint policy.
