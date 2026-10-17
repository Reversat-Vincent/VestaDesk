//! The Settings page, where the user changes their preferences.

use egui::Ui;

use crate::i18n::Language;
use crate::router::Route;

/// Draws the Settings page in `language`.
pub(crate) fn show(ui: &mut Ui, language: Language) {
    ui.heading(Route::Settings.title(language));
}
