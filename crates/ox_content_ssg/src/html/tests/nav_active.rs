//! Which sidebar entry a page marks as current.

use super::super::nav::generate_nav_html;
use super::super::{NavGroup, NavItem};

#[test]
fn test_generate_nav_html_marks_no_group_active_without_a_route() {
    // `404.html` is generated with an empty url path, and a sidebar group
    // without a link of its own has an empty path too.
    let nav_groups = vec![NavGroup {
        title: "Guide".to_string(),
        collapsed: Some(false),
        sticky_collapsed: None,
        items: vec![
            NavItem {
                title: "Group".to_string(),
                path: String::new(),
                href: "#".to_string(),
                collapsed: Some(false),
                sticky_collapsed: None,
                children: vec![NavItem {
                    title: "Getting started".to_string(),
                    path: "getting-started".to_string(),
                    href: "/docs/getting-started/index.html".to_string(),
                    children: vec![],
                    collapsed: None,
                    sticky_collapsed: None,
                }],
            },
            NavItem {
                title: "Reference".to_string(),
                path: String::new(),
                href: "#".to_string(),
                collapsed: Some(false),
                sticky_collapsed: None,
                children: vec![NavItem {
                    title: "API".to_string(),
                    path: "api".to_string(),
                    href: "/docs/api/index.html".to_string(),
                    children: vec![],
                    collapsed: None,
                    sticky_collapsed: None,
                }],
            },
        ],
    }];

    let not_found = generate_nav_html(&nav_groups, "");
    assert!(!not_found.contains("active"), "{not_found}");

    let page = generate_nav_html(&nav_groups, "api");
    assert_eq!(page.matches(" active").count(), 1, "{page}");
    assert!(page.contains(r#"class="nav-link active">API"#), "{page}");
}
