//! The status bar spans the whole width of the window, under the activity bar.
#![cfg(test)]

use egui::{Context, Pos2, RawInput, Rect};
use vesta_desk::action;
use vesta_desk::app::WINDOW_SIZE;
use vesta_desk::layout::{self, Areas};
use vesta_desk::state::AppState;

/// Returns where the layout places its areas in `window`.
fn areas(window: Rect) -> Areas {
    let input = RawInput {
        screen_rect: Some(window),
        ..RawInput::default()
    };
    let ctx = Context::default();
    let (_, mut actions) = action::channel(ctx.clone());
    let mut areas = None;
    ctx.run_ui(input, |ui| {
        areas = Some(layout::show(ui, &AppState::default(), &mut actions));
    })
    .drop_without_applying_deltas();
    areas.unwrap()
}

#[test]
fn the_status_bar_spans_the_window_under_the_activity_bar() {
    let window = Rect::from_min_size(Pos2::ZERO, WINDOW_SIZE);
    let areas = areas(window);
    assert_eq!(areas.status_bar.x_range(), window.x_range());
    assert_eq!(
        areas.activity_bar.left_bottom(),
        areas.status_bar.left_top()
    );
}
