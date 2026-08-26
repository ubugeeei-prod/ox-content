use super::{SectionIndexItem, SectionIndexStyle, is_safe_section_href, render_section_index};
use crate::html::{
    HeadValidation, NavGroup, NavItem, PageData, ReaderChrome, SsgConfig, generate_html,
    section_index::SECTION_INDEX_CSS,
};

fn item(title: &str, href: &str, description: Option<&str>) -> SectionIndexItem {
    SectionIndexItem {
        title: title.to_string(),
        href: href.to_string(),
        description: description.map(str::to_string),
    }
}

fn page(content: &str) -> PageData {
    PageData {
        title: "Guide".to_string(),
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

fn config() -> SsgConfig {
    SsgConfig {
        site_name: "Docs".to_string(),
        base: "/".to_string(),
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
        a11y: crate::A11y::default(),
        page_chrome: false,
        json_ld: crate::JsonLd::default(),
        site_url: None,
        head_validation: HeadValidation::Off,
    }
}

fn nav() -> Vec<NavGroup> {
    vec![NavGroup {
        title: "Site".to_string(),
        items: vec![NavItem {
            title: "Guide".to_string(),
            path: "guide".to_string(),
            href: "/guide/index.html".to_string(),
            children: vec![],
            collapsed: None,
            sticky_collapsed: None,
        }],
        collapsed: None,
        sticky_collapsed: None,
    }]
}

#[test]
fn happy_path_cards() {
    let html = render_section_index(
        "Guide",
        &[
            item("Page A", "/guide/a/index.html", Some("First")),
            item("Page B", "/guide/b/index.html", None),
        ],
        SectionIndexStyle::Cards,
    );

    assert!(html.contains(r#"class="ox-section-index ox-section-index--cards""#), "{html}");
    assert!(html.contains("Page A"), "{html}");
    assert!(html.contains("Page B"), "{html}");
    assert!(html.contains(r#"href="/guide/a/index.html""#), "{html}");
    assert!(html.contains(r#"href="/guide/b/index.html""#), "{html}");
    assert!(html.contains("First"), "{html}");
    assert!(html.contains("ox-section-index__card"), "{html}");

    let page_html = generate_html(&page(&html), &nav(), &config());
    assert!(page_html.contains("ox-content:css:section-index"), "{page_html}");
    assert!(page_html.contains(SECTION_INDEX_CSS.trim()), "{page_html}");
}

#[test]
fn happy_path_list() {
    let html = render_section_index(
        "Guide",
        &[item("Page A", "/guide/a/index.html", None)],
        SectionIndexStyle::List,
    );

    assert!(html.contains("ox-section-index--list"), "{html}");
    assert!(html.contains("ox-section-index__list"), "{html}");
    assert!(!html.contains("ox-section-index__card"), "{html}");
    assert!(html.contains("Page A"), "{html}");
}

#[test]
fn javascript_href_rejected() {
    let html = render_section_index(
        "Guide",
        &[
            item("Safe", "/guide/a/index.html", None),
            item("XSS", "javascript:alert(1)", None),
            item("Data", "data:text/html,hi", None),
            item("Proto", "//evil.example/x", None),
        ],
        SectionIndexStyle::List,
    );

    assert!(html.contains("Safe"), "{html}");
    assert!(!html.contains("javascript:"), "{html}");
    assert!(!html.contains("alert(1)"), "{html}");
    assert!(!html.contains("XSS"), "{html}");
    assert!(!html.contains("data:"), "{html}");
    assert!(!html.contains("//evil.example"), "{html}");
    assert!(!is_safe_section_href("javascript:alert(1)"));
    assert!(!is_safe_section_href("vbscript:alert(1)"));
    assert!(is_safe_section_href("/guide/a/index.html"));
}

#[test]
fn hostile_title_escaped() {
    let html = render_section_index(
        r"</title><script>alert(1)</script>",
        &[item(r"<img src=x onerror=alert(1)>", "/guide/a/index.html", Some("<script>x</script>"))],
        SectionIndexStyle::Cards,
    );

    assert!(!html.contains("<script>alert(1)</script>"), "{html}");
    assert!(html.contains("&lt;script&gt;alert(1)&lt;/script&gt;"), "{html}");
    assert!(!html.contains("<img src=x"), "{html}");
    assert!(html.contains("&lt;img src=x onerror=alert(1)&gt;"), "{html}");
    assert!(html.contains("&lt;script&gt;x&lt;/script&gt;"), "{html}");
}

#[test]
fn hostile_href_is_attribute_escaped() {
    let html = render_section_index(
        "Guide",
        &[item("Local", r#"/guide/a/index.html" onclick="alert(1)"#, None)],
        SectionIndexStyle::List,
    );

    assert!(!html.contains(r#"onclick="alert(1)""#), "{html}");
    assert!(html.contains("/guide/a/index.html&quot; onclick=&quot;alert(1)"), "{html}");
}

#[test]
fn empty_items_still_emit_nav() {
    let html = render_section_index("Guide", &[], SectionIndexStyle::Cards);
    assert!(html.contains(r#"aria-label="Section pages""#), "{html}");
    assert!(html.contains("<h1>Guide</h1>"), "{html}");
    assert!(html.contains("<ul"), "{html}");
}
