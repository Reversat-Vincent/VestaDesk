//! Every text exists in every language, and each language has its name written in that language.

use vesta_desk::i18n::{Language, Text};

#[test]
fn every_text_exists_in_every_language() {
    for &language in Language::ALL {
        for &text in Text::ALL {
            assert!(
                !language.text(text).trim().is_empty(),
                "{text:?} is missing in {language:?}"
            );
        }
    }
}

#[test]
fn each_language_has_its_name_written_in_that_language() {
    let names: Vec<&str> = Language::ALL
        .iter()
        .map(|language| language.name())
        .collect();
    assert_eq!(names, ["English", "Français", "日本語"]);
}
