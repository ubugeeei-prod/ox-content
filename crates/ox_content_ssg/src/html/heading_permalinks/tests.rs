use super::{
    HEADING_PERMALINK_CSS, heading_permalink_always, page_has_heading_permalinks,
    push_heading_permalink_body_class, push_heading_permalink_css,
};
use crate::html::theme::ThemeConfig;
use crate::html::{
    A11y, HeadValidation, PageChromeFlags, PageData, ReaderChrome, SsgConfig, generate_html,
};

fn page(content: &str) -> PageData {
    PageData {
        title: "Permalink".to_string(),
        description: None,
        content: content.to_string(),
        toc: vec![],
        last_updated: None,
        contributors: vec![],
        path: "permalink".to_string(),
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

fn config(theme: Option<ThemeConfig>) -> SsgConfig {
    SsgConfig {
        site_name: "Docs".to_string(),
        base: "/".to_string(),
        breadcrumb_root_href: None,
        og_image: None,
        site_url: None,
        head_validation: HeadValidation::Off,
        theme,
        locale: None,
        available_locales: None,
        pagination: false,
        breadcrumbs: false,
        reader_chrome: ReaderChrome::default(),
        locale_switcher: false,
        locale_paths: vec![],
        a11y: A11y::default(),
        page_chrome: false,
        json_ld: crate::JsonLd::default(),
    }
}

fn permalink_heading() -> &'static str {
    r##"<h2 id="hello">Hello<a class="header-anchor" href="#hello" aria-label="Permalink to &quot;Hello&quot;">#</a></h2>"##
}

#[test]
fn css_is_omitted_when_headings_have_no_permalink() {
    let html = generate_html(&page("<h1 id=\"hello\">Hello</h1>"), &[], &config(None));
    assert!(!html.contains("ox-content:css:heading-permalinks"), "{html}");
    assert!(!html.contains(".header-anchor"), "{html}");
    assert!(!html.contains(r#"class="ox-heading-permalinks--always""#), "{html}");
}

#[test]
fn css_is_included_when_renderer_emits_the_marker() {
    let html = generate_html(&page(permalink_heading()), &[], &config(None));
    assert!(html.contains("ox-content:css:heading-permalinks"), "{html}");
    assert!(html.contains(".header-anchor"), "{html}");
    assert!(!html.contains(r#"class="ox-heading-permalinks--always""#), "{html}");
}

#[test]
fn always_visible_is_a_body_class_only() {
    let theme = ThemeConfig { heading_permalink: Some("always".into()), ..Default::default() };
    let html = generate_html(&page(permalink_heading()), &[], &config(Some(theme)));
    assert!(html.contains(r#"class="ox-heading-permalinks--always""#), "{html}");
    assert!(html.contains(r##"<a class="header-anchor" href="#hello""##), "{html}");
}

#[test]
fn hover_theme_does_not_change_heading_html() {
    let theme = ThemeConfig { heading_permalink: Some("hover".into()), ..Default::default() };
    let html = generate_html(&page(permalink_heading()), &[], &config(Some(theme)));
    assert!(html.contains(permalink_heading()), "{html}");
    assert!(!html.contains(r#"class="ox-heading-permalinks--always""#), "{html}");
}

#[test]
fn visibility_helpers_and_css_contract() {
    assert!(!page_has_heading_permalinks("<h1 id=\"hello\">Hello</h1>"));
    assert!(page_has_heading_permalinks(permalink_heading()));
    assert!(!heading_permalink_always(None));
    assert!(!heading_permalink_always(Some(&ThemeConfig::default())));
    assert!(heading_permalink_always(Some(&ThemeConfig {
        heading_permalink: Some("always".into()),
        ..Default::default()
    })));

    let mut css = Vec::new();
    push_heading_permalink_css(&mut css, permalink_heading());
    assert_eq!(css.len(), 1);

    let mut classes = Vec::new();
    push_heading_permalink_body_class(&mut classes, None, permalink_heading());
    assert!(classes.is_empty());

    assert!(HEADING_PERMALINK_CSS.contains("margin-inline-start"));
    assert!(HEADING_PERMALINK_CSS.contains("(hover: hover) and (pointer: fine)"));
    assert!(HEADING_PERMALINK_CSS.contains(":focus-visible"));
    assert!(HEADING_PERMALINK_CSS.contains("prefers-reduced-motion"));
    assert!(!HEADING_PERMALINK_CSS.contains("box-shadow"));
    assert!(!HEADING_PERMALINK_CSS.contains("<script"));
}
