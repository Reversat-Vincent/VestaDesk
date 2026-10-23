//! Notifications ([`Notification`]) that any part of the app raises for the user, and the list that decides which are shown and for how long ([`Notifications`]).
//!
//! A screen raises a notification by pushing [`AppAction::Notify`] to the action queue, and a
//! background task by sending it, like any other [action](crate::action): the next screen update
//! adds it to [`AppState::notifications`]. The first [`MAX_VISIBLE`] notifications added are shown,
//! and the others wait their turn, in the order they were added.
//!
//! A notification keeps its [`Text`], not its words in one language: it is translated each time it
//! is shown, so a visible notification switches to the new language as soon as the user changes it.
//!
//! VestaDesk has its own notifications rather than those of a ready-made library, so that they follow the app's own colours and translations.
//!
//! # How long a notification stays
//!
//! A shown notification goes once its delay, [`DEFAULT_DELAY`], has passed; a waiting one starts
//! counting down once it is shown. The countdown pauses while the pointer is over the notification,
//! and the user can close it at any time. A notification identical to one already in the list is
//! not added again: the one in the list counts it and restarts its countdown, so each notification
//! is in the list once at most.
//!
//! The delay is the same for every level, errors included. The code that raises a notification that must not be missed, such as a failed save, makes it stay until the user closes it.
//!
//! [`AppAction::Notify`]: crate::action::AppAction::Notify
//! [`AppState::notifications`]: crate::state::AppState::notifications

use std::time::Duration;

use crate::i18n::{Language, Text};

/// The most notifications shown at once.
pub const MAX_VISIBLE: usize = 5;

/// How long a shown notification stays by default: 3 s.
pub const DEFAULT_DELAY: Duration = Duration::from_secs(3);

enum_with_all! {
    /// How important a notification is.
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
    pub enum Level {
        /// Information, which needs no action.
        Info,
        /// An operation that worked.
        Success,
        /// A problem that the app worked around.
        Warning,
        /// An operation that failed.
        Error,
    }
}

/// A message for the user, raised with [`AppAction::Notify`](crate::action::AppAction::Notify).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Notification {
    /// How important the message is.
    pub level: Level,
    /// The message, translated each time it is shown.
    pub text: Text,
    /// Whether it stays until the user closes it, instead of going after the delay.
    pub stays_until_closed: bool,
}

impl Notification {
    /// Returns the message in `language`.
    ///
    /// # Examples
    ///
    /// ```
    /// use vesta_desk::i18n::{Language, Text};
    /// use vesta_desk::notifications::{Level, Notification};
    ///
    /// let notification = Notification {
    ///     level: Level::Info,
    ///     text: Text::Settings,
    ///     stays_until_closed: false,
    /// };
    /// assert_eq!(notification.message(Language::French), "Paramètres");
    /// ```
    #[must_use]
    pub fn message(&self, language: Language) -> &'static str {
        language.text(self.text)
    }
}

/// A notification in the [list](Notifications), with the number of times it was raised and its countdown.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Entry {
    /// The notification.
    notification: Notification,
    /// How many times it was raised.
    count: u64,
    /// Its countdown, which runs only while it is shown, or `None` if it stays until closed.
    countdown: Option<Countdown>,
}

impl Entry {
    /// Returns the notification.
    ///
    /// # Examples
    ///
    /// ```
    /// use vesta_desk::i18n::Text;
    /// use vesta_desk::notifications::{DEFAULT_DELAY, Level, Notification, Notifications};
    ///
    /// let saved = Notification {
    ///     level: Level::Success,
    ///     text: Text::Settings,
    ///     stays_until_closed: false,
    /// };
    /// let mut notifications = Notifications::default();
    /// notifications.push(saved, DEFAULT_DELAY);
    /// assert_eq!(notifications.visible()[0].notification(), saved);
    /// ```
    #[must_use]
    pub fn notification(&self) -> Notification {
        self.notification
    }

    /// Returns how many times the notification was raised: more than once when identical ones joined it.
    ///
    /// # Examples
    ///
    /// ```
    /// use vesta_desk::i18n::Text;
    /// use vesta_desk::notifications::{DEFAULT_DELAY, Level, Notification, Notifications};
    ///
    /// let failed = Notification {
    ///     level: Level::Error,
    ///     text: Text::Settings,
    ///     stays_until_closed: true,
    /// };
    /// let mut notifications = Notifications::default();
    /// notifications.push(failed, DEFAULT_DELAY);
    /// notifications.push(failed, DEFAULT_DELAY);
    /// assert_eq!(notifications.visible()[0].count(), 2);
    /// ```
    #[must_use]
    pub fn count(&self) -> u64 {
        self.count
    }

    /// Returns how much of its delay the notification has left, from 1 down to 0 when it goes, or `None` if it stays until closed.
    ///
    /// # Examples
    ///
    /// ```
    /// use std::time::Duration;
    ///
    /// use vesta_desk::i18n::Text;
    /// use vesta_desk::notifications::{Level, Notification, Notifications};
    ///
    /// let saved = Notification {
    ///     level: Level::Success,
    ///     text: Text::Settings,
    ///     stays_until_closed: false,
    /// };
    /// let mut notifications = Notifications::default();
    /// notifications.push(saved, Duration::from_secs(4));
    /// notifications.count_down(Duration::from_secs(1));
    /// assert_eq!(notifications.visible()[0].delay_left(), Some(0.75));
    /// ```
    #[must_use]
    pub fn delay_left(&self) -> Option<f32> {
        self.countdown.map(|countdown| {
            // Nothing is left of a zero delay, and dividing by it would give no number.
            if countdown.delay.is_zero() {
                0.0
            } else {
                countdown.left.div_duration_f32(countdown.delay)
            }
        })
    }

    /// Returns whether the notification's time is up: never if it stays until closed.
    fn time_is_up(&self) -> bool {
        self.countdown
            .is_some_and(|countdown| countdown.left.is_zero())
    }
}

/// The delay after which a notification goes, and how much of it is left.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct Countdown {
    /// How long the notification stays once shown.
    delay: Duration,
    /// How much of the delay is left.
    left: Duration,
}

/// The notifications raised and not gone yet, in the order they were added: the first [`MAX_VISIBLE`] are shown, and the others wait their turn.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Notifications {
    /// Every notification, in the order it was added.
    list: Vec<Entry>,
    /// The notification under the pointer, whose countdown is paused.
    hovered: Option<Notification>,
}

impl Notifications {
    /// Adds `notification` after the others, to stay for `delay` once shown; if an identical one is in the list, that one counts it and restarts its countdown instead.
    pub fn push(&mut self, notification: Notification, delay: Duration) {
        // A notification that stays until closed has no countdown.
        let countdown =
            (!notification.stays_until_closed).then_some(Countdown { delay, left: delay });
        if let Some(entry) = self
            .list
            .iter_mut()
            .find(|entry| entry.notification == notification)
        {
            entry.count += 1;
            entry.countdown = countdown;
        } else {
            self.list.push(Entry {
                notification,
                count: 1,
                countdown,
            });
        }
    }

    /// Removes `notification`, so that the first one waiting takes its place.
    pub fn dismiss(&mut self, notification: Notification) {
        self.list.retain(|entry| entry.notification != notification);
    }

    /// Sets the notification under the pointer, or none: its countdown pauses until the pointer leaves it.
    pub fn hover(&mut self, notification: Option<Notification>) {
        self.hovered = notification;
    }

    /// Counts the shown notifications down by `elapsed`: those whose time is up go, and the first ones waiting take their places.
    pub fn count_down(&mut self, elapsed: Duration) {
        let hovered = self.hovered;
        for entry in self.list.iter_mut().take(MAX_VISIBLE) {
            if hovered != Some(entry.notification)
                && let Some(countdown) = &mut entry.countdown
            {
                countdown.left = countdown.left.saturating_sub(elapsed);
            }
        }
        self.list.retain(|entry| !entry.time_is_up());
    }

    /// Returns the notification under the pointer, if any.
    ///
    /// # Examples
    ///
    /// ```
    /// use vesta_desk::i18n::Text;
    /// use vesta_desk::notifications::{Level, Notification, Notifications};
    ///
    /// let saved = Notification {
    ///     level: Level::Success,
    ///     text: Text::Settings,
    ///     stays_until_closed: false,
    /// };
    /// let mut notifications = Notifications::default();
    /// notifications.hover(Some(saved));
    /// assert_eq!(notifications.hovered(), Some(saved));
    /// ```
    #[must_use]
    pub fn hovered(&self) -> Option<Notification> {
        self.hovered
    }

    /// Returns the time left until the next shown notification goes, or `None` if none of them counts down.
    ///
    /// # Examples
    ///
    /// ```
    /// use std::time::Duration;
    ///
    /// use vesta_desk::i18n::Text;
    /// use vesta_desk::notifications::{Level, Notification, Notifications};
    ///
    /// let saved = Notification {
    ///     level: Level::Success,
    ///     text: Text::Settings,
    ///     stays_until_closed: false,
    /// };
    /// let mut notifications = Notifications::default();
    /// notifications.push(saved, Duration::from_secs(3));
    /// notifications.count_down(Duration::from_secs(1));
    /// assert_eq!(notifications.next_expiry(), Some(Duration::from_secs(2)));
    /// ```
    #[must_use]
    pub fn next_expiry(&self) -> Option<Duration> {
        self.visible()
            .iter()
            .filter(|entry| self.hovered != Some(entry.notification))
            .filter_map(|entry| entry.countdown)
            .map(|countdown| countdown.left)
            .min()
    }

    /// Returns the notifications shown: the first [`MAX_VISIBLE`], in the order they were added.
    ///
    /// # Examples
    ///
    /// ```
    /// use vesta_desk::i18n::Text;
    /// use vesta_desk::notifications::{DEFAULT_DELAY, Level, Notification, Notifications};
    ///
    /// let saved = Notification {
    ///     level: Level::Success,
    ///     text: Text::Settings,
    ///     stays_until_closed: false,
    /// };
    /// let mut notifications = Notifications::default();
    /// notifications.push(saved, DEFAULT_DELAY);
    /// assert_eq!(notifications.visible().len(), 1);
    /// ```
    #[must_use]
    pub fn visible(&self) -> &[Entry] {
        self.list.get(..MAX_VISIBLE).unwrap_or(&self.list)
    }

    /// Returns the notifications that wait their turn to be shown, in the order they were added.
    ///
    /// # Examples
    ///
    /// ```
    /// use vesta_desk::i18n::Text;
    /// use vesta_desk::notifications::{DEFAULT_DELAY, Level, Notification, Notifications};
    ///
    /// let mut notifications = Notifications::default();
    /// for level in [Level::Info, Level::Warning] {
    ///     for text in [Text::Home, Text::Settings, Text::About] {
    ///         let notification = Notification { level, text, stays_until_closed: false };
    ///         notifications.push(notification, DEFAULT_DELAY);
    ///     }
    /// }
    /// // The first five are shown, so the sixth waits.
    /// assert_eq!(notifications.waiting().len(), 1);
    /// ```
    #[must_use]
    pub fn waiting(&self) -> &[Entry] {
        self.list.get(MAX_VISIBLE..).unwrap_or_default()
    }
}
