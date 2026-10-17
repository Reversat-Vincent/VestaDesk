//! The About page, with the app's version and third-party notices.

use egui::Ui;

use crate::i18n::{Language, Text};
use crate::router::Route;

/// Draws the About page in `language`.
pub(crate) fn show(ui: &mut Ui, language: Language) {
    ui.heading(Route::About.title(language));
    ui.label(language.format(Text::Version, &[("version", env!("CARGO_PKG_VERSION"))]));
}
