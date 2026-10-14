//! The Home page, which the app opens on.

use egui::Ui;

use crate::router::Route;

/// Draws the Home page.
pub(crate) fn show(ui: &mut Ui) {
    ui.heading(Route::Home.title());
}
