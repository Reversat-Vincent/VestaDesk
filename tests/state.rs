//! The app opens on the Home page with its texts in English, and navigating changes the page and nothing else.

use vesta_desk::action::AppAction;
use vesta_desk::i18n::Language;
use vesta_desk::router::Route;
use vesta_desk::state::AppState;

#[test]
fn the_app_opens_on_the_home_page() {
    assert_eq!(AppState::default().route(), Route::Home);
}

#[test]
fn the_app_shows_its_texts_in_english_by_default() {
    assert_eq!(AppState::default().language(), Language::English);
}

#[test]
fn navigating_changes_the_page_and_nothing_else() {
    let mut state = AppState::default();
    for &route in Route::ALL {
        state.apply(AppAction::Navigate(route));
        assert_eq!(state.route(), route);
    }
    // Back on the first page, the data is as it started: navigating changed nothing else.
    state.apply(AppAction::Navigate(Route::Home));
    assert_eq!(state, AppState::default());
}
