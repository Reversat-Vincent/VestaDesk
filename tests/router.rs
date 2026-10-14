//! Every page has its own title, so a page copied from another cannot keep the other's name.

use std::collections::HashSet;

use vesta_desk::router::Route;

#[test]
fn every_page_has_its_own_title() {
    let titles: HashSet<&str> = Route::ALL.iter().map(|route| route.title()).collect();
    assert_eq!(titles.len(), Route::ALL.len(), "two pages share a title");
    assert!(!titles.contains(""), "a page has no title");
}
