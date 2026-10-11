//! Application actions: the [`AppAction`] enum, the per-frame `ActionQueue` and the cross-thread `ActionSender`.

/// A change to the app's data that the user triggers, applied by [`AppState::apply`](crate::state::AppState::apply).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AppAction {}
