use super::super::{
    A11y, HeadValidation, NavGroup, NavItem, PageData, ReaderChrome, SsgConfig, generate_bare_html,
    generate_html,
};
use super::A11Y_CSS;

fn page(content: &str) -> PageData {
    PageData {
        title: "Current".to_string(),
        description: None,
        content: content.to_string(),
        toc: vec![],
        last_updated: None,
        contributors: vec![],
        path: "guide".to_string(),
        entry_page: None,
        prev: None,
        next: None,
        breadcrumbs: None,
        chrome: crate::PageChromeFlags::default(),
        robots: None,
        canonical: None,
        markdown_source: None,
    }
}

fn config(a11y: A11y) -> SsgConfig {
    SsgConfig {
        site_name: "Docs".to_string(),
        base: "/docs/".to_string(),
        breadcrumb_root_href: None,
        og_image: None,
        theme: None,
        locale: None,
        available_locales: None,
        pagination: false,
        breadcrumbs: false,
        reader_chrome: ReaderChrome::default(),
        locale_switcher: false,
        locale_paths: vec![],
        a11y,
        page_chrome: false,
        json_ld: crate::JsonLd::default(),
        site_url: None,
        head_validation: HeadValidation::Off,
    }
}

fn nav() -> Vec<NavGroup> {
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

fn render(a11y: A11y) -> String {
    generate_html(&page("<h1>Hello</h1><p>Article body</p>"), &nav(), &config(a11y))
}

fn body_after_open(html: &str) -> &str {
    let start = html.find("<body").expect("generated page must have a body");
    let end = html[start..].find('>').expect("body tag must close");
    &html[start + end + 1..]
}

fn first_focusable(html: &str) -> &str {
    let body = body_after_open(html);
    let mut best: Option<(usize, &str)> = None;
    for tag in ["<a ", "<a>", "<button", "<input", "<select", "<textarea"] {
        let Some(at) = body.find(tag) else {
            continue;
        };
        if best.is_some_and(|(best_at, _)| at >= best_at) {
            continue;
        }
        let rest = &body[at..];
        let end = rest.find('>').map_or(rest.len(), |i| i + 1);
        best = Some((at, &rest[..end]));
    }
    best.map(|(_, tag)| tag).expect("generated page must have a focusable control")
}

#[test]
fn disabled_by_default() {
    let html = render(A11y::disabled());
    let defaulted =
        generate_html(&page("<h1>Hello</h1><p>Article body</p>"), &nav(), &config(A11y::default()));

    assert_eq!(html, defaulted);
    assert!(!html.contains("ox-skip-link"), "{html}");
    assert!(!html.contains("id=\"ox-main\""), "{html}");
    assert!(!html.contains("ox-content:css:a11y"), "{html}");
    assert!(!html.contains("@media print"), "{html}");
    assert!(!html.contains("Skip to content"), "{html}");
}

#[test]
fn generate_bare_html_is_unchanged() {
    let html = generate_bare_html("<h1>Hello</h1>", "Test Page");

    assert_eq!(
        html,
        "<!DOCTYPE html>\n<html lang=\"en\">\n<head>\n  <meta charset=\"UTF-8\">\n  <meta name=\"viewport\" content=\"width=device-width, initial-scale=1.0\">\n  <title>Test Page</title>\n</head>\n<body>\n<h1>Hello</h1>\n</body>\n</html>"
    );
    assert!(!html.contains("ox-skip-link"), "{html}");
    assert!(!html.contains("@media print"), "{html}");
}

#[test]
fn happy_path_skip_link_first_and_targets_main() {
    let html = render(A11y::enabled());
    let first = first_focusable(&html);

    assert!(
        html.contains("<a class=\"ox-skip-link\" href=\"#ox-main\">Skip to content</a>"),
        "{html}"
    );
    assert!(first.contains("ox-skip-link"), "{first}");
    assert!(first.contains("href=\"#ox-main\""), "{first}");
    assert!(html.contains("<main class=\"main\" id=\"ox-main\">"), "{html}");
    assert!(html.contains("<h1>Hello</h1><p>Article body</p>"), "{html}");
    assert!(html.contains("ox-content:css:a11y"), "{html}");
    assert!(A11Y_CSS.contains(":focus"), "{A11Y_CSS}");
}

#[test]
fn hostile_label_escaped() {
    let html = render(A11y {
        skip_link_label: Some(r#"<img src=x onerror=alert(1)>" onclick="alert(1)"#.to_string()),
    });

    let skip = html
        .split(r#"class="ox-skip-link""#)
        .nth(1)
        .and_then(|rest| rest.split("</a>").next())
        .expect("skip link should be present");

    assert!(
        skip.contains("&lt;img src=x onerror=alert(1)&gt;")
            || skip.contains("&#60;img src=x onerror=alert(1)&#62;"),
        "{skip}"
    );
    assert!(skip.contains("&quot;") || skip.contains("&#34;"), "{skip}");
    assert!(!skip.contains("<img src=x onerror=alert(1)>"), "{skip}");
    assert!(!skip.contains(r#"onclick="alert(1)""#), "{skip}");
    assert!(!skip.contains("<script"), "{skip}");
}

#[test]
fn print_css_hides_chrome() {
    let html = render(A11y::enabled());

    assert!(html.contains("@media print"), "{html}");
    assert!(A11Y_CSS.contains("@media print"), "{A11Y_CSS}");
    assert!(A11Y_CSS.contains(".header"), "{A11Y_CSS}");
    assert!(A11Y_CSS.contains(".sidebar"), "{A11Y_CSS}");
    assert!(A11Y_CSS.contains(".search-modal"), "{A11Y_CSS}");
    assert!(A11Y_CSS.contains(".ox-copy"), "{A11Y_CSS}");
    assert!(A11Y_CSS.contains(".ox-back-to-top"), "{A11Y_CSS}");
    assert!(A11Y_CSS.contains(".ox-external-icon"), "{A11Y_CSS}");
    assert!(html.contains("<h1>Hello</h1><p>Article body</p>"), "{html}");
    assert!(html.contains("<article class=\"content\">"), "{html}");
}
