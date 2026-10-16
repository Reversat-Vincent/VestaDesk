//! Domain state ([`AppState`]) and the pure state-transition function that applies actions ([`AppState::apply`]).

use crate::action::AppAction;
use crate::router::Route;
use crate::settings::Settings;

/// All of the app's data, in one place.
///
/// [`App`](crate::app::App) holds the only instance: screens read it, and only [`AppState::apply`] changes it.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct AppState {
    /// The user's preferences, including the page shown in the central area.
    settings: Settings,
}

impl AppState {
    /// Returns the data of an app that starts with `settings`, on the page they name.
    ///
    /// # Examples
    ///
    /// ```
    /// use vesta_desk::router::Route;
    /// use vesta_desk::settings::Settings;
    /// use vesta_desk::state::AppState;
    ///
    /// let settings = Settings {
    ///     page: Route::About,
    ///     ..Settings::default()
    /// };
    /// assert_eq!(AppState::new(settings).route(), Route::About);
    /// ```
    #[must_use]
    pub fn new(settings: Settings) -> Self {
        Self { settings }
    }

    /// Returns the user's preferences, which the app saves each time they change.
    ///
    /// # Examples
    ///
    /// ```
    /// use vesta_desk::action::AppAction;
    /// use vesta_desk::router::Route;
    /// use vesta_desk::state::AppState;
    ///
    /// // The settings keep the page shown, so that the app reopens on it.
    /// let mut state = AppState::default();
    /// state.apply(AppAction::Navigate(Route::About));
    /// assert_eq!(state.settings().page, Route::About);
    /// ```
    #[must_use]
    pub fn settings(&self) -> &Settings {
        &self.settings
    }

    /// Returns the page shown in the central area.
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
        self.settings.page
    }

    /// Applies `action` to the data; this is the only function that changes it.
    pub fn apply(&mut self, action: AppAction) {
        match action {
            AppAction::Navigate(route) => self.settings.page = route,
        }
    }
}
