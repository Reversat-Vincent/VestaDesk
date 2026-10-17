//! The French dictionary.

use crate::i18n::Text;

/// Returns `text` in French.
pub(crate) fn text(text: Text) -> &'static str {
    match text {
        Text::Home => "Accueil",
        Text::Settings => "Paramètres",
        Text::About => "À propos",
    }
}
