use super::super::nav::generate_nav_html;
use super::super::utils::{format_last_updated, generate_toc_html, html_locale_attrs};
use super::super::*;

#[test]
fn test_generate_html() {
    let page_data = PageData {
        title: "Test Page".to_string(),
        description: Some("Test description".to_string()),
        content: "<h1>Hello</h1>".to_string(),
        toc: vec![TocEntry { depth: 1, text: "Hello".to_string(), slug: "hello".to_string() }],
        last_updated: Some(0),
        path: "test".to_string(),
        entry_page: None,
        prev: None,
        next: None,
        breadcrumbs: None,
        chrome: PageChromeFlags::default(),
    };

    let nav_groups = vec![NavGroup {
        title: "Guide".to_string(),
        items: vec![NavItem {
            title: "Test Page".to_string(),
            path: "test".to_string(),
            href: "/docs/test/index.html".to_string(),
            children: vec![],
            collapsed: None,
            sticky_collapsed: None,
        }],
        collapsed: None,
        sticky_collapsed: None,
    }];

    let config = SsgConfig {
        site_name: "Test Site".to_string(),
        base: "/docs/".to_string(),
        og_image: None,
        theme: None,
        locale: None,
        available_locales: None,
        pagination: false,
        breadcrumbs: false,
        reader_chrome: ReaderChrome::default(),
        locale_switcher: false,
        locale_paths: vec![],
        a11y: A11y::default(),
        page_chrome: false,
    };

    let html = generate_html(&page_data, &nav_groups, &config);

    insta::assert_snapshot!(super::snapshot_text(&html));
}

#[test]
fn test_generate_bare_html() {
    let html = generate_bare_html("<h1>Hello</h1>", "Test Page");

    assert_eq!(
        html,
        "<!DOCTYPE html>\n<html lang=\"en\">\n<head>\n  <meta charset=\"UTF-8\">\n  <meta name=\"viewport\" content=\"width=device-width, initial-scale=1.0\">\n  <title>Test Page</title>\n</head>\n<body>\n<h1>Hello</h1>\n</body>\n</html>"
    );
}

#[test]
fn test_generate_bare_page_without_metadata_matches_the_old_bare_output() {
    // Bare mode is also the no-JS size baseline in the benchmark tables, so a
    // page with nothing configured must not grow a single tag.
    let data =
        BarePageData { title: "Test Page", content: "<h1>Hello</h1>", ..BarePageData::default() };

    assert_eq!(generate_bare_page(&data), generate_bare_html("<h1>Hello</h1>", "Test Page"));
}

#[test]
fn test_generate_bare_page_emits_head_metadata() {
    let data = BarePageData {
        title: "Guide",
        content: "<p>body</p>",
        lang: "ja",
        dir: "ltr",
        description: Some("How it works"),
        canonical_url: Some("https://example.com/guide/"),
        site_name: Some("Docs"),
        og_image: Some("https://example.com/guide/og-image.png"),
        ..BarePageData::default()
    };

    insta::assert_snapshot!(super::snapshot_text(&generate_bare_page(&data)));
}

#[test]
fn test_generate_bare_page_injects_consumer_markup() {
    let data = BarePageData {
        title: "Guide",
        content: "<p>body</p>",
        head: "<link rel=\"stylesheet\" href=\"/assets/site.css\">",
        body_start: "<header>site</header>",
        body_end: "<footer>end</footer>",
        ..BarePageData::default()
    };

    insta::assert_snapshot!(super::snapshot_text(&generate_bare_page(&data)));
}

#[test]
fn test_generate_bare_page_escapes_metadata_but_keeps_injected_markup_raw() {
    let data = BarePageData {
        title: "Guide",
        content: "<p>body</p>",
        description: Some("a \"quoted\" & <angled> value"),
        site_name: Some("Docs & More"),
        head: "<meta name=\"raw\" content=\"kept\">",
        ..BarePageData::default()
    };

    insta::assert_snapshot!(super::snapshot_text(&generate_bare_page(&data)));
}

#[test]
fn test_generate_bare_html_escapes_title_but_keeps_content_raw() {
    let html = generate_bare_html("<h1>Raw & ready</h1>", "<script>alert(1)</script>");
    insta::assert_snapshot!(super::snapshot_text(&html));
}

#[test]
fn test_generate_nav_html_with_nested_collapsed_items() {
    let nav_groups = vec![NavGroup {
        title: "Guide & API".to_string(),
        collapsed: Some(true),
        sticky_collapsed: Some(true),
        items: vec![NavItem {
            title: "Runtime <Core>".to_string(),
            path: "runtime".to_string(),
            href: "javascript:alert(1)".to_string(),
            collapsed: Some(false),
            sticky_collapsed: Some(true),
            children: vec![NavItem {
                title: "Setup".to_string(),
                path: "runtime/setup".to_string(),
                href: "/docs/runtime/setup/index.html".to_string(),
                children: vec![],
                collapsed: None,
                sticky_collapsed: None,
            }],
        }],
    }];

    let html = generate_nav_html(&nav_groups, "runtime/setup");
    insta::assert_snapshot!(super::snapshot_text(&html));
}

#[test]
fn test_nested_nav_css_keeps_disclosure_and_hierarchy_visible() {
    assert!(
        SSG_CSS.contains(".nav-details > .nav-summary"),
        "linked sidebar groups need an explicit disclosure row"
    );
    assert!(
        SSG_CSS.contains(".nav-summary::-webkit-details-marker"),
        "the browser marker must not occupy a separate layout row"
    );
    assert!(
        SSG_CSS.contains(".nav-list--nested {\n  margin-inline-start:"),
        "nested navigation must use RTL-safe visual indentation"
    );
    assert!(
        SSG_CSS.contains(".nav-list--nested {\n  margin-inline-start:")
            && SSG_CSS.contains("border-inline-start:"),
        "nested navigation needs both an indentation step and a hierarchy rail"
    );
}

#[test]
fn test_table_css_draws_each_separator_once() {
    assert!(
        SSG_CSS.contains(".content table {\n  width: 100%;")
            && SSG_CSS.contains("border: 1px solid var(--octc-color-border);")
            && SSG_CSS.contains("border-collapse: separate;"),
        "tables need one shared separated-border model at every breakpoint"
    );
    assert!(
        SSG_CSS.contains("border-inline-end: 1px solid var(--octc-color-border);")
            && SSG_CSS.contains("border-block-end: 1px solid var(--octc-color-border);"),
        "cells must paint only one logical edge per internal seam"
    );
    assert!(
        SSG_CSS.contains(".content tr > :last-child")
            && SSG_CSS.contains(".content table > :last-child > tr:last-child > *"),
        "the table border must own the outer inline and block edges"
    );
    assert_eq!(
        SSG_CSS.matches("border-collapse: separate;").count(),
        1,
        "mobile must not install a second border model"
    );
    assert!(
        !SSG_CSS.contains("border-collapse: collapse;"),
        "collapsed cell borders conflict with the scrollable mobile table"
    );
}

#[test]
fn test_mobile_content_css_preserves_safe_reading_gutters() {
    assert!(
        SSG_CSS.contains("--octc-mobile-gutter: clamp(1rem, 4vw, 1.25rem);"),
        "mobile layouts need a shared readable gutter token"
    );
    assert!(
        SSG_CSS.contains(
            "padding-left: max(var(--octc-mobile-gutter), env(safe-area-inset-left, 0px));"
        ) && SSG_CSS.contains(
            "padding-right: max(var(--octc-mobile-gutter), env(safe-area-inset-right, 0px));"
        ),
        "content gutters must include each physical display safe area"
    );
    assert!(
        SSG_CSS.contains(".content table {\n    display: block;")
            && SSG_CSS.contains("max-width: 100%;"),
        "wide tables must scroll inside the safe content gutter"
    );
    assert!(
        !SSG_CSS.contains("padding: 0.75rem 0.4rem;"),
        "the narrow breakpoint must not collapse back to a 6.4px gutter"
    );
}

#[test]
fn test_generate_html_without_toc_omits_outline() {
    let page_data = PageData {
        title: "No TOC".to_string(),
        description: None,
        content: "<p>Content</p>".to_string(),
        toc: vec![],
        last_updated: None,
        path: "no-toc".to_string(),
        entry_page: None,
        prev: None,
        next: None,
        breadcrumbs: None,
        chrome: PageChromeFlags::default(),
    };
    let config = SsgConfig {
        site_name: "Test Site".to_string(),
        base: "/".to_string(),
        og_image: None,
        theme: None,
        locale: None,
        available_locales: None,
        pagination: false,
        breadcrumbs: false,
        reader_chrome: ReaderChrome::default(),
        locale_switcher: false,
        locale_paths: vec![],
        a11y: A11y::default(),
        page_chrome: false,
    };

    let html = generate_html(&page_data, &[], &config);
    insta::assert_snapshot!(super::snapshot_text(&html));
}

#[test]
fn test_format_last_updated_rejects_invalid_timestamps() {
    assert!(format_last_updated(-1).is_none());
}

#[test]
fn test_html_locale_attrs_use_current_locale_and_direction() {
    let config = SsgConfig {
        site_name: "Localized".to_string(),
        base: "/".to_string(),
        og_image: None,
        theme: None,
        locale: Some("ar".to_string()),
        available_locales: None,
        pagination: false,
        breadcrumbs: false,
        reader_chrome: ReaderChrome::default(),
        locale_switcher: false,
        locale_paths: vec![],
        a11y: A11y::default(),
        page_chrome: false,
    };

    assert_eq!(html_locale_attrs(&config), ("ar", "rtl"));

    let page_data = PageData {
        title: "مرحبا".to_string(),
        description: None,
        content: "<p>Content</p>".to_string(),
        toc: vec![],
        last_updated: None,
        path: "ar".to_string(),
        entry_page: None,
        prev: None,
        next: None,
        breadcrumbs: None,
        chrome: PageChromeFlags::default(),
    };
    let html = generate_html(&page_data, &[], &config);
    insta::assert_snapshot!(super::snapshot_text(&html));
}

#[test]
fn test_generate_toc_html_escapes_entries() {
    let html = generate_toc_html(&[TocEntry {
        depth: 2,
        text: "A <script>".to_string(),
        slug: "a\" onclick=\"alert(1)".to_string(),
    }]);

    insta::assert_snapshot!(super::snapshot_text(&html));
}
