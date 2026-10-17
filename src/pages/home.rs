//! The Home page, which the app opens on.

use egui::Ui;

use crate::i18n::Language;
use crate::router::Route;

/// Draws the Home page in `language`.
pub(crate) fn show(ui: &mut Ui, language: Language) {
    ui.heading(Route::Home.title(language));
}
