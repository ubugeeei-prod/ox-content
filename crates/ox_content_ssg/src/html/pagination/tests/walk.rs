//! Where the sidebar walk starts and stops.
//!
//! Resolving the pager used to flatten the whole sidebar into an owned
//! `Vec` and search it. It now walks the tree in place and stops one entry
//! past the current page, so the cases that matter are the boundaries: the
//! first entry, the last one, a match buried in a nested subtree whose
//! successor lives in a later group, and a page the sidebar does not list
//! at all — where a walk that forgot to check for a match would hand back
//! whatever entry it happened to stop on.

use std::time::{Duration, Instant};

use super::super::super::{NavGroup, generate_html};
use super::{config, guide, nav_item, nav_item_with_children, page, pager_html};

fn three_pages() -> Vec<NavGroup> {
    guide(vec![
        nav_item("One", "one", "/docs/one/index.html"),
        nav_item("Two", "two", "/docs/two/index.html"),
        nav_item("Three", "three", "/docs/three/index.html"),
    ])
}

#[test]
fn the_first_entry_has_no_previous() {
    let html = generate_html(&page("one"), &three_pages(), &config(true));
    let pager = pager_html(&html).expect("the first page still has a next link");

    assert!(!pager.contains(r#"rel="prev""#), "{pager}");
    assert!(pager.contains(r#"href="/docs/two/index.html""#), "{pager}");
}

#[test]
fn the_last_entry_has_no_next() {
    let html = generate_html(&page("three"), &three_pages(), &config(true));
    let pager = pager_html(&html).expect("the last page still has a previous link");

    assert!(!pager.contains(r#"rel="next""#), "{pager}");
    assert!(pager.contains(r#"href="/docs/two/index.html""#), "{pager}");
}

#[test]
fn a_page_the_sidebar_does_not_list_gets_no_pager() {
    let html = generate_html(&page("orphan"), &three_pages(), &config(true));

    assert!(pager_html(&html).is_none(), "an unlisted page must not borrow a neighbour: {html}");
}

#[test]
fn the_successor_may_live_in_a_later_group() {
    let nav = vec![
        guide(vec![nav_item_with_children(
            "Parent",
            "parent",
            "/docs/parent/index.html",
            vec![nav_item("Last child", "parent/z", "/docs/parent/z/index.html")],
        )])
        .remove(0),
        NavGroup {
            title: "Reference".to_string(),
            items: vec![nav_item("API", "api", "/docs/api/index.html")],
            collapsed: None,
            sticky_collapsed: None,
        },
    ];

    let html = generate_html(&page("parent/z"), &nav, &config(true));
    let pager = pager_html(&html).expect("the last child of a group has a next in the next group");

    assert!(pager.contains(r#"href="/docs/api/index.html""#), "{pager}");
    assert!(pager.contains("API"), "{pager}");
    assert!(pager.contains(r#"href="/docs/parent/index.html""#), "{pager}");
}

#[test]
fn the_match_stops_the_walk_before_a_later_group() {
    // Two entries after the match: only the first may be picked up.
    let nav = vec![
        guide(vec![
            nav_item("One", "one", "/docs/one/index.html"),
            nav_item("Two", "two", "/docs/two/index.html"),
        ])
        .remove(0),
        NavGroup {
            title: "Reference".to_string(),
            items: vec![
                nav_item("API", "api", "/docs/api/index.html"),
                nav_item("CLI", "cli", "/docs/cli/index.html"),
            ],
            collapsed: None,
            sticky_collapsed: None,
        },
    ];

    let html = generate_html(&page("two"), &nav, &config(true));
    let pager = pager_html(&html).expect("a pager is expected");

    assert!(pager.contains(r#"href="/docs/api/index.html""#), "{pager}");
    assert!(!pager.contains(r#"href="/docs/cli/index.html""#), "{pager}");
}

#[test]
fn scheme_checks_stay_case_insensitive() {
    // The lowercased copies these used to build were an allocation per
    // entry per page; the comparisons must still ignore case.
    let nav = guide(vec![
        nav_item("Upper external", "ext", "HTTPS://EXAMPLE.COM/x"),
        nav_item("Here", "here", "/docs/here/index.html"),
        nav_item("Script", "script", "JavaScript:alert(1)"),
        nav_item("After", "after", "/docs/after/index.html"),
    ]);

    let html = generate_html(&page("here"), &nav, &config(true));
    let pager = pager_html(&html).expect("a pager is expected");

    assert!(!pager.contains("EXAMPLE.COM"), "an external link is not steppable: {pager}");
    assert!(!pager.to_ascii_lowercase().contains("javascript:"), "{pager}");
    assert!(pager.contains(r#"href="/docs/after/index.html""#), "{pager}");
    assert!(!pager.contains(r#"rel="prev""#), "{pager}");
}

#[test]
fn resolving_the_pager_costs_the_same_per_page_as_the_sidebar_grows() {
    // Flattening cloned three strings per entry on every page, so a site
    // paid it once per page — quadratic in sidebar size. Four times the
    // entries may cost more per page, but not sixteen times.
    let measure = |entries: usize| {
        let items = (0..entries)
            .map(|index| {
                let path = format!("p{index}");
                let href = format!("/docs/p{index}/index.html");
                nav_item("T", &path, &href)
            })
            .collect();
        let nav = guide(items);
        let target = page(&format!("p{}", entries / 2));
        let enabled = config(true);
        let disabled = config(false);
        let start = Instant::now();
        for _ in 0..20 {
            let _ = generate_html(&target, &nav, &enabled);
        }
        let with = start.elapsed();
        let start = Instant::now();
        for _ in 0..20 {
            let _ = generate_html(&target, &nav, &disabled);
        }
        with.saturating_sub(start.elapsed())
    };

    let small = measure(500).max(Duration::from_micros(1));
    let large = measure(2_000);
    assert!(
        large < small * 16,
        "the pager cost {large:?} at 2,000 entries against {small:?} at 500"
    );
}
