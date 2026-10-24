//! The app opens on the Home page with its texts in English, and navigating changes the page and
//! nothing else. A raised notification joins the list, goes after the default delay unless the
//! pointer is over it, and goes when closed; a screen update counts the notifications down before
//! its actions, removes those whose time is up after them, and counts no time when it comes long
//! after the next one's time is up.

use std::time::Duration;

use vesta_desk::action::AppAction;
use vesta_desk::i18n::{Language, Text};
use vesta_desk::notifications::{DEFAULT_DELAY, Entry, LATE_UPDATE, Level, Notification};
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
    // Start with a notification half counted down and under the pointer, which navigating must keep as it is.
    let mut state = AppState::default();
    state.apply(AppAction::Notify(NOTIFICATION));
    state.apply(AppAction::CountDown(DEFAULT_DELAY / 2));
    state.apply(AppAction::Hover(Some(NOTIFICATION)));
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
    state.apply(AppAction::Expire);
    assert_eq!(shown(&state), [NOTIFICATION]);
    state.apply(AppAction::CountDown(millisecond));
    // Its time is up, but it goes only once the notifications whose time is up are removed.
    assert_eq!(shown(&state), [NOTIFICATION]);
    state.apply(AppAction::Expire);
    assert!(shown(&state).is_empty());
}

#[test]
fn a_notification_under_the_pointer_stays_until_the_user_closes_it() {
    let mut state = AppState::default();
    state.apply(AppAction::Notify(NOTIFICATION));
    state.apply(AppAction::Hover(Some(NOTIFICATION)));
    state.apply(AppAction::CountDown(DEFAULT_DELAY));
    state.apply(AppAction::Expire);
    assert_eq!(shown(&state), [NOTIFICATION]);
    state.apply(AppAction::Dismiss(NOTIFICATION));
    assert!(shown(&state).is_empty());
}

/// The app runs a screen update on each frame, with the time the window had focus since the last one and the actions of the queue.
#[test]
fn a_screen_update_counts_down_before_its_actions_and_removes_after_them() {
    let mut state = AppState::default();
    // A notification raised in a screen update is not charged the time before it.
    state.update(DEFAULT_DELAY, [AppAction::Notify(NOTIFICATION)]);
    assert_eq!(state.notifications().next_expiry(), Some(DEFAULT_DELAY));
    // Raised again as its time runs out, it joins the one shown instead of coming back as a new one.
    state.update(DEFAULT_DELAY, [AppAction::Notify(NOTIFICATION)]);
    let counts: Vec<u64> = state
        .notifications()
        .visible()
        .iter()
        .map(Entry::count)
        .collect();
    assert_eq!(counts, [2]);
}

/// Windows counts the time while the computer sleeps, so the first screen update after it comes long after the next notification's time.
#[test]
fn a_screen_update_long_after_the_next_expiry_counts_no_time() {
    let mut state = AppState::default();
    state.apply(AppAction::Notify(NOTIFICATION));
    let asleep = DEFAULT_DELAY + LATE_UPDATE + Duration::from_millis(1);
    state.update(asleep, []);
    assert_eq!(state.notifications().next_expiry(), Some(DEFAULT_DELAY));
    // Up to LATE_UPDATE after the next notification's time, the time counts.
    state.update(DEFAULT_DELAY + LATE_UPDATE, []);
    assert!(shown(&state).is_empty());
}
