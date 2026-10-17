//! The French dictionary.

use crate::i18n::{Plural, Text};

/// Returns `text` in French, in its form for `plural` if it changes with a count.
pub(crate) fn text(text: Text, plural: Plural) -> &'static str {
    match text {
        Text::Home => "Accueil",
        Text::Settings => "Paramètres",
        Text::About => "À propos",
        Text::Version => "Version {version}",
        Text::Notifications => match plural {
            Plural::One => "{count} notification",
            Plural::Other => "{count} notifications",
        },
    }
}
