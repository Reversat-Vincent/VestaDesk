//! Actions ([`AppAction`]): every change to the app's data, pushed by the screens to an [`ActionQueue`] or sent by background tasks through an [`ActionSender`].
//!
//! Buttons, keyboard shortcuts and the command palette never change the data themselves: while the
//! window is drawn, they push an action to the queue. Background tasks send theirs from any thread,
//! which also asks for a screen update, so that their actions take effect even if the user does
//! nothing. At the start of the next screen update, the app takes every action from the queue and
//! applies it with [`AppState::apply`].
//!
#![doc = include_str!("../docs/assets/actions.svg")]
//!
//! Background tasks get their sender from [`App::sender`], and screens get only the queue, so they
//! can only push. Sending locks the egui context to ask for a screen update: in a closure that
//! already holds that lock, such as the one given to `ui.input`, it would freeze the app.
//!
//! # Adding an action
//!
//! 1. Add a variant to [`AppAction`], with a doc comment that says what it changes.
//! 2. Apply it in [`AppState::apply`]: its `match` has no catch-all arm, so the build fails until it does.
//! 3. Push it from the screens that trigger it, passing the queue down to them if needed, or send it from a background task.
//! 4. Test its effect on an [`AppState`] in `tests/state.rs`.
//! 5. Show it in the diagram above, `docs/assets/actions.svg`, with any field it adds to [`AppState`].
//!
//! [`App::sender`]: crate::app::App::sender
//! [`AppState`]: crate::state::AppState
//! [`AppState::apply`]: crate::state::AppState::apply

use std::mem;
use std::sync::mpsc::{self, Receiver, Sender};

use egui::{Context, ViewportId};

use crate::error::{AppError, Result};
use crate::notifications::Notification;
use crate::router::Route;

/// A change to the app's data, applied by [`AppState::apply`](crate::state::AppState::apply).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AppAction {
    /// Shows the page of the route in the central area.
    Navigate(Route),
    /// Raises the notification, which joins the [list of notifications](crate::notifications::Notifications).
    Notify(Notification),
}

/// Returns a sender for background tasks and the queue it sends to; each action sent asks `ctx` for a screen update.
///
/// # Examples
///
/// ```
/// use egui::Context;
/// use vesta_desk::action::{self, AppAction};
/// use vesta_desk::router::Route;
///
/// let (sender, mut queue) = action::channel(Context::default());
/// sender.send(AppAction::Navigate(Route::About))?;
/// assert_eq!(queue.take(), [AppAction::Navigate(Route::About)]);
/// # Ok::<(), vesta_desk::error::AppError>(())
/// ```
#[must_use]
pub fn channel(ctx: Context) -> (ActionSender, ActionQueue) {
    let (sender, received) = mpsc::channel();
    let queue = ActionQueue {
        pushed: Vec::new(),
        received,
    };
    (ActionSender { sender, ctx }, queue)
}

/// The actions waiting for the app to apply them at the start of the next screen update.
#[derive(Debug)]
pub struct ActionQueue {
    /// The actions that the screens pushed, in order.
    pushed: Vec<AppAction>,
    /// The actions that background tasks sent, in order.
    received: Receiver<AppAction>,
}

impl ActionQueue {
    /// Adds `action` to the queue; screens call it while the window is drawn.
    pub fn push(&mut self, action: AppAction) {
        self.pushed.push(action);
    }

    /// Returns the actions that the screens pushed since the queue was last taken, in order.
    ///
    /// # Examples
    ///
    /// ```
    /// use egui::Context;
    /// use vesta_desk::action::{self, AppAction};
    /// use vesta_desk::router::Route;
    ///
    /// let (_, mut queue) = action::channel(Context::default());
    /// queue.push(AppAction::Navigate(Route::About));
    /// assert_eq!(queue.pushed(), [AppAction::Navigate(Route::About)]);
    /// ```
    #[must_use]
    pub fn pushed(&self) -> &[AppAction] {
        &self.pushed
    }

    /// Takes every action out of the queue: first those the screens pushed, then those background tasks sent, each in order.
    #[must_use]
    pub fn take(&mut self) -> Vec<AppAction> {
        let mut actions = mem::take(&mut self.pushed);
        actions.extend(self.received.try_iter());
        actions
    }
}

/// Sends actions to an [`ActionQueue`] from any thread, for background tasks.
#[derive(Debug, Clone)]
pub struct ActionSender {
    /// The channel to the queue.
    sender: Sender<AppAction>,
    /// The egui context, asked for a screen update of the main window after each action.
    ctx: Context,
}

impl ActionSender {
    /// Sends `action` to the queue and asks for the screen update that applies it.
    ///
    /// An action sent after the app's last screen update is dropped, even if `send` succeeds.
    ///
    /// # Errors
    ///
    /// Returns [`AppError::AppClosed`] if the app has closed.
    #[track_caller]
    pub fn send(&self, action: AppAction) -> Result<()> {
        self.sender.send(action).map_err(|_| AppError::AppClosed)?;
        self.ctx.request_repaint_of(ViewportId::ROOT);
        Ok(())
    }
}
