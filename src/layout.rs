//! The window layout, as in VS Code: the activity bar, the page and the status bar.
//!
//! [`show`] places the status bar along the bottom of the window, across its whole width, and the
//! activity bar on the left, above it. The activity bar has a fixed width that the user cannot
//! change. The page fills the rest and scrolls when it does not fit, so all three areas stay usable
//! down to the [minimum window size](crate::app::MIN_WINDOW_SIZE).
//!
#![doc = include_str!("../docs/assets/window-layout.svg")]
//!
//! The activity bar has one item per page, in the order of [`Route::ALL`]; until pages have icons,
//! each item shows the first letter of its page's title. Clicking an item opens its page, and the
//! item of the open page is highlighted. Each page keeps its own scroll position.

use egui::{Button, CentralPanel, Panel, Rect, ScrollArea, Ui};

use crate::action::{ActionQueue, AppAction};
use crate::i18n::Language;
use crate::pages;
use crate::router::Route;
use crate::state::AppState;

/// The width of the activity bar, in points.
pub const ACTIVITY_BAR_WIDTH: f32 = 48.0;

/// The height of the status bar, in points.
pub const STATUS_BAR_HEIGHT: f32 = 22.0;

/// Where [`show`] placed the three areas of the window, in points.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Areas {
    /// The activity bar, on the left.
    pub activity_bar: Rect,
    /// The central area, which shows the page.
    pub page: Rect,
    /// The status bar, along the bottom.
    pub status_bar: Rect,
}

/// Draws the three areas over all of `ui` from `state`, pushes the user's actions to `actions`, and returns where it placed the areas.
pub fn show(ui: &mut Ui, state: &AppState, actions: &mut ActionQueue) -> Areas {
    let route = state.route();
    let language = state.language();
    // A panel takes its side of the space the earlier panels left: the status bar goes first to span the whole width.
    let status_bar = Panel::bottom("status_bar")
        .resizable(false)
        .exact_size(STATUS_BAR_HEIGHT)
        .show(ui, |_| {})
        .response
        .rect;
    let activity_bar = Panel::left("activity_bar")
        .resizable(false)
        .exact_size(ACTIVITY_BAR_WIDTH)
        .show(ui, |ui| menu(ui, route, language, actions))
        .response
        .rect;
    let page = CentralPanel::default()
        .show(ui, |ui| {
            ScrollArea::both()
                .id_salt(route)
                .auto_shrink(false)
                .show(ui, |ui| pages::show(ui, route, language))
        })
        .response
        .rect;
    Areas {
        activity_bar,
        page,
        status_bar,
    }
}

/// Draws the item of each page in `language`, highlights the one of `current`, and pushes [`AppAction::Navigate`] when the user clicks one.
fn menu(ui: &mut Ui, current: Route, language: Language, actions: &mut ActionQueue) {
    ui.vertical_centered_justified(|ui| {
        for &route in Route::ALL {
            let item = Button::selectable(route == current, initial(route.title(language)));
            if ui.add(item).clicked() {
                actions.push(AppAction::Navigate(route));
            }
        }
    });
}

/// Returns the first letter of `title`.
fn initial(title: &str) -> &str {
    title
        .chars()
        .next()
        .map_or(title, |letter| &title[..letter.len_utf8()])
}
