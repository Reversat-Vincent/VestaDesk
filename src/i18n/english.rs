//! The English dictionary.

use crate::i18n::{Plural, Text};

/// Returns `text` in English, in its form for `plural` if it changes with a count.
pub(crate) fn text(text: Text, plural: Plural) -> &'static str {
    match text {
        Text::Home => "Home",
        Text::Settings => "Settings",
        Text::About => "About",
        Text::Version => "Version {version}",
        Text::Notifications => match plural {
            Plural::One => "{count} notification",
            Plural::Other => "{count} notifications",
        },
    }
}
