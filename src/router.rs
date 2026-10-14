//! Navigation between pages: [`Route`] names each page of the app.
//!
//! The activity bar shows one item per page, in the order of [`Route::ALL`],
//! and the central area shows the page of the current route.
//!
//! # Adding a page
//!
//! A page added to the `routes!` list fails to build until it has its title and its content:
//! `routes!` declares [`Route`] and [`Route::ALL`] from one list, so no page can be missing from the
//! activity bar, and every `match` on [`Route`] lists each page, with no catch-all arm.

/// Declares [`Route`], with one variant per page, and [`Route::ALL`], which lists them in the same order.
macro_rules! routes {
    ($($(#[$attribute:meta])* $route:ident,)+) => {
        /// A page of the app, which the activity bar opens.
        #[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash)]
        pub enum Route {
            $($(#[$attribute])* $route,)+
        }

        impl Route {
            /// Every page, in the order of the activity bar.
            pub const ALL: &[Self] = &[$(Self::$route),+];
        }
    };
}

routes! {
    /// The landing page, which the app opens on.
    #[default]
    Home,
    /// The user's preferences.
    Settings,
    /// The app's version and third-party notices.
    About,
}

impl Route {
    /// Returns the name of the page.
    ///
    /// # Examples
    ///
    /// ```
    /// use vesta_desk::router::Route;
    ///
    /// assert_eq!(Route::Settings.title(), "Settings");
    /// ```
    #[must_use]
    pub fn title(self) -> &'static str {
        match self {
            Self::Home => "Home",
            Self::Settings => "Settings",
            Self::About => "About",
        }
    }
}
