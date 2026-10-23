//! The app opens on the Home page with its texts in English, navigating changes the page and nothing else, and a raised notification joins the list, goes after the default delay unless the pointer is over it, and goes when closed.

use std::time::Duration;

use vesta_desk::action::AppAction;
use vesta_desk::i18n::{Language, Text};
use vesta_desk::notifications::{DEFAULT_DELAY, Entry, Level, Notification};
use vesta_desk::router::Route;
use vesta_desk::state::AppState;

/// The notification that the tests raise.
const NOTIFICATION: Notification = Notification {
    level: Level::Success,
    text: Text::Settings,
    stays_until_closed: false,
};

/// Returns the notifications shown in `state`.
fn shown(state: &AppState) -> Vec<Notification> {
    let entries = state.notifications().visible();
    entries.iter().map(Entry::notification).collect()
}

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
    assert_eq!(shown(&state), [NOTIFICATION]);
    // Fewer than five are raised, so none waits.
    assert!(state.notifications().waiting().is_empty());
}

#[test]
fn a_raised_notification_goes_after_the_default_delay() {
    let mut state = AppState::default();
    state.apply(AppAction::Notify(NOTIFICATION));
    let millisecond = Duration::from_millis(1);
    let just_before = DEFAULT_DELAY.saturating_sub(millisecond);
    state.apply(AppAction::CountDown(just_before));
    assert_eq!(shown(&state), [NOTIFICATION]);
    state.apply(AppAction::CountDown(millisecond));
    assert!(shown(&state).is_empty());
}

#[test]
fn a_notification_under_the_pointer_stays_until_the_user_closes_it() {
    let mut state = AppState::default();
    state.apply(AppAction::Notify(NOTIFICATION));
    state.apply(AppAction::Hover(Some(NOTIFICATION)));
    state.apply(AppAction::CountDown(DEFAULT_DELAY));
    assert_eq!(shown(&state), [NOTIFICATION]);
    state.apply(AppAction::Dismiss(NOTIFICATION));
    assert!(shown(&state).is_empty());
}
