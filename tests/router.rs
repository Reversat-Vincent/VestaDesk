//! Every page has its own title in every language, so a page copied from another cannot keep the other's name.

use std::collections::HashSet;

use vesta_desk::i18n::Language;
use vesta_desk::router::Route;

#[test]
fn every_page_has_its_own_title_in_every_language() {
    for &language in Language::ALL {
        let titles: HashSet<&str> = Route::ALL
            .iter()
            .map(|route| route.title(language))
            .collect();
        assert_eq!(
            titles.len(),
            Route::ALL.len(),
            "two pages share a title in {language:?}"
        );
    }
}
