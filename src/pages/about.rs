//! The About page, with the app's version and third-party notices.

use egui::Ui;

use crate::router::Route;

/// Draws the About page.
pub(crate) fn show(ui: &mut Ui) {
    ui.heading(Route::About.title());
}
