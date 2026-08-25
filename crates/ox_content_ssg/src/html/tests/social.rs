use super::super::*;

fn page(content: &str) -> PageData {
    PageData {
        title: "Social".to_string(),
        description: None,
        content: content.to_string(),
        toc: vec![],
        last_updated: None,
        contributors: vec![],
        path: "social".to_string(),
        entry_page: None,
        prev: None,
        next: None,
        breadcrumbs: None,
        chrome: PageChromeFlags::default(),
        robots: None,
        canonical: None,
    }
}

fn config() -> SsgConfig {
    SsgConfig {
        site_name: "Test".to_string(),
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
        a11y: A11y::default(),
        page_chrome: false,
        json_ld: JsonLd::default(),
        site_url: None,
        head_validation: HeadValidation::Off,
    }
}

#[test]
fn compact_tweet_pages_do_not_ship_full_card_css() {
    let html = generate_html(
        &page(r#"<figure class="ox-tweet ox-tweet--fetched"></figure>"#),
        &[],
        &config(),
    );
    assert!(html.contains("ox-content:css:plugin-social:start"));
    assert!(!html.contains("ox-content:css:plugin-social-tweet-full:start"));
    assert!(!html.contains("--ox-tweet-color-blue"));
}

#[test]
fn full_tweet_pages_ship_gated_full_card_css() {
    let html = generate_html(
        &page(r#"<figure class="ox-tweet ox-tweet--fetched ox-tweet--full"></figure>"#),
        &[],
        &config(),
    );
    assert!(html.contains("ox-content:css:plugin-social:start"));
    assert!(html.contains("ox-content:css:plugin-social-tweet-full:start"));
    assert!(html.contains("--ox-tweet-color-blue"));
}
