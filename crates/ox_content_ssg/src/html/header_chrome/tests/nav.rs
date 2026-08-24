use super::super::{HEADER_CHROME_JS, PageChromeFlags};
use super::{dropdown, nav_item, render, theme_nav};

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
fn dropdown_js_closes_on_escape_and_restores_focus() {
    assert!(
        HEADER_CHROME_JS.contains("Escape")
            && HEADER_CHROME_JS.contains(".focus(")
            && HEADER_CHROME_JS.contains(".ox-locale-switcher > button"),
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
