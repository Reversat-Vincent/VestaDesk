//! Navigation between pages: [`Route`] names each page of the app.
//!
//! The activity bar shows one item per page, in the order of [`Route::ALL`],
//! and the central area shows the page of the current route.
//!
//! # Adding a page
//!
//! A page added to [`Route`] fails to build until it has its title, in every language, and its
//! content: [`Route::ALL`] is declared from the same list, so no page can be missing from the
//! activity bar, and every `match` on [`Route`] lists each page, with no catch-all arm.

use crate::i18n::{Language, Text};

enum_with_all! {
    /// A page of the app, which the activity bar opens.
    #[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash)]
    pub enum Route {
        /// The landing page, which the app opens on.
        #[default]
        Home,
        /// The user's preferences.
        Settings,
        /// The app's version and third-party notices.
        About,
    }
}

impl Route {
    /// Returns the name of the page in `language`.
    ///
    /// # Examples
    ///
    /// ```
    /// use vesta_desk::i18n::Language;
    /// use vesta_desk::router::Route;
    ///
    /// assert_eq!(Route::Settings.title(Language::French), "Paramètres");
    /// ```
    #[must_use]
    pub fn title(self, language: Language) -> &'static str {
        language.text(match self {
            Self::Home => Text::Home,
            Self::Settings => Text::Settings,
            Self::About => Text::About,
        })
    }
}
