//! Notifications ([`Notification`]) that any part of the app raises for the user, and the list that decides which are shown ([`Notifications`]).
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
//! [`AppAction::Notify`]: crate::action::AppAction::Notify
//! [`AppState::notifications`]: crate::state::AppState::notifications

use crate::i18n::{Language, Text};

/// The most notifications shown at once.
pub const MAX_VISIBLE: usize = 5;

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
    /// };
    /// assert_eq!(notification.message(Language::French), "Paramètres");
    /// ```
    #[must_use]
    pub fn message(&self, language: Language) -> &'static str {
        language.text(self.text)
    }
}

/// The notifications raised and not gone yet, in the order they were added: the first [`MAX_VISIBLE`] are shown, and the others wait their turn.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Notifications {
    /// Every notification, in the order it was added.
    list: Vec<Notification>,
}

impl Notifications {
    /// Adds `notification` after the others.
    pub fn push(&mut self, notification: Notification) {
        self.list.push(notification);
    }

    /// Returns the notifications shown: the first [`MAX_VISIBLE`], in the order they were added.
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
    /// };
    /// let mut notifications = Notifications::default();
    /// notifications.push(saved);
    /// assert_eq!(notifications.visible(), [saved]);
    /// ```
    #[must_use]
    pub fn visible(&self) -> &[Notification] {
        self.list.get(..MAX_VISIBLE).unwrap_or(&self.list)
    }

    /// Returns the notifications that wait their turn to be shown, in the order they were added.
    ///
    /// # Examples
    ///
    /// ```
    /// use vesta_desk::i18n::Text;
    /// use vesta_desk::notifications::{Level, Notification, Notifications};
    ///
    /// let mut notifications = Notifications::default();
    /// for level in [Level::Info, Level::Warning] {
    ///     for text in [Text::Home, Text::Settings, Text::About] {
    ///         notifications.push(Notification { level, text });
    ///     }
    /// }
    /// // The first five are shown, so the sixth waits.
    /// let sixth = Notification {
    ///     level: Level::Warning,
    ///     text: Text::About,
    /// };
    /// assert_eq!(notifications.waiting(), [sixth]);
    /// ```
    #[must_use]
    pub fn waiting(&self) -> &[Notification] {
        self.list.get(MAX_VISIBLE..).unwrap_or_default()
    }
}
