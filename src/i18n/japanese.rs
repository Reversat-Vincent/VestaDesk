//! The Japanese dictionary.

use crate::i18n::Text;

/// Returns `text` in Japanese, which has the same form for every count.
pub(crate) fn text(text: Text) -> &'static str {
    match text {
        Text::Home => "ホーム",
        Text::Settings => "設定",
        Text::About => "ソフトウェア",
        Text::Version => "バージョン {version}",
        Text::Notifications => "通知 {count}件",
    }
}
