//! i18n: the dictionaries that give every [`Text`] of the app in each [`Language`].
//!
//! The screens never write a text themselves: they get it in the app's language with
//! [`Language::text`]. Each language has its dictionary, a `match` that gives every text, with no
//! catch-all arm, so the build fails until every dictionary has every text. A dictionary returns a
//! `&'static str` without allocating, so translated texts cost nothing on each screen update.
//!
#![doc = include_str!("../docs/assets/translating-a-text.svg")]
//!
//! # Adding a text
//!
//! 1. Add a variant to [`Text`], with a doc comment that says where the text is shown.
//! 2. Translate it in the dictionary of each language: the build fails until every one has it.
//! 3. Show it with [`Language::text`], in the language of [`AppState::language`].
//!
//! # Adding a language
//!
//! 1. Add a variant to [`Language`], and its name, written in that language, to [`Language::name`].
//! 2. Add its dictionary, a module like `english` that gives every text, and return it from [`Language::text`].
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

    /// Returns `text` in this language, from its dictionary.
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
        match self {
            Self::English => english::text(text),
            Self::French => french::text(text),
            Self::Japanese => japanese::text(text),
        }
    }
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
    }
}
