use super::super::*;

fn heading(depth: u8, text: &str, slug: &str) -> TocEntry {
    TocEntry { depth, text: text.to_string(), slug: slug.to_string() }
}

fn page(toc: Vec<TocEntry>) -> PageData {
    PageData {
        title: "Guide".to_string(),
        description: None,
        content: "<h1>Hello</h1>".to_string(),
        toc,
        last_updated: None,
        path: "guide".to_string(),
        entry_page: None,
        prev: None,
        next: None,
        breadcrumbs: None,
    }
}

fn config(theme: Option<ThemeConfig>) -> SsgConfig {
    SsgConfig {
        site_name: "Docs".to_string(),
        base: "/".to_string(),
        og_image: None,
        theme,
        locale: None,
        available_locales: None,
        pagination: false,
        breadcrumbs: false,
        reader_chrome: ReaderChrome::default(),
        locale_switcher: false,
        locale_paths: vec![],
    }
}

fn render(toc: Vec<TocEntry>, theme: Option<ThemeConfig>) -> String {
    generate_html(&page(toc), &[], &config(theme))
}

fn hello_toc() -> Vec<TocEntry> {
    vec![heading(1, "Hello", "hello")]
}

fn theme_with_aside(aside: bool) -> ThemeConfig {
    ThemeConfig { aside: Some(aside), ..Default::default() }
}

fn assert_outline_absent(html: &str) {
    assert!(!html.contains(r#"<aside class="toc""#), "outline aside must be absent:\n{html}");
    assert!(
        !html.contains(r#"class="main main--with-toc""#),
        "main--with-toc must be absent:\n{html}"
    );
    assert!(!html.contains("On this page"), "On this page must be absent:\n{html}");
}

#[test]
fn aside_enabled_reads_theme_flag() {
    use super::super::aside::{aside_enabled, has_toc};

    assert!(!aside_enabled(None));
    assert!(!aside_enabled(Some(&ThemeConfig::default())));
    assert!(!aside_enabled(Some(&theme_with_aside(false))));
    assert!(aside_enabled(Some(&theme_with_aside(true))));
    assert!(!has_toc(true, ""));
    assert!(has_toc(true, r##"<li><a href="#hello">Hello</a></li>"##));
    assert!(!has_toc(false, r##"<li><a href="#hello">Hello</a></li>"##));
}

#[test]
fn aside_is_disabled_by_default() {
    let html = render(hello_toc(), None);
    assert_outline_absent(&html);
    assert!(html.contains(r#"<main class="main">"#), "{html}");
}

#[test]
fn aside_omitted_on_theme_hides_outline() {
    let html = render(hello_toc(), Some(ThemeConfig::default()));
    assert_outline_absent(&html);
}

#[test]
fn aside_true_with_toc_emits_outline() {
    let html = render(
        vec![heading(1, "Hello", "hello"), heading(2, "Setup", "setup")],
        Some(theme_with_aside(true)),
    );

    assert!(html.contains(r#"<aside class="toc" aria-label="On this page">"#), "{html}");
    assert!(html.contains(r#"<div class="toc-title">On this page</div>"#), "{html}");
    assert!(html.contains(r#"<ul class="toc-list">"#), "{html}");
    assert!(html.contains(r#"<main class="main main--with-toc">"#), "{html}");
    assert!(
        html.contains(r##"<a href="#hello" class="toc-link toc-link--depth-1">Hello</a>"##),
        "{html}"
    );
    assert!(
        html.contains(r##"<a href="#setup" class="toc-link toc-link--depth-2">Setup</a>"##),
        "{html}"
    );
}

#[test]
fn aside_true_with_empty_toc_emits_nothing() {
    let html = render(vec![], Some(theme_with_aside(true)));
    assert_outline_absent(&html);
    assert!(html.contains(r#"<main class="main">"#), "{html}");
}

#[test]
fn aside_false_explicitly_hides_outline() {
    let html = render(hello_toc(), Some(theme_with_aside(false)));
    assert_outline_absent(&html);
}

#[test]
fn aside_true_escapes_heading_text_and_slug() {
    let html = render(
        vec![heading(2, "A <script>", "a\" onclick=\"alert(1)")],
        Some(theme_with_aside(true)),
    );

    assert!(
        html.contains(
            r##"<a href="#a&quot; onclick=&quot;alert(1)" class="toc-link toc-link--depth-2">A &lt;script&gt;</a>"##
        ),
        "{html}"
    );
    assert!(!html.contains("A <script>"), "{html}");
    assert!(!html.contains(r##"href="#a" onclick="alert(1)""##), "{html}");
}

#[test]
fn generate_bare_html_unchanged() {
    let html = generate_bare_html("<h1>Hello</h1>", "Test Page");
    assert_eq!(
        html,
        "<!DOCTYPE html>\n<html lang=\"en\">\n<head>\n  <meta charset=\"UTF-8\">\n  <meta name=\"viewport\" content=\"width=device-width, initial-scale=1.0\">\n  <title>Test Page</title>\n</head>\n<body>\n<h1>Hello</h1>\n</body>\n</html>"
    );
    assert_outline_absent(&html);
}
