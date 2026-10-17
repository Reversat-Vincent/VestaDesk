//! The English dictionary.

use crate::i18n::Text;

/// Returns `text` in English.
pub(crate) fn text(text: Text) -> &'static str {
    match text {
        Text::Home => "Home",
        Text::Settings => "Settings",
        Text::About => "About",
    }
}
