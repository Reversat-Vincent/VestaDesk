//! The pages shown in the central area, one sub-module per [`Route`].

mod about;
mod home;
mod settings;

use egui::Ui;

use crate::router::Route;

/// Draws the page of `route`.
pub fn show(ui: &mut Ui, route: Route) {
    match route {
        Route::Home => home::show(ui),
        Route::Settings => settings::show(ui),
        Route::About => about::show(ui),
    }
}
