//! Domain state ([`AppState`]) and the pure state-transition function that applies actions ([`AppState::apply`]).

use crate::action::AppAction;

/// All of the app's data, in one place.
///
/// [`App`](crate::app::App) holds the only instance: screens read it, and only [`AppState::apply`] changes it.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct AppState {}

impl AppState {
    /// Applies `action` to the data; this is the only function that changes it.
    pub fn apply(&mut self, action: AppAction) {
        match action {}
    }
}
