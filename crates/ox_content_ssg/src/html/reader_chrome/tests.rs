use super::super::{
    NavGroup, NavItem, PageData, ReaderChrome, SsgConfig, generate_bare_html, generate_html,
};
use super::{READER_CHROME_CSS, READER_CHROME_JS, apply_reader_chrome};

fn page(content: &str) -> PageData {
    PageData {
        title: "Current".to_string(),
        description: None,
        content: content.to_string(),
        toc: vec![],
        last_updated: None,
        path: "guide".to_string(),
        entry_page: None,
        prev: None,
        next: None,
        breadcrumbs: None,
    }
}

fn config(reader_chrome: ReaderChrome) -> SsgConfig {
    SsgConfig {
        site_name: "Docs".to_string(),
        base: "/docs/".to_string(),
        og_image: None,
        theme: None,
        locale: None,
        available_locales: None,
        pagination: false,
        breadcrumbs: false,
        reader_chrome,
        locale_switcher: false,
        locale_paths: vec![],
        a11y: crate::A11y::default(),
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

fn render(content: &str, chrome: ReaderChrome) -> String {
    generate_html(&page(content), &nav(), &config(chrome))
}

fn html_open_tag(html: &str) -> &str {
    let start = html.find("<html").expect("generated page must have an html tag");
    let end = html[start..].find('>').expect("html tag must close");
    &html[start..=start + end]
}

const ARTICLE: &str = concat!(
    "<p>See <a href=\"https://example.com/docs\">the docs</a> and ",
    "<a href=\"/local/page\">local</a>.</p>\n",
    "<pre><code>curl https://example.com/api\n</code></pre>\n",
    "<p>Inline <code><a href=\"https://example.com/secret\">secret</a></code>.</p>",
);

#[test]
fn reader_chrome_is_disabled_by_default() {
    let html = render(ARTICLE, ReaderChrome::disabled());

    assert!(!html.contains("data-ox-reader-chrome"), "{html}");
    assert!(!html.contains("ox-copy"), "{html}");
    assert!(!html.contains("ox-external"), "{html}");
    assert!(!html.contains("ox-back-to-top"), "{html}");
    assert!(!html.contains("data-ox-copy"), "{html}");
    assert!(!html.contains("reader-chrome"), "{html}");
    assert!(html.contains("<pre><code>curl https://example.com/api\n</code></pre>"), "{html}");
    assert!(html.contains(r#"<a href="https://example.com/docs">the docs</a>"#), "{html}");
}

#[test]
fn generate_html_is_unchanged_when_reader_chrome_is_off() {
    let off = render("<h1>Hello</h1>", ReaderChrome::disabled());
    let defaulted =
        generate_html(&page("<h1>Hello</h1>"), &nav(), &config(ReaderChrome::default()));

    assert_eq!(off, defaulted);
    assert!(!off.contains("data-ox-reader-chrome"));
    assert!(!off.contains("ox-copy"));
    assert!(!off.contains("ox-back-to-top"));
}

#[test]
fn generate_bare_html_is_unchanged() {
    let html = generate_bare_html("<h1>Hello</h1>", "Test Page");

    assert_eq!(
        html,
        "<!DOCTYPE html>\n<html lang=\"en\">\n<head>\n  <meta charset=\"UTF-8\">\n  <meta name=\"viewport\" content=\"width=device-width, initial-scale=1.0\">\n  <title>Test Page</title>\n</head>\n<body>\n<h1>Hello</h1>\n</body>\n</html>"
    );
    assert!(!html.contains("ox-copy"));
    assert!(!html.contains("ox-back-to-top"));
    assert!(!html.contains("reader-chrome"));
}

#[test]
fn enabled_defaults_emit_copy_external_and_back_to_top() {
    let html = render(ARTICLE, ReaderChrome::enabled());

    assert!(html.contains(r"data-ox-reader-chrome"), "{html}");
    assert!(html.contains(r"data-ox-copy"), "{html}");
    assert!(html.contains(r"data-ox-external-links"), "{html}");
    assert!(html.contains(r"data-ox-back-to-top"), "{html}");
    assert!(html.contains(r#"<div class="ox-code">"#), "{html}");
    assert!(
        html.contains(
            r#"<button type="button" class="ox-copy" data-ox-copy aria-label="Copy code">Copy</button>"#
        ),
        "{html}"
    );
    assert!(html.contains("<pre><code>curl https://example.com/api\n</code></pre>"), "{html}");
    assert!(
        !html.contains("data-ox-copy-text"),
        "copy must not snapshot fence text at build time: {html}"
    );
    assert!(html.contains(r#"class="ox-external""#), "{html}");
    assert!(html.contains(r#"rel="noopener noreferrer""#), "{html}");
    assert!(html.contains(r#"href="https://example.com/docs""#), "{html}");
    assert!(html.contains(r#"class="ox-external-icon""#), "{html}");
    assert!(html.contains(r#"<a href="/local/page">local</a>"#), "{html}");
    assert!(
        html.contains(r#"<button type="button" class="ox-back-to-top" data-ox-back-to-top hidden aria-label="Back to top">Back to top</button>"#),
        "{html}"
    );
    assert!(html.contains("ox-content:css:reader-chrome"), "{html}");
    assert!(html.contains("data-ox-reader-chrome"), "{html}");
}

#[test]
fn object_can_disable_copy() {
    let html =
        render(ARTICLE, ReaderChrome { copy: false, external_links: true, back_to_top: true });

    assert!(!html.contains(r#"class="ox-code""#), "{html}");
    assert!(!html.contains(r#"class="ox-copy""#), "{html}");
    let open = html_open_tag(&html);
    assert!(!open.contains("data-ox-copy"), "{open}");
    assert!(html.contains("<pre><code>curl https://example.com/api\n</code></pre>"), "{html}");
    assert!(html.contains(r#"class="ox-external""#), "{html}");
    assert!(html.contains("ox-back-to-top"), "{html}");
}

#[test]
fn fences_and_code_spans_are_not_external_links() {
    let html = render(ARTICLE, ReaderChrome::enabled());
    let secret = html.split("Inline").nth(1).expect("inline code sentence should remain");

    assert!(secret.contains(r#"<a href="https://example.com/secret">secret</a>"#), "{secret}");
    assert!(!secret.contains("ox-external"), "{secret}");
    assert!(html.contains("curl https://example.com/api"), "{html}");
    assert!(
        !html.contains(r#"<a href="https://example.com/api""#),
        "fence text must not become a link: {html}"
    );
}

#[test]
fn hostile_href_and_title_are_escaped() {
    let html = render(
        r#"<p><a href="https://example.com/ok" title="<img src=x onerror=alert(1)>" onclick="alert(1)">ok</a></p>"#,
        ReaderChrome::enabled(),
    );

    assert!(html.contains("https://example.com/ok"), "{html}");
    assert!(
        html.contains("title=\"&lt;img src=x onerror=alert(1)&gt;\"")
            || html.contains("title=\"&#60;img src=x onerror=alert(1)&#62;\""),
        "{html}"
    );
    assert!(!html.contains("onclick="), "{html}");
    assert!(!html.contains("<img src=x onerror=alert(1)>"), "{html}");
}

#[test]
fn dangerous_schemes_do_not_get_a_live_action() {
    let html = render(
        concat!(
            r#"<p><a href="javascript:alert(1)">js</a> "#,
            r#"<a href="data:text/html,hi">data</a> "#,
            r#"<a href="vbscript:msgbox(1)">vb</a></p>"#,
        ),
        ReaderChrome::enabled(),
    );

    assert!(!html.contains("javascript:"), "{html}");
    assert!(!html.contains("data:text/html"), "{html}");
    assert!(!html.contains("vbscript:"), "{html}");
    assert!(!html.contains("href=\"javascript"), "{html}");
    assert!(html.contains(r#"class="ox-external-inert""#), "{html}");
    assert!(html.contains(">js</span>"), "{html}");
    assert!(html.contains(">data</span>"), "{html}");
    assert!(html.contains(">vb</span>"), "{html}");
}

#[test]
fn reduced_motion_class_and_css_are_present() {
    let html = render(ARTICLE, ReaderChrome::enabled());

    assert!(READER_CHROME_CSS.contains("prefers-reduced-motion"), "{READER_CHROME_CSS}");
    assert!(READER_CHROME_CSS.contains("ox-reader-chrome--reduced-motion"), "{READER_CHROME_CSS}");
    assert!(html.contains("prefers-reduced-motion"), "{html}");
    assert!(html.contains("ox-reader-chrome--reduced-motion"), "{html}");
    assert!(READER_CHROME_JS.contains("ox-reader-chrome--reduced-motion"), "{READER_CHROME_JS}");
    assert!(READER_CHROME_JS.contains("prefers-reduced-motion"), "{READER_CHROME_JS}");
}

#[test]
fn unclosed_or_hostile_input_is_left_intact() {
    let unclosed_pre = "<pre><code>no end";
    let unclosed_link = r#"<a href="https://example.com/docs">no end"#;

    assert_eq!(apply_reader_chrome(unclosed_pre, ReaderChrome::enabled()), unclosed_pre);
    assert_eq!(apply_reader_chrome(unclosed_link, ReaderChrome::enabled()), unclosed_link);

    let html = render(unclosed_pre, ReaderChrome::enabled());
    assert!(!html.contains(r#"class="ox-code""#), "{html}");
}

#[test]
fn relative_and_hash_links_are_skipped() {
    let html = render(
        concat!(
            r#"<p><a href="./guide">rel</a> "#,
            r##"<a href="#top">hash</a> "##,
            r#"<a href="mailto:docs@example.com">mail</a></p>"#,
        ),
        ReaderChrome::enabled(),
    );

    assert!(html.contains(r#"<a href="./guide">rel</a>"#), "{html}");
    assert!(html.contains(r##"<a href="#top">hash</a>"##), "{html}");
    assert!(html.contains(r#"<a href="mailto:docs@example.com">mail</a>"#), "{html}");
    assert!(!html.contains(r#"class="ox-external""#), "{html}");
}

#[test]
fn encoded_javascript_href_is_not_a_live_action() {
    let html =
        apply_reader_chrome(r#"<a href="javascript&#58;alert(1)">js</a>"#, ReaderChrome::enabled());

    assert!(!html.contains("javascript:"), "{html}");
    assert!(!html.contains("javascript&#58;"), "{html}");
    assert!(html.contains(r#"class="ox-external-inert""#), "{html}");
}
