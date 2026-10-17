//! i18n: the dictionaries that give every [`Text`] of the app in each [`Language`].
//!
//! The screens never write a text themselves: they get it in the app's language, from the
//! dictionary of that language. Each dictionary is a `match` that gives every text, with no
//! catch-all arm, so the build fails until every dictionary has every text. A dictionary returns a
//! `&'static str` without allocating, so a text without values costs nothing on each screen update.
//!
#![doc = include_str!("../docs/assets/translating-a-text.svg")]
//!
//! # Showing a text
//!
//! Call one of these on the app's language, [`AppState::language`]:
//!
//! - [`Language::text`] gives a text as it is, such as the name of a page.
//! - [`Language::format`] fills in the values of a text, written as placeholders such as `{version}` in the dictionaries.
//! - [`Language::count`] gives the form of a text for a count, written `{count}`: English and French have a form for one item and one for other counts, Japanese has one form.
//!
//! # Adding a text
//!
//! 1. Add a variant to [`Text`], with a doc comment that says where the text is shown and names its placeholders.
//! 2. Translate it in the dictionary of each language, with the same placeholders: the build fails until every one has it.
//! 3. If it changes with a count, give its form for each [`Plural`] in the English and French dictionaries.
//! 4. Show it with [`Language::text`], [`Language::format`] or [`Language::count`].
//!
//! # Adding a language
//!
//! 1. Add a variant to [`Language`], its name, written in that language, to [`Language::name`], and its plural rule to [`Language::plural`].
//! 2. Add its dictionary, a module like `english` that gives every text, and use it in `Language::form`.
//!
//! [`AppState::language`]: crate::state::AppState::language

mod english;
mod french;
mod japanese;

enum_with_all! {
    /// A language of the app's texts.
    #[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash)]
    pub enum Language {
        /// English, the app's language by default.
        #[default]
        English,
        /// French.
        French,
        /// Japanese.
        Japanese,
    }
}

impl Language {
    /// Returns the name of the language, written in that language.
    ///
    /// # Examples
    ///
    /// ```
    /// use vesta_desk::i18n::Language;
    ///
    /// assert_eq!(Language::French.name(), "Français");
    /// ```
    #[must_use]
    pub fn name(self) -> &'static str {
        match self {
            Self::English => "English",
            Self::French => "Français",
            Self::Japanese => "日本語",
        }
    }

    /// Returns the plural form that `count` takes in this language: [`Plural::One`] for 1 in English, for 0 and 1 in French, and never in Japanese.
    ///
    /// # Examples
    ///
    /// ```
    /// use vesta_desk::i18n::{Language, Plural};
    ///
    /// assert_eq!(Language::English.plural(0), Plural::Other);
    /// assert_eq!(Language::French.plural(0), Plural::One);
    /// ```
    #[must_use]
    pub fn plural(self, count: u64) -> Plural {
        let one = match self {
            Self::English => count == 1,
            Self::French => count <= 1,
            Self::Japanese => false,
        };
        if one { Plural::One } else { Plural::Other }
    }

    /// Returns `text` in this language, as its dictionary gives it.
    ///
    /// # Examples
    ///
    /// ```
    /// use vesta_desk::i18n::{Language, Text};
    ///
    /// assert_eq!(Language::French.text(Text::Settings), "Paramètres");
    /// ```
    #[must_use]
    pub fn text(self, text: Text) -> &'static str {
        self.form(text, Plural::Other)
    }

    /// Returns `text` in this language, with each placeholder, such as `{version}`, replaced by its value in `values`; a placeholder without a value stays as it is.
    ///
    /// # Examples
    ///
    /// ```
    /// use vesta_desk::i18n::{Language, Text};
    ///
    /// let version = Language::Japanese.format(Text::Version, &[("version", "0.1.0")]);
    /// assert_eq!(version, "バージョン 0.1.0");
    /// ```
    #[must_use]
    pub fn format(self, text: Text, values: &[(&str, &str)]) -> String {
        fill(self.text(text), values)
    }

    /// Returns `text` in this language, in its form for `count`, with `{count}` replaced by `count`.
    ///
    /// # Examples
    ///
    /// ```
    /// use vesta_desk::i18n::{Language, Text};
    ///
    /// assert_eq!(Language::English.count(Text::Notifications, 1), "1 notification");
    /// assert_eq!(Language::French.count(Text::Notifications, 0), "0 notification");
    /// ```
    #[must_use]
    pub fn count(self, text: Text, count: u64) -> String {
        fill(
            self.form(text, self.plural(count)),
            &[("count", &count.to_string())],
        )
    }

    /// Returns the form of `text` for `plural` in this language, from its dictionary.
    fn form(self, text: Text, plural: Plural) -> &'static str {
        match self {
            Self::English => english::text(text, plural),
            Self::French => french::text(text, plural),
            Self::Japanese => japanese::text(text),
        }
    }
}

/// The plural form of a text that changes with a count, which [`Language::plural`] gives for each count.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Plural {
    /// The form for one item.
    One,
    /// The form for any other count, and the only form in Japanese.
    Other,
}

enum_with_all! {
    /// A text of the app, which the dictionary of each language translates.
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
    pub enum Text {
        /// The name of the Home page.
        Home,
        /// The name of the Settings page.
        Settings,
        /// The name of the About page.
        About,
        /// The version of the app, on the About page: `{version}` is its number.
        Version,
        /// A number of notifications, which changes with `{count}`.
        Notifications,
    }
}

/// Returns `template` with each placeholder, such as `{version}`, replaced by its value in `values`; a placeholder without a value stays as it is.
fn fill(template: &str, values: &[(&str, &str)]) -> String {
    let mut text = String::with_capacity(template.len());
    let mut rest = template;
    while let Some((before, after)) = rest.split_once('{')
        && let Some((name, after)) = after.split_once('}')
    {
        text.push_str(before);
        if let Some((_, value)) = values.iter().find(|(key, _)| *key == name) {
            text.push_str(value);
        } else {
            text.push('{');
            text.push_str(name);
            text.push('}');
        }
        rest = after;
    }
    text.push_str(rest);
    text
}
