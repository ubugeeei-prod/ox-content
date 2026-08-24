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
