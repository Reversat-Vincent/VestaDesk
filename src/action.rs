//! Application actions: the [`AppAction`] enum, the per-frame `ActionQueue` and the cross-thread `ActionSender`.

use crate::router::Route;

/// A change to the app's data that the user triggers, applied by [`AppState::apply`](crate::state::AppState::apply).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AppAction {
    /// Shows the page of the route in the central area.
    Navigate(Route),
}
