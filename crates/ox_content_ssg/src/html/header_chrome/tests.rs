use super::super::{
    HeaderNavItem, NavGroup, NavItem, PageChromeFlags, PageData, ReaderChrome, SsgConfig,
    ThemeAnnouncement, ThemeConfig, ThemeFooter, TocEntry, generate_html,
};
use super::HEADER_CHROME_JS;

fn nav_item(text: &str, link: Option<&str>) -> HeaderNavItem {
    HeaderNavItem { text: text.to_string(), link: link.map(ToOwned::to_owned), items: vec![] }
}

fn dropdown(text: &str, items: Vec<HeaderNavItem>) -> HeaderNavItem {
    HeaderNavItem { text: text.to_string(), link: None, items }
}

fn page() -> PageData {
    PageData {
        title: "Guide".to_string(),
        description: None,
        content: "<p class=\"ox-edit-this-page\"><a href=\"https://example.com/edit\">Edit this page</a></p>"
            .to_string(),
        toc: vec![TocEntry { depth: 1, text: "Hello".to_string(), slug: "hello".to_string() }],
        last_updated: Some(0),
        path: "guide".to_string(),
        entry_page: None,
        prev: None,
        next: None,
        breadcrumbs: None,
        chrome: PageChromeFlags::default(),
    }
}

fn sidebar() -> Vec<NavGroup> {
    vec![NavGroup {
        title: "Guide".to_string(),
        items: vec![NavItem {
            title: "Guide".to_string(),
            path: "guide".to_string(),
            href: "/docs/guide/index.html".to_string(),
            children: vec![],
            collapsed: None,
            sticky_collapsed: None,
        }],
        collapsed: None,
        sticky_collapsed: None,
    }]
}

fn config(theme: Option<ThemeConfig>, page_chrome: bool) -> SsgConfig {
    SsgConfig {
        site_name: "Docs".to_string(),
        base: "/docs/".to_string(),
        og_image: None,
        theme,
        locale: None,
        available_locales: None,
        pagination: false,
        breadcrumbs: false,
        reader_chrome: ReaderChrome::default(),
        locale_switcher: false,
        locale_paths: vec![],
        a11y: crate::A11y::default(),
        page_chrome,
    }
}

fn render(theme: Option<ThemeConfig>, page_chrome: bool, chrome: PageChromeFlags) -> String {
    let mut page_data = page();
    page_data.chrome = chrome;
    generate_html(&page_data, &sidebar(), &config(theme, page_chrome))
}

fn theme_nav(items: Vec<HeaderNavItem>) -> ThemeConfig {
    ThemeConfig { nav: Some(items), ..ThemeConfig::default() }
}

#[test]
fn nav_disabled_by_default() {
    let html = render(None, false, PageChromeFlags::default());

    assert!(!html.contains(r#"<nav class="header-nav""#), "{html}");
    assert!(!html.contains(r#"<div class="ox-announce""#), "{html}");
    assert!(!html.contains(r#"class="ox-has-announce""#), "{html}");
    assert!(!html.contains(r#"class="ox-no-navbar""#), "{html}");
    assert!(!html.contains("data-ox-announce"), "{html}");
    assert!(html.contains(r#"<header class="header">"#), "{html}");
    assert!(html.contains(r#"<aside class="sidebar">"#), "{html}");
}

#[test]
fn nav_happy_path_links() {
    let html = render(
        Some(theme_nav(vec![
            nav_item("Guide", Some("/guide/")),
            nav_item("API", Some("https://example.com/api")),
        ])),
        false,
        PageChromeFlags::default(),
    );
    let nav = html.find(r#"<nav class="header-nav""#).map(|i| &html[i..]).expect("header nav");

    assert!(nav.contains(r#"aria-label="Header""#), "{nav}");
    assert!(nav.contains(r#"href="/guide/""#), "{nav}");
    assert!(nav.contains(">Guide</a>"), "{nav}");
    assert!(nav.contains(r#"href="https://example.com/api""#), "{nav}");
    assert!(nav.contains(">API</a>"), "{nav}");
    assert!(!nav.contains("aria-expanded"), "{nav}");
}

#[test]
fn nav_dropdown_markup_has_aria() {
    let html = render(
        Some(theme_nav(vec![dropdown(
            "API",
            vec![nav_item("SSG", Some("/api/ssg/")), nav_item("Search", Some("/api/search/"))],
        )])),
        false,
        PageChromeFlags::default(),
    );
    let nav = html.find(r#"<nav class="header-nav""#).map(|i| &html[i..]).expect("header nav");

    assert!(
        nav.contains(
            r#"<button type="button" aria-expanded="false" aria-haspopup="true">API</button>"#
        ),
        "{nav}"
    );
    assert!(nav.contains("header-nav-dropdown"), "{nav}");
    assert!(nav.contains("header-nav-menu"), "{nav}");
    assert!(nav.contains(r#"href="/api/ssg/""#), "{nav}");
    assert!(nav.contains(">SSG</a>"), "{nav}");
    assert!(html.contains("aria-expanded"), "{html}");
    assert!(html.contains("Escape"), "{html}");
}

#[test]
fn nav_javascript_href_rejected() {
    let html = render(
        Some(theme_nav(vec![
            nav_item("Safe", Some("/guide/")),
            nav_item("Evil", Some("javascript:alert(1)")),
            nav_item("Data", Some("data:text/html,hi")),
            nav_item("Vb", Some("vbscript:msgbox(1)")),
            nav_item("Proto", Some("//evil.example/x")),
        ])),
        false,
        PageChromeFlags::default(),
    );

    assert!(html.contains(r#"href="/guide/""#), "{html}");
    assert!(html.contains(">Safe</a>"), "{html}");
    assert!(!html.contains("javascript:"), "{html}");
    assert!(!html.contains("data:text/html"), "{html}");
    assert!(!html.contains("vbscript:"), "{html}");
    assert!(!html.contains("//evil.example/x"), "{html}");
    assert!(!html.contains(">Evil<"), "{html}");
}

#[test]
fn announcement_escaped() {
    let html = render(
        Some(ThemeConfig {
            announcement: Some(ThemeAnnouncement {
                text: "<script>alert(1)</script>".to_string(),
                link: Some("https://example.com/news".to_string()),
                dismiss_key: Some("welcome".to_string()),
            }),
            ..ThemeConfig::default()
        }),
        false,
        PageChromeFlags::default(),
    );

    assert!(html.contains(r#"<div class="ox-announce""#), "{html}");
    assert!(body_class(&html).contains("ox-has-announce"), "{html}");
    assert!(html.contains(r#"data-ox-announce="welcome""#), "{html}");
    assert!(html.contains("parentElement"), "{html}");
    assert!(html.contains("&lt;script&gt;alert(1)&lt;/script&gt;"), "{html}");
    assert!(!html.contains("<script>alert(1)</script>"), "{html}");
    assert!(html.contains(r#"href="https://example.com/news""#), "{html}");
}

#[test]
fn announcement_javascript_link_rejected() {
    let html = render(
        Some(ThemeConfig {
            announcement: Some(ThemeAnnouncement {
                text: "Ship day".to_string(),
                link: Some("javascript:alert(1)".to_string()),
                dismiss_key: None,
            }),
            ..ThemeConfig::default()
        }),
        false,
        PageChromeFlags::default(),
    );

    assert!(html.contains("Ship day"), "{html}");
    assert!(!html.contains("javascript:"), "{html}");
    assert!(!announce_html(&html).contains("<a "), "{}", announce_html(&html));
}

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
fn nav_label_is_escaped() {
    let html = render(
        Some(theme_nav(vec![nav_item("<img src=x onerror=alert(1)>", Some("/safe/"))])),
        false,
        PageChromeFlags::default(),
    );

    assert!(html.contains("&lt;img src=x onerror=alert(1)&gt;"), "{html}");
    assert!(!html.contains("<img src=x"), "{html}");
    assert!(html.contains(r#"href="/safe/""#), "{html}");
}

#[test]
fn announcement_http_and_protocol_relative_rejected() {
    for link in ["http://evil.example/x", "//evil.example/x", "data:text/html,hi"] {
        let html = render(
            Some(ThemeConfig {
                announcement: Some(ThemeAnnouncement {
                    text: "Notice".to_string(),
                    link: Some(link.to_string()),
                    dismiss_key: None,
                }),
                ..ThemeConfig::default()
            }),
            false,
            PageChromeFlags::default(),
        );
        let bar = announce_html(&html);
        assert!(bar.contains("Notice"), "{bar}");
        assert!(!bar.contains("<a "), "{link} {bar}");
        assert!(!html.contains(link), "{html}");
    }
}

fn announce_html(html: &str) -> &str {
    let start = html.find(r#"<div class="ox-announce""#).expect("announcement bar");
    let rest = &html[start..];
    let end = rest.find("</div>").expect("announcement close");
    &rest[..end]
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

fn body_class(html: &str) -> &str {
    let start = html.find("<body").expect("body");
    let tag = &html[start..];
    let end = tag.find('>').expect("body close");
    &tag[..=end]
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

#[test]
fn dropdown_js_closes_on_escape_and_restores_focus() {
    assert!(
        HEADER_CHROME_JS.contains("Escape") && HEADER_CHROME_JS.contains(".focus("),
        "{HEADER_CHROME_JS}"
    );
}

#[test]
fn header_nav_css_scrolls_on_small_viewports() {
    let html = render(
        Some(theme_nav(vec![nav_item("Guide", Some("/guide/"))])),
        false,
        PageChromeFlags::default(),
    );
    assert!(html.contains("overflow-x: auto") && html.contains("flex-wrap: nowrap"), "{html}");
}
