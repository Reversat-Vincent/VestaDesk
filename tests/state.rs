//! The app opens on the Home page with its texts in English, navigating changes the page and nothing else, and raising a notification adds it to the list.

use vesta_desk::action::AppAction;
use vesta_desk::i18n::{Language, Text};
use vesta_desk::notifications::{Level, Notification};
use vesta_desk::router::Route;
use vesta_desk::state::AppState;

/// The notification that the tests raise.
const NOTIFICATION: Notification = Notification {
    level: Level::Success,
    text: Text::Settings,
};

#[test]
fn the_app_opens_on_the_home_page() {
    assert_eq!(AppState::default().route(), Route::Home);
}

#[test]
fn the_app_shows_its_texts_in_english_by_default() {
    assert_eq!(AppState::default().language(), Language::English);
}

#[test]
fn navigating_changes_the_page_and_nothing_else() {
    // Start with a notification, which navigating must keep.
    let mut state = AppState::default();
    state.apply(AppAction::Notify(NOTIFICATION));
    let start = state.clone();
    for &route in Route::ALL {
        state.apply(AppAction::Navigate(route));
        assert_eq!(state.route(), route);
    }
    // Back on the first page, the data is as it started: navigating changed nothing else.
    state.apply(AppAction::Navigate(Route::Home));
    assert_eq!(state, start);
}

#[test]
fn raising_a_notification_adds_it_to_the_list() {
    let mut state = AppState::default();
    state.apply(AppAction::Notify(NOTIFICATION));
    assert_eq!(state.notifications().visible(), [NOTIFICATION]);
    // Fewer than five are raised, so none waits.
    assert!(state.notifications().waiting().is_empty());
}
