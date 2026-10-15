//! The queue gives every action once and in order, and background tasks send theirs from any thread, waking the window, until the app closes.

use std::sync::{Mutex, mpsc};
use std::thread;

use egui::{Context, ViewportId};
use vesta_desk::action::{self, AppAction};
use vesta_desk::error::AppError;
use vesta_desk::router::Route;

#[test]
fn the_queue_gives_every_action_once_and_in_order() {
    let (sender, mut queue) = action::channel(Context::default());
    sender.send(AppAction::Navigate(Route::About)).unwrap();
    sender.send(AppAction::Navigate(Route::Home)).unwrap();
    queue.push(AppAction::Navigate(Route::Settings));
    queue.push(AppAction::Navigate(Route::Home));
    // The screens' actions come first, then those of the background tasks.
    assert_eq!(
        queue.take(),
        [
            AppAction::Navigate(Route::Settings),
            AppAction::Navigate(Route::Home),
            AppAction::Navigate(Route::About),
            AppAction::Navigate(Route::Home),
        ]
    );
    assert!(queue.take().is_empty());
}

/// Like the app, the woken window takes the actions from the queue, so the action must be in it before the window wakes.
#[test]
fn an_action_sent_from_another_thread_wakes_the_window() {
    let ctx = Context::default();
    let (sender, queue) = action::channel(ctx.clone());
    let queue = Mutex::new(queue);
    let (wake_up, wake_ups) = mpsc::channel();
    ctx.set_request_repaint_callback(move |request| {
        let taken = queue.lock().unwrap().take();
        wake_up.send((request.viewport_id, taken)).unwrap();
    });
    thread::spawn(move || sender.send(AppAction::Navigate(Route::About)))
        .join()
        .unwrap()
        .unwrap();
    assert_eq!(
        wake_ups.try_iter().collect::<Vec<_>>(),
        [(ViewportId::ROOT, vec![AppAction::Navigate(Route::About)])]
    );
}

#[test]
fn sending_an_action_after_the_app_closed_fails() {
    let (sender, queue) = action::channel(Context::default());
    // The app owns the queue, so closing the app drops it.
    drop(queue);
    let sent = sender.send(AppAction::Navigate(Route::Home));
    assert!(matches!(sent, Err(AppError::AppClosed)), "{sent:?}");
}
