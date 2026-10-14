//! Domain state ([`AppState`]) and the pure state-transition function that applies actions ([`AppState::apply`]).

use crate::action::AppAction;
use crate::router::Route;

/// All of the app's data, in one place.
///
/// [`App`](crate::app::App) holds the only instance: screens read it, and only [`AppState::apply`] changes it.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct AppState {
    /// The page shown in the central area.
    route: Route,
}

impl AppState {
    /// Returns the page shown in the central area: [`Route::Home`] until the user opens another.
    ///
    /// # Examples
    ///
    /// ```
    /// use vesta_desk::action::AppAction;
    /// use vesta_desk::router::Route;
    /// use vesta_desk::state::AppState;
    ///
    /// let mut state = AppState::default();
    /// state.apply(AppAction::Navigate(Route::Settings));
    /// assert_eq!(state.route(), Route::Settings);
    /// ```
    #[must_use]
    pub fn route(&self) -> Route {
        self.route
    }

    /// Applies `action` to the data; this is the only function that changes it.
    pub fn apply(&mut self, action: AppAction) {
        match action {
            AppAction::Navigate(route) => self.route = route,
        }
    }
}
