use super::super::super::{ThemeConfig, ThemeFooter};
use super::super::{PageChromeFlags, ThemeAnnouncement};
use super::{body_class, nav_item, render};

#[test]
fn frontmatter_hides_sidebar() {
    let html =
        render(None, true, PageChromeFlags { sidebar: Some(false), ..PageChromeFlags::default() });

    assert!(!html.contains(r#"<aside class="sidebar""#), "{html}");
    assert!(html.contains(r#"<header class="header">"#), "{html}");
}

#[test]
fn frontmatter_hides_navbar() {
    let html =
        render(None, true, PageChromeFlags { navbar: Some(false), ..PageChromeFlags::default() });

    assert!(!html.contains(r#"<header class="header">"#), "{html}");
    assert!(body_class(&html).contains("ox-no-navbar"), "{html}");
    assert!(html.contains(r#"<aside class="sidebar">"#), "{html}");
}

#[test]
fn page_chrome_off_ignores_hide_flags() {
    let html = render(
        None,
        false,
        PageChromeFlags { sidebar: Some(false), navbar: Some(false), ..PageChromeFlags::default() },
    );

    assert!(html.contains(r#"<aside class="sidebar">"#), "{html}");
    assert!(html.contains(r#"<header class="header">"#), "{html}");
    assert!(!body_class(&html).contains("ox-no-navbar"), "{html}");
}

#[test]
fn frontmatter_hides_footer_outline_last_updated_and_edit_link() {
    let mut theme = ThemeConfig {
        aside: Some(true),
        footer: Some(ThemeFooter { message: Some("Built here".to_string()), copyright: None }),
        ..ThemeConfig::default()
    };
    let html = render(
        Some(theme.clone()),
        true,
        PageChromeFlags {
            outline: Some(false),
            footer: Some(false),
            last_updated: Some(false),
            edit_link: Some(false),
            ..PageChromeFlags::default()
        },
    );

    assert!(!html.contains(r#"<aside class="toc""#), "{html}");
    assert!(!html.contains("Built here"), "{html}");
    assert!(!html.contains("Last updated"), "{html}");
    assert!(body_class(&html).contains("ox-hide-edit-link"), "{html}");

    theme.aside = Some(true);
    let shown = render(Some(theme), true, PageChromeFlags::default());
    assert!(shown.contains(r#"<aside class="toc""#), "{shown}");
    assert!(shown.contains("Built here"), "{shown}");
    assert!(shown.contains("Last updated"), "{shown}");
    assert!(!body_class(&shown).contains("ox-hide-edit-link"), "{shown}");
}

#[test]
fn hostile_unclosed_input_does_not_panic() {
    let html = render(
        Some(ThemeConfig {
            nav: Some(vec![nav_item("<b>unclosed", Some("/x/\">xss"))]),
            announcement: Some(ThemeAnnouncement {
                text: "<div unclosed".into(),
                link: Some("https://example.com/\">".into()),
                dismiss_key: Some("bad key!".into()),
            }),
            ..ThemeConfig::default()
        }),
        true,
        PageChromeFlags::default(),
    );
    assert!(html.contains("&lt;b&gt;unclosed") && html.contains("&lt;div unclosed"), "{html}");
    assert!(!html.contains("<b>unclosed") && !html.contains("<div unclosed"), "{html}");
}
