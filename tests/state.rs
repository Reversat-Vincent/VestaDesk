//! The app opens on the page its settings name, the Home page by default, and navigating changes the page and nothing else.

use vesta_desk::action::AppAction;
use vesta_desk::router::Route;
use vesta_desk::settings::Settings;
use vesta_desk::state::AppState;

#[test]
fn by_default_the_app_opens_on_the_home_page() {
    assert_eq!(AppState::default().route(), Route::Home);
}

#[test]
fn the_app_opens_on_the_page_its_settings_name() {
    for &page in Route::ALL {
        let settings = Settings {
            page,
            ..Settings::default()
        };
        assert_eq!(AppState::new(settings).route(), page);
    }
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
