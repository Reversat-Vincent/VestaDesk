//! Domain state ([`AppState`]) and the pure state-transition function that applies actions ([`AppState::apply`]).

use crate::action::AppAction;
use crate::i18n::Language;
use crate::notifications::{DEFAULT_DELAY, Notifications};
use crate::router::Route;

/// All of the app's data, in one place.
///
/// [`App`](crate::app::App) holds the only instance: screens read it, and only [`AppState::apply`] changes it.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct AppState {
    /// The page shown in the central area.
    route: Route,
    /// The language of the app's texts.
    language: Language,
    /// The notifications raised and not gone yet.
    notifications: Notifications,
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

    /// Returns the language of the app's texts: [`Language::English`] by default.
    ///
    /// # Examples
    ///
    /// ```
    /// use vesta_desk::i18n::Language;
    /// use vesta_desk::state::AppState;
    ///
    /// assert_eq!(AppState::default().language(), Language::English);
    /// ```
    #[must_use]
    pub fn language(&self) -> Language {
        self.language
    }

    /// Returns the notifications raised and not gone yet: none when the app opens.
    ///
    /// # Examples
    ///
    /// ```
    /// use vesta_desk::state::AppState;
    ///
    /// assert!(AppState::default().notifications().visible().is_empty());
    /// ```
    #[must_use]
    pub fn notifications(&self) -> &Notifications {
        &self.notifications
    }

    /// Applies `action` to the data; this is the only function that changes it.
    pub fn apply(&mut self, action: AppAction) {
        match action {
            AppAction::Navigate(route) => self.route = route,
            AppAction::Notify(notification) => self.notifications.push(notification, DEFAULT_DELAY),
            AppAction::Dismiss(notification) => self.notifications.dismiss(notification),
            AppAction::Hover(notification) => self.notifications.hover(notification),
            AppAction::CountDown(elapsed) => self.notifications.count_down(elapsed),
        }
    }
}
