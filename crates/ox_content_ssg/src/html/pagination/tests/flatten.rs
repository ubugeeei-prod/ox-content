//! Which sidebar entries the pager will step onto.
//!
//! An entry has to name a page of this site. Empty, external, and
//! fragment-only hrefs are all skipped, and a group header carries the last
//! of those.

use super::super::super::generate_html;
use super::{config, guide, nav_item, nav_item_with_children, page, pager_html};

#[test]
fn nested_sidebar_children_flatten_depth_first() {
    let nav = guide(vec![
        nav_item_with_children(
            "Parent",
            "parent",
            "/docs/parent/index.html",
            vec![
                nav_item("Child A", "parent/a", "/docs/parent/a/index.html"),
                nav_item("Child B", "parent/b", "/docs/parent/b/index.html"),
            ],
        ),
        nav_item("Sibling", "sibling", "/docs/sibling/index.html"),
    ]);

    let html = generate_html(&page("parent/a"), &nav, &config(true));
    let pager = pager_html(&html).expect("nested child should emit a pager");

    assert!(pager.contains(r#"href="/docs/parent/index.html""#), "{pager}");
    assert!(pager.contains("Parent"), "{pager}");
    assert!(pager.contains(r#"href="/docs/parent/b/index.html""#), "{pager}");
    assert!(pager.contains("Child B"), "{pager}");
    assert!(!pager.contains(r#"href="/docs/sibling/index.html""#), "{pager}");
}

#[test]
fn flatten_skips_items_with_empty_href() {
    let nav = guide(vec![
        nav_item("Skipped", "skipped", ""),
        nav_item("First", "first", "/docs/first/index.html"),
        nav_item("Second", "second", "/docs/second/index.html"),
    ]);

    let html = generate_html(&page("first"), &nav, &config(true));
    let pager = pager_html(&html).expect("real neighbors should still emit a pager");

    assert!(!pager.contains(r#"rel="prev""#), "{pager}");
    assert!(!pager.contains("Skipped"), "{pager}");
    assert!(pager.contains(r#"href="/docs/second/index.html""#), "{pager}");
    assert!(pager.contains("Second"), "{pager}");
}

#[test]
fn flatten_skips_external_https_hrefs() {
    let nav = guide(vec![
        nav_item("Intro", "intro", "/docs/intro/index.html"),
        nav_item("External", "external", "https://example.com/docs"),
        nav_item("API", "api", "/docs/api/index.html"),
    ]);

    let html = generate_html(&page("intro"), &nav, &config(true));
    let pager = pager_html(&html).expect("in-site neighbor should still emit a pager");

    assert!(pager.contains(r#"href="/docs/api/index.html""#), "{pager}");
    assert!(pager.contains("API"), "{pager}");
    assert!(!pager.contains("https://example.com/docs"), "{pager}");
    assert!(!pager.contains("External"), "{pager}");
}

#[test]
fn flatten_skips_group_headers_without_a_link() {
    // A sidebar group nested inside another group becomes a nav item with
    // `href="#"`. Stepping onto it left the pager pointing at `#`.
    let nav = guide(vec![
        nav_item("First", "first", "/docs/first/index.html"),
        nav_item_with_children(
            "Nested Group",
            "",
            "#",
            vec![nav_item("Child", "group/child", "/docs/group/child/index.html")],
        ),
        nav_item("Last", "last", "/docs/last/index.html"),
    ]);

    let html = generate_html(&page("first"), &nav, &config(true));
    let pager = pager_html(&html).expect("first page should emit a pager");

    assert!(!pager.contains(r##"href="#""##), "{pager}");
    assert!(!pager.contains("Nested Group"), "{pager}");
    assert!(pager.contains(r#"href="/docs/group/child/index.html""#), "{pager}");
    assert!(pager.contains("Child"), "{pager}");
}

#[test]
fn group_headers_do_not_capture_pages_with_an_empty_path() {
    // The header's own path is empty, and so is the normalized path of the
    // site root, so the root page used to resolve its neighbors from the
    // header's position in the sidebar instead of its own.
    let nav = guide(vec![
        nav_item_with_children(
            "Nested Group",
            "",
            "#",
            vec![nav_item("Child", "group/child", "/docs/group/child/index.html")],
        ),
        nav_item("Last", "last", "/docs/last/index.html"),
    ]);

    let html = generate_html(&page("/"), &nav, &config(true));

    assert!(pager_html(&html).is_none(), "a page outside the sidebar has no neighbors: {html}");
}

#[test]
fn flatten_skips_fragment_only_hrefs() {
    let nav = guide(vec![
        nav_item("Intro", "intro", "/docs/intro/index.html"),
        nav_item("Anchor", "", "#section"),
        nav_item("API", "api", "/docs/api/index.html"),
    ]);

    let html = generate_html(&page("intro"), &nav, &config(true));
    let pager = pager_html(&html).expect("in-site neighbor should still emit a pager");

    assert!(pager.contains(r#"href="/docs/api/index.html""#), "{pager}");
    assert!(!pager.contains("Anchor"), "{pager}");
}
