use super::super::*;

#[test]
fn test_generate_html_omits_toc_aside_until_opted_in() {
    let page_data = PageData {
        title: "Test Page".to_string(),
        description: None,
        content: "<h1>Hello</h1>".to_string(),
        toc: vec![TocEntry { depth: 1, text: "Hello".to_string(), slug: "hello".to_string() }],
        last_updated: None,
        path: "test".to_string(),
        entry_page: None,
    };
    let config = SsgConfig {
        site_name: "Test Site".to_string(),
        base: "/".to_string(),
        og_image: None,
        theme: None,
        locale: None,
        available_locales: None,
    };

    let html = generate_html(&page_data, &[], &config);
    assert!(!html.contains("<aside class=\"toc\""), "{html}");
    assert!(!html.contains("<main class=\"main main--with-toc\">"), "{html}");

    let themed = generate_html(
        &page_data,
        &[],
        &SsgConfig { theme: Some(ThemeConfig::default()), ..config },
    );
    assert!(!themed.contains("<aside class=\"toc\""), "{themed}");
}

#[test]
fn test_generate_html_renders_toc_aside_when_enabled() {
    let page_data = PageData {
        title: "Test Page".to_string(),
        description: None,
        content: "<h1>Hello</h1>".to_string(),
        toc: vec![TocEntry { depth: 1, text: "Hello".to_string(), slug: "hello".to_string() }],
        last_updated: None,
        path: "test".to_string(),
        entry_page: None,
    };
    let config = SsgConfig {
        site_name: "Test Site".to_string(),
        base: "/".to_string(),
        og_image: None,
        theme: Some(ThemeConfig { aside: Some(true), ..Default::default() }),
        locale: None,
        available_locales: None,
    };

    let html = generate_html(&page_data, &[], &config);
    assert!(html.contains("<aside class=\"toc\""), "{html}");
    assert!(html.contains("<main class=\"main main--with-toc\">"), "{html}");
    assert!(html.contains("href=\"#hello\""), "{html}");
}

#[test]
fn test_generate_html_renders_prev_next_when_enabled() {
    let page_data = PageData {
        title: "Setup".to_string(),
        description: None,
        content: "<p>Body</p>".to_string(),
        toc: vec![],
        last_updated: None,
        path: "setup".to_string(),
        entry_page: None,
    };
    let nav_groups = vec![NavGroup {
        title: "Guide".to_string(),
        items: vec![
            NavItem {
                title: "Intro".to_string(),
                path: "intro".to_string(),
                href: "/intro.html".to_string(),
                children: vec![],
                collapsed: None,
                sticky_collapsed: None,
            },
            NavItem {
                title: "Setup".to_string(),
                path: "setup".to_string(),
                href: "/setup.html".to_string(),
                children: vec![],
                collapsed: None,
                sticky_collapsed: None,
            },
            NavItem {
                title: "API".to_string(),
                path: "api".to_string(),
                href: "/api.html".to_string(),
                children: vec![],
                collapsed: None,
                sticky_collapsed: None,
            },
        ],
        collapsed: None,
        sticky_collapsed: None,
    }];
    let config = SsgConfig {
        site_name: "Test Site".to_string(),
        base: "/".to_string(),
        og_image: None,
        theme: Some(ThemeConfig { prev_next: Some(true), ..Default::default() }),
        locale: None,
        available_locales: None,
    };

    let html = generate_html(&page_data, &nav_groups, &config);
    assert!(html.contains("class=\"pager\""), "{html}");
    assert!(html.contains("pager-link--prev"), "{html}");
    assert!(html.contains("pager-link--next"), "{html}");
    assert!(html.contains("href=\"/intro.html\""), "{html}");
    assert!(html.contains("href=\"/api.html\""), "{html}");
    assert!(html.contains("ox-content:css:pager:start"), "{html}");
}

#[test]
fn test_generate_html_skips_prev_next_on_entry_pages() {
    let page_data = PageData {
        title: "Home".to_string(),
        description: None,
        content: String::new(),
        toc: vec![],
        last_updated: None,
        path: "index".to_string(),
        entry_page: Some(EntryPageConfig::default()),
    };
    let nav_groups = vec![NavGroup {
        title: "Guide".to_string(),
        items: vec![
            NavItem {
                title: "Home".to_string(),
                path: "index".to_string(),
                href: "/index.html".to_string(),
                children: vec![],
                collapsed: None,
                sticky_collapsed: None,
            },
            NavItem {
                title: "Intro".to_string(),
                path: "intro".to_string(),
                href: "/intro.html".to_string(),
                children: vec![],
                collapsed: None,
                sticky_collapsed: None,
            },
        ],
        collapsed: None,
        sticky_collapsed: None,
    }];
    let config = SsgConfig {
        site_name: "Test Site".to_string(),
        base: "/".to_string(),
        og_image: None,
        theme: Some(ThemeConfig { prev_next: Some(true), ..Default::default() }),
        locale: None,
        available_locales: None,
    };

    let html = generate_html(&page_data, &nav_groups, &config);
    assert!(!html.contains("class=\"pager\""), "{html}");
}
