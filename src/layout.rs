//! The window layout, as in VS Code: the activity bar, the page and the status bar.
//!
//! [`show`] places the status bar along the bottom of the window, across its whole width, and the
//! activity bar on the left, above it. The activity bar has a fixed width that the user cannot
//! change. The page fills the rest and scrolls when it does not fit, so all three areas stay usable
//! down to the [minimum window size](crate::app::MIN_WINDOW_SIZE).

use egui::{CentralPanel, Panel, Rect, ScrollArea, Ui};

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

/// Draws the three areas over all of `ui`, shows `page` in the central one, and returns where it placed them.
pub fn show(ui: &mut Ui, page: impl FnOnce(&mut Ui)) -> Areas {
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
        .show(ui, |_| {})
        .response
        .rect;
    let page = CentralPanel::default()
        .show(ui, |ui| {
            ScrollArea::both().auto_shrink(false).show(ui, page)
        })
        .response
        .rect;
    Areas {
        activity_bar,
        page,
        status_bar,
    }
}
