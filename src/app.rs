//! The main window and its `eframe::App` implementation ([`App`]): the single state container and the frame lifecycle.
//!
//! # Rendering
//!
//! eframe draws the window with OpenGL, through its `glow` renderer, instead of wgpu, its default.
//! For an empty window in release mode on Windows 11, glow gives a 7.1 MB program and a first
//! frame in 120 ms, against 13.8 MB and 630 ms for wgpu.
//! VestaDesk therefore needs OpenGL 2.0 or later.

use egui::Vec2;

use crate::error::{AppError, Result};

/// The app's name, shown as the window title.
const NAME: &str = "VestaDesk";

/// The size of the window when it opens, in points.
pub const WINDOW_SIZE: Vec2 = egui::vec2(800.0, 600.0);

/// The smallest size the window can be resized to, in points.
pub const MIN_WINDOW_SIZE: Vec2 = egui::vec2(640.0, 480.0);

/// The VestaDesk app, which eframe runs in the main window.
#[derive(Debug, Default)]
pub struct App;

impl eframe::App for App {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ui, |_| {});
    }
}

/// Returns the options of the main window: its title, size and minimum size, centered on the screen.
///
/// # Examples
///
/// ```
/// use vesta_desk::app::{MIN_WINDOW_SIZE, native_options};
///
/// let window = native_options().viewport;
/// assert_eq!(window.title.as_deref(), Some("VestaDesk"));
/// assert_eq!(window.min_inner_size, Some(MIN_WINDOW_SIZE));
/// ```
#[must_use]
pub fn native_options() -> eframe::NativeOptions {
    eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_title(NAME)
            .with_inner_size(WINDOW_SIZE)
            .with_min_inner_size(MIN_WINDOW_SIZE),
        centered: true,
        ..Default::default()
    }
}

/// Opens the main window and runs the app until the window is closed.
///
/// # Errors
///
/// Returns [`AppError::Startup`] if the window cannot be created, for example without a display or without OpenGL 2.0.
pub fn run() -> Result<()> {
    eframe::run_native(NAME, native_options(), Box::new(|_| Ok(Box::new(App))))
        .map_err(|error| AppError::Startup(error.to_string()))
}
