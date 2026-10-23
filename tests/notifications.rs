//! At most five notifications are shown while the others wait in order, a shown one tells how much of its delay is left and goes after it unless the pointer is over it or it stays until closed, closing one makes room, an identical one is counted and restarts the countdown, and each is shown in the language of the moment.

use std::time::Duration;

use vesta_desk::i18n::{Language, Text};
use vesta_desk::notifications::{Entry, Level, Notification, Notifications};

/// The delay that the tests give their notifications.
const DELAY: Duration = Duration::from_secs(10);

/// One second.
const SECOND: Duration = Duration::from_secs(1);

/// One millisecond, to check a notification just before and when its time is up.
const MILLISECOND: Duration = Duration::from_millis(1);

/// A notification that goes after its delay.
const SAVED: Notification = Notification {
    level: Level::Success,
    text: Text::Settings,
    stays_until_closed: false,
};

/// Another notification that goes after its delay.
const OTHER: Notification = Notification {
    level: Level::Info,
    text: Text::Home,
    stays_until_closed: false,
};

/// A notification that stays until the user closes it.
const FAILED: Notification = Notification {
    level: Level::Error,
    text: Text::Settings,
    stays_until_closed: true,
};

/// Returns two different notifications of each level, which go after their delay.
fn eight_notifications() -> Vec<Notification> {
    Level::ALL
        .iter()
        .flat_map(|&level| {
            [Text::Home, Text::About].map(|text| Notification {
                level,
                text,
                stays_until_closed: false,
            })
        })
        .collect()
}

/// Returns the list of `raised`, added in order with the delay [`DELAY`].
fn list_of(raised: &[Notification]) -> Notifications {
    let mut notifications = Notifications::default();
    for &notification in raised {
        notifications.push(notification, DELAY);
    }
    notifications
}

/// Returns the notifications of `entries`.
fn notifications_in(entries: &[Entry]) -> Vec<Notification> {
    entries.iter().map(Entry::notification).collect()
}

/// Returns how much of their delay the notifications shown in `notifications` have left.
fn delay_left(notifications: &Notifications) -> Vec<Option<f32>> {
    notifications
        .visible()
        .iter()
        .map(Entry::delay_left)
        .collect()
}

#[test]
fn at_most_five_notifications_are_shown_and_the_others_wait_in_order() {
    let raised = eight_notifications();
    let notifications = list_of(&raised);
    let (visible, waiting) = raised.split_at(5);
    assert_eq!(notifications_in(notifications.visible()), visible);
    assert_eq!(notifications_in(notifications.waiting()), waiting);
}

#[test]
fn a_shown_notification_goes_once_its_delay_has_passed() {
    let mut notifications = list_of(&[SAVED]);
    notifications.count_down(DELAY.saturating_sub(MILLISECOND));
    assert_eq!(notifications_in(notifications.visible()), [SAVED]);
    notifications.count_down(MILLISECOND);
    assert!(notifications.visible().is_empty());
}

#[test]
fn a_waiting_notification_starts_counting_down_once_shown() {
    let raised = eight_notifications();
    let mut notifications = list_of(&raised);
    notifications.count_down(DELAY);
    // The five shown went, and the three waiting took their places with their whole delay left.
    assert_eq!(notifications_in(notifications.visible()), &raised[5..]);
    notifications.count_down(DELAY.saturating_sub(MILLISECOND));
    assert_eq!(notifications_in(notifications.visible()), &raised[5..]);
    notifications.count_down(MILLISECOND);
    assert!(notifications.visible().is_empty());
}

#[test]
fn the_countdown_pauses_while_the_pointer_is_over_the_notification() {
    let mut notifications = list_of(&[SAVED, OTHER]);
    notifications.count_down(SECOND);
    notifications.hover(Some(SAVED));
    notifications.count_down(DELAY);
    // The other one went, while the one under the pointer stayed.
    assert_eq!(notifications_in(notifications.visible()), [SAVED]);
    // Once the pointer leaves it, its countdown goes on from where it paused.
    notifications.hover(None);
    notifications.count_down(DELAY.saturating_sub(SECOND + MILLISECOND));
    assert_eq!(notifications_in(notifications.visible()), [SAVED]);
    notifications.count_down(MILLISECOND);
    assert!(notifications.visible().is_empty());
}

#[test]
fn closing_a_notification_shows_the_first_one_waiting_in_its_place() {
    let raised = eight_notifications();
    let mut notifications = list_of(&raised);
    notifications.dismiss(raised[1]);
    let shown = [raised[0], raised[2], raised[3], raised[4], raised[5]];
    assert_eq!(notifications_in(notifications.visible()), shown);
}

#[test]
fn a_notification_that_stays_until_closed_goes_only_when_closed() {
    let mut notifications = list_of(&[FAILED]);
    notifications.count_down(Duration::MAX);
    assert_eq!(notifications_in(notifications.visible()), [FAILED]);
    notifications.dismiss(FAILED);
    assert!(notifications.visible().is_empty());
}

#[test]
fn an_identical_notification_is_counted_and_restarts_the_countdown() {
    let mut notifications = list_of(&[SAVED]);
    notifications.count_down(SECOND);
    notifications.push(SAVED, DELAY);
    // One notification is shown, raised twice.
    let counts: Vec<u64> = notifications.visible().iter().map(Entry::count).collect();
    assert_eq!(counts, [2]);
    // Its countdown restarted, so its whole delay is left.
    notifications.count_down(DELAY.saturating_sub(MILLISECOND));
    assert_eq!(notifications_in(notifications.visible()), [SAVED]);
    notifications.count_down(MILLISECOND);
    assert!(notifications.visible().is_empty());
}

#[test]
fn a_shown_notification_tells_how_much_of_its_delay_is_left() {
    let mut notifications = list_of(&[SAVED, FAILED]);
    assert_eq!(delay_left(&notifications), [Some(1.0), None]);
    notifications.count_down(DELAY / 2);
    // A notification that stays until closed has no time left to show.
    assert_eq!(delay_left(&notifications), [Some(0.5), None]);
}

#[test]
fn the_delay_left_stops_changing_while_the_pointer_is_over_the_notification() {
    let mut notifications = list_of(&[SAVED]);
    notifications.count_down(DELAY / 2);
    notifications.hover(Some(SAVED));
    notifications.count_down(DELAY / 4);
    assert_eq!(delay_left(&notifications), [Some(0.5)]);
}

/// The app wakes up at the next expiry, so that notifications go even if the user does nothing.
#[test]
fn the_next_expiry_is_when_the_next_shown_notification_goes() {
    let mut notifications = Notifications::default();
    assert_eq!(notifications.next_expiry(), None);
    notifications.push(SAVED, DELAY);
    notifications.count_down(SECOND);
    notifications.push(OTHER, DELAY);
    assert_eq!(
        notifications.next_expiry(),
        Some(DELAY.saturating_sub(SECOND))
    );
    // Neither the notification under the pointer nor one that stays until closed counts down.
    notifications.hover(Some(SAVED));
    assert_eq!(notifications.next_expiry(), Some(DELAY));
    notifications.dismiss(OTHER);
    notifications.push(FAILED, DELAY);
    assert_eq!(notifications.next_expiry(), None);
}

/// A notification keeps its text rather than its words, so a visible one switches to the new language as soon as the user changes it.
#[test]
fn a_notification_is_shown_in_the_language_of_the_moment() {
    assert_eq!(SAVED.message(Language::English), "Settings");
    assert_eq!(SAVED.message(Language::Japanese), "設定");
}
