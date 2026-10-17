//! Every text exists in every language with the same placeholders, each language has its name
//! written in that language, values fill in their placeholders, and a count takes the plural form of each language.

use vesta_desk::i18n::{Language, Text};

/// Returns the names of the placeholders of `text`, such as `version` in "Version {version}", sorted.
fn placeholders(text: &str) -> Vec<&str> {
    let mut names: Vec<&str> = text
        .split('{')
        .skip(1)
        .filter_map(|part| part.split_once('}'))
        .map(|(name, _)| name)
        .collect();
    names.sort_unstable();
    names
}

#[test]
fn every_text_exists_in_every_language() {
    for &language in Language::ALL {
        for &text in Text::ALL {
            // A count of 1 gives the form of a text for one item, and 2 its form for other counts.
            for count in [1, 2] {
                assert!(
                    !language.count(text, count).trim().is_empty(),
                    "{text:?} is missing in {language:?}"
                );
            }
        }
    }
}

/// The order of the placeholders can change with the language, but not their names.
#[test]
fn the_placeholders_of_each_text_are_the_same_in_every_language() {
    for &text in Text::ALL {
        let english = placeholders(Language::English.text(text));
        for &language in Language::ALL {
            assert_eq!(
                placeholders(language.text(text)),
                english,
                "{text:?} in {language:?}"
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

#[test]
fn a_value_fills_its_placeholder_and_a_missing_one_stays_visible() {
    let version = Language::English.format(Text::Version, &[("version", "0.1.0")]);
    assert_eq!(version, "Version 0.1.0");
    assert_eq!(
        Language::English.format(Text::Version, &[]),
        "Version {version}"
    );
}

/// French uses the form for one item for 0 too, and Japanese has a single form.
#[test]
fn a_count_takes_the_plural_form_of_each_language() {
    let notifications =
        |language: Language| [0, 1, 2].map(|count| language.count(Text::Notifications, count));
    assert_eq!(
        notifications(Language::English),
        ["0 notifications", "1 notification", "2 notifications"]
    );
    assert_eq!(
        notifications(Language::French),
        ["0 notification", "1 notification", "2 notifications"]
    );
    assert_eq!(
        notifications(Language::Japanese),
        ["通知 0件", "通知 1件", "通知 2件"]
    );
}
