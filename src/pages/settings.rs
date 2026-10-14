//! The Settings page, where the user changes their preferences.

use egui::Ui;

use crate::router::Route;

/// Draws the Settings page.
pub(crate) fn show(ui: &mut Ui) {
    ui.heading(Route::Settings.title());
}
