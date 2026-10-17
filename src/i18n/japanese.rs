//! The Japanese dictionary.

use crate::i18n::Text;

/// Returns `text` in Japanese.
pub(crate) fn text(text: Text) -> &'static str {
    match text {
        Text::Home => "ホーム",
        Text::Settings => "設定",
        Text::About => "ソフトウェア",
    }
}
