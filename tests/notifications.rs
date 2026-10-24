//! At most five notifications are shown while the others wait in order, and only a shown one can
//! be under the pointer. A shown one tells how much of its delay is left and goes once it has
//! passed, unless the pointer is over it or it stays until closed, and a waiting one never goes
//! before it is shown. Closing one makes room and leaves none under the pointer, and an identical
//! one is counted and restarts the countdown, even when raised as the time of the first runs out.

use std::time::Duration;

use vesta_desk::i18n::Text;
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

/// Runs a screen update of `notifications` after `elapsed`, with no action in its queue: counts them down, then removes those whose time is up.
fn update(notifications: &mut Notifications, elapsed: Duration) {
    notifications.count_down(elapsed);
    notifications.expire();
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
    update(&mut notifications, DELAY.saturating_sub(MILLISECOND));
    assert_eq!(notifications_in(notifications.visible()), [SAVED]);
    update(&mut notifications, MILLISECOND);
    assert!(notifications.visible().is_empty());
}

#[test]
fn a_waiting_notification_starts_counting_down_once_shown() {
    let raised = eight_notifications();
    let mut notifications = list_of(&raised);
    update(&mut notifications, DELAY);
    // The five shown went, and the three waiting took their places with their whole delay left.
    assert_eq!(notifications_in(notifications.visible()), &raised[5..]);
    update(&mut notifications, DELAY.saturating_sub(MILLISECOND));
    assert_eq!(notifications_in(notifications.visible()), &raised[5..]);
    update(&mut notifications, MILLISECOND);
    assert!(notifications.visible().is_empty());
}

/// Even with no delay, a waiting notification stays until it is shown.
#[test]
fn a_waiting_notification_goes_only_once_shown() {
    let raised = eight_notifications();
    let mut notifications = list_of(&raised[..5]);
    notifications.push(raised[5], Duration::ZERO);
    update(&mut notifications, SECOND);
    assert_eq!(notifications_in(notifications.waiting()), [raised[5]]);
    // Shown once another one is closed, it goes at the next screen update.
    notifications.dismiss(raised[0]);
    update(&mut notifications, Duration::ZERO);
    assert_eq!(notifications_in(notifications.visible()), &raised[1..5]);
}

#[test]
fn the_countdown_pauses_while_the_pointer_is_over_the_notification() {
    let mut notifications = list_of(&[SAVED, OTHER]);
    update(&mut notifications, SECOND);
    notifications.hover(Some(SAVED));
    update(&mut notifications, DELAY);
    // The other one went, while the one under the pointer stayed.
    assert_eq!(notifications_in(notifications.visible()), [SAVED]);
    // Once the pointer leaves it, its countdown goes on from where it paused.
    notifications.hover(None);
    update(
        &mut notifications,
        DELAY.saturating_sub(SECOND + MILLISECOND),
    );
    assert_eq!(notifications_in(notifications.visible()), [SAVED]);
    update(&mut notifications, MILLISECOND);
    assert!(notifications.visible().is_empty());
}

#[test]
fn only_a_shown_notification_can_be_under_the_pointer() {
    let raised = eight_notifications();
    let mut notifications = list_of(&raised);
    notifications.hover(Some(raised[5]));
    assert_eq!(notifications.hovered(), None);
    notifications.hover(Some(raised[4]));
    assert_eq!(notifications.hovered(), Some(raised[4]));
}

#[test]
fn closing_a_notification_shows_the_first_one_waiting_in_its_place() {
    let raised = eight_notifications();
    let mut notifications = list_of(&raised);
    notifications.dismiss(raised[1]);
    let shown = [raised[0], raised[2], raised[3], raised[4], raised[5]];
    assert_eq!(notifications_in(notifications.visible()), shown);
}

/// The pointer is over a notification when the user closes it, and an identical one raised later must still count down.
#[test]
fn closing_the_notification_under_the_pointer_leaves_none_under_it() {
    let mut notifications = list_of(&[SAVED]);
    notifications.hover(Some(SAVED));
    notifications.dismiss(SAVED);
    assert_eq!(notifications.hovered(), None);
    notifications.push(SAVED, DELAY);
    update(&mut notifications, DELAY);
    assert!(notifications.visible().is_empty());
}

#[test]
fn a_notification_that_stays_until_closed_goes_only_when_closed() {
    let mut notifications = list_of(&[FAILED]);
    update(&mut notifications, Duration::MAX);
    assert_eq!(notifications_in(notifications.visible()), [FAILED]);
    notifications.dismiss(FAILED);
    assert!(notifications.visible().is_empty());
}

#[test]
fn an_identical_notification_is_counted_and_restarts_the_countdown() {
    let mut notifications = list_of(&[SAVED]);
    update(&mut notifications, SECOND);
    notifications.push(SAVED, DELAY);
    // One notification is shown, raised twice.
    let counts: Vec<u64> = notifications.visible().iter().map(Entry::count).collect();
    assert_eq!(counts, [2]);
    // Its countdown restarted, so its whole delay is left.
    update(&mut notifications, DELAY.saturating_sub(MILLISECOND));
    assert_eq!(notifications_in(notifications.visible()), [SAVED]);
    update(&mut notifications, MILLISECOND);
    assert!(notifications.visible().is_empty());
}

/// The screens raised the repeat while the notification was still shown, during its last screen update.
#[test]
fn an_identical_notification_raised_as_the_time_runs_out_still_joins_it() {
    let mut notifications = list_of(&[SAVED]);
    // A screen update applies the actions of its queue between the countdown and the removal.
    notifications.count_down(DELAY);
    notifications.push(SAVED, DELAY);
    notifications.expire();
    let counts: Vec<u64> = notifications.visible().iter().map(Entry::count).collect();
    assert_eq!(counts, [2]);
    assert_eq!(delay_left(&notifications), [Some(1.0)]);
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

/// Only the shown notifications count down, so the app does not wake up for a waiting one.
#[test]
fn a_waiting_notification_is_not_the_next_to_go() {
    // Five shown notifications that stay until closed, so that the one raised after them waits.
    let staying: Vec<Notification> = eight_notifications()[..5]
        .iter()
        .map(|&notification| Notification {
            stays_until_closed: true,
            ..notification
        })
        .collect();
    let mut notifications = list_of(&staying);
    notifications.push(SAVED, DELAY);
    assert_eq!(notifications.next_expiry(), None);
}
