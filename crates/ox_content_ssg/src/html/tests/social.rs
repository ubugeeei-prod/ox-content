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
        markdown_source: None,
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
fn static_tweet_pages_ship_rich_card_css() {
    let html = generate_html(
        &page(r#"<article class="ox-tweet ox-tweet--rich"></article>"#),
        &[],
        &config(),
    );
    assert!(html.contains("ox-content:css:plugin-social:start"));
    assert!(html.contains(".ox-tweet--rich .ox-tweet__card"));
    assert!(html.contains(".ox-tweet--rich .ox-tweet__avatar-fallback"));
    assert!(!html.contains("ox-content:css:plugin-social-tweet-full:start"));
}

#[test]
fn reddit_pages_ship_social_css() {
    let html =
        generate_html(&page(r#"<article class="ox-reddit-card"></article>"#), &[], &config());
    assert!(html.contains("ox-content:css:plugin-social:start"));
    assert!(html.contains(".ox-reddit-card__subreddit"));
    assert!(!html.contains("ox-content:css:plugin-social-tweet-full:start"));
}

#[test]
fn full_tweet_pages_ship_gated_full_card_css() {
    let html = generate_html(
        &page(
            r#"<figure class="ox-tweet ox-tweet--fetched ox-tweet--full"><a data-ox-tweet-copy></a></figure>"#,
        ),
        &[],
        &config(),
    );
    assert!(html.contains("ox-content:css:plugin-social:start"));
    assert!(html.contains("ox-content:css:plugin-social-tweet-full:start"));
    assert!(html.contains("--ox-tweet-color-blue"));
    assert!(html.contains("ox-content:js:plugin-social-tweet-full:start"));
    assert!(html.contains("initTweetCards(document);"));
}

#[test]
fn native_media_pages_ship_social_css() {
    for class_name in ["ox-audio", "ox-video"] {
        let html = generate_html(
            &page(&format!(r#"<figure class="{class_name}"></figure>"#)),
            &[],
            &config(),
        );
        assert!(html.contains("ox-content:css:plugin-social:start"), "{class_name}");
        assert!(html.contains(&format!(".{class_name}")), "{class_name}");
        assert!(!html.contains("ox-content:css:plugin-social-tweet-full:start"), "{class_name}");
    }
}

#[test]
fn full_tweet_css_keeps_rich_copy_and_replies_affordances() {
    assert!(SOCIAL_TWEET_FULL_CSS.contains("--ox-tweet-icon-copy"));
    assert!(SOCIAL_TWEET_FULL_CSS.contains(".ox-tweet__icon--copy"));
    assert!(SOCIAL_TWEET_FULL_CSS.contains(".ox-tweet--full .ox-tweet__action--copy:hover"));
    assert!(SOCIAL_TWEET_FULL_CSS.contains("[data-ox-tweet-copied]"));
    assert!(SOCIAL_TWEET_FULL_CSS.contains(".ox-tweet__copied-text"));
    assert!(
        SOCIAL_TWEET_FULL_CSS.contains(
            ".ox-tweet--full .ox-tweet__replies-link {\n  box-sizing: border-box;\n  display: flex;"
        ),
        "{SOCIAL_TWEET_FULL_CSS}"
    );
    assert!(
        SOCIAL_TWEET_FULL_CSS.contains("\n  width: 100%;\n  min-height: 32px;"),
        "{SOCIAL_TWEET_FULL_CSS}"
    );
    assert!(SOCIAL_TWEET_FULL_CSS.contains(".ox-tweet--full .ox-tweet__replies-link:hover"));
    assert!(!SOCIAL_TWEET_FULL_CSS.contains("text-align: center"), "{SOCIAL_TWEET_FULL_CSS}");
}

#[test]
fn fetched_tweet_cards_sit_level_with_bluesky_cards() {
    let html = generate_html(
        &page(r#"<figure class="ox-tweet ox-tweet--fetched"></figure>"#),
        &[],
        &config(),
    );

    // A fetched tweet is `<figure><header>…`, so the shared `.ox-tweet > a`
    // padding never applies and the card would start flush at its top border.
    assert!(html.contains(".ox-tweet--fetched {\n  padding: 1rem 0;\n}"), "{html}");
    assert!(html.contains(".ox-tweet > a,\n.ox-bluesky > a {"), "{html}");
}
