use super::super::nav::generate_nav_html;
use super::super::{NavGroup, NavItem};

fn localized_nav(group_title: &str, item_title: &str, path: &str) -> Vec<NavGroup> {
    vec![NavGroup {
        title: group_title.to_string(),
        collapsed: Some(true),
        sticky_collapsed: Some(true),
        items: vec![NavItem {
            title: item_title.to_string(),
            path: path.to_string(),
            href: format!("/{path}.html"),
            collapsed: Some(true),
            sticky_collapsed: Some(true),
            children: vec![NavItem {
                title: "Child".to_string(),
                path: format!("{path}/child"),
                href: format!("/{path}/child.html"),
                children: vec![],
                collapsed: None,
                sticky_collapsed: None,
            }],
        }],
    }]
}

#[test]
fn sticky_keys_do_not_depend_on_translated_labels_or_paths() {
    let english = generate_nav_html(&localized_nav("Guide", "Runtime", "runtime"), "runtime");
    let japanese =
        generate_nav_html(&localized_nav("ガイド", "実行環境", "ja/runtime"), "ja/runtime");

    for html in [&english, &japanese] {
        assert!(html.contains("data-ox-nav-state-key=\"group:0\""), "{html}");
        assert!(html.contains("data-ox-nav-state-key=\"item:group:0.0\""), "{html}");
    }
    assert!(!english.contains("data-ox-nav-state-key=\"group:0:Guide"), "{english}");
    assert!(!japanese.contains("data-ox-nav-state-key=\"item:group:0.0:ja"), "{japanese}");
}
