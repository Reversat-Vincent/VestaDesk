//! At most five notifications are shown at once while the others wait their turn in order, and a notification is shown in the language of the moment.

use vesta_desk::i18n::{Language, Text};
use vesta_desk::notifications::{Level, Notification, Notifications};

#[test]
fn at_most_five_notifications_are_shown_and_the_others_wait_in_order() {
    // Two different notifications of each level.
    let raised: Vec<Notification> = Level::ALL
        .iter()
        .flat_map(|&level| [Text::Home, Text::About].map(|text| Notification { level, text }))
        .collect();
    let mut notifications = Notifications::default();
    for &notification in &raised {
        notifications.push(notification);
    }
    let (visible, waiting) = raised.split_at(5);
    assert_eq!(notifications.visible(), visible);
    assert_eq!(notifications.waiting(), waiting);
}

/// A notification keeps its text rather than its words, so a visible one switches to the new language as soon as the user changes it.
#[test]
fn a_notification_is_shown_in_the_language_of_the_moment() {
    let notification = Notification {
        level: Level::Info,
        text: Text::Settings,
    };
    assert_eq!(notification.message(Language::English), "Settings");
    assert_eq!(notification.message(Language::Japanese), "設定");
}
