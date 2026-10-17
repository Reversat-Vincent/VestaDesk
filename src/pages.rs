//! The pages shown in the central area, one sub-module per [`Route`].

mod about;
mod home;
mod settings;

use egui::Ui;

use crate::i18n::Language;
use crate::router::Route;

/// Draws the page of `route` in `language`.
pub fn show(ui: &mut Ui, route: Route, language: Language) {
    match route {
        Route::Home => home::show(ui, language),
        Route::Settings => settings::show(ui, language),
        Route::About => about::show(ui, language),
    }
}
