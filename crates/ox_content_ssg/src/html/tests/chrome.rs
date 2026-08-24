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
