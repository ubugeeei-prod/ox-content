use super::super::super::{EntryPageConfig, PageChromeFlags, generate_html};
use super::{config, page, sidebar};

fn render_source(href: Option<&str>) -> String {
    let mut page_data = page();
    page_data.content = "<h1>Guide</h1>\n<p>Body.</p>\n<p class=\"ox-edit-this-page\"><a href=\"https://example.com/edit\">Edit this page</a></p>".to_string();
    page_data.markdown_source = href.map(ToOwned::to_owned);
    generate_html(&page_data, &sidebar(), &config(None, false))
}

#[test]
fn copy_as_markdown_is_omitted_when_source_is_off() {
    let html = render_source(None);

    assert!(!html.contains("ox-markdown-source"), "{html}");
    assert!(!html.contains("ox-copy-markdown"), "{html}");
    assert!(!html.contains("View Markdown"), "{html}");
    assert!(!html.contains("data-ox-copy-markdown"), "{html}");
    assert!(!html.contains("Copy as Markdown"), "{html}");
    assert!(!html.contains("ox-content:css:header-chrome"), "{html}");
}

#[test]
fn copy_as_markdown_points_at_the_companion_when_on() {
    let html = render_source(Some("/docs/guide.md"));

    assert!(html.contains(r#"<p class="ox-markdown-source">"#), "{html}");
    assert!(
        html.contains(r#"<a class="ox-view-markdown" href="/docs/guide.md">View Markdown</a>"#),
        "{html}"
    );
    assert!(
        html.contains(
            r#"<button type="button" class="ox-copy-markdown" data-ox-copy-markdown aria-label="Copy as Markdown">Copy as Markdown</button>"#
        ),
        "{html}"
    );
    assert!(html.contains("ox-copy-markdown-status"), "{html}");
    let after_title = html.split("</h1>").nth(1).expect("title");
    assert!(after_title.contains("ox-markdown-source"), "{after_title}");
    assert!(html.contains("ox-edit-this-page"), "{html}");
    assert!(html.contains("ox-content:css:header-chrome"), "{html}");
    assert!(html.contains("navigator.clipboard"), "{html}");
    assert!(html.contains("fetch("), "{html}");
    assert!(
        !html.contains("data-ox-copy-markdown-text") && !html.contains("---\ntitle:"),
        "copy must use companion bytes, not inlined source: {html}"
    );
}

#[test]
fn hide_edit_link_does_not_hide_markdown_source() {
    let mut page_data = page();
    page_data.content = "<h1>Guide</h1>\n<p class=\"ox-edit-this-page\"><a href=\"https://example.com/edit\">Edit</a></p>".to_string();
    page_data.markdown_source = Some("/docs/guide.md".into());
    page_data.chrome = PageChromeFlags { edit_link: Some(false), ..PageChromeFlags::default() };
    let html = generate_html(&page_data, &sidebar(), &config(None, true));

    assert!(html.contains("ox-hide-edit-link"), "{html}");
    assert!(html.contains("ox-markdown-source"), "{html}");
    assert!(html.contains("View Markdown"), "{html}");
}

#[test]
fn dangerous_companion_href_is_omitted() {
    for href in
        ["javascript:alert(1)", "data:text/markdown,hi", "//evil.example/x.md", "../secret.md"]
    {
        let html = render_source(Some(href));
        assert!(!html.contains("ox-markdown-source"), "{href}: {html}");
        assert!(!html.contains(href), "{href}: {html}");
    }
}

#[test]
fn hostile_companion_href_is_escaped() {
    let html = render_source(Some(r#"/guide.md"><img src=x onerror=alert(1)>.md"#));
    assert!(html.contains("ox-markdown-source"), "{html}");
    assert!(!html.contains(r#"href="/guide.md"><img"#), "{html}");
    assert!(!html.contains("<img src=x"), "{html}");
}

#[test]
fn entry_pages_skip_copy_as_markdown() {
    let mut page_data = page();
    page_data.content = "<h1>Home</h1>".to_string();
    page_data.markdown_source = Some("/index.md".into());
    page_data.entry_page = Some(EntryPageConfig::default());
    let html = generate_html(&page_data, &sidebar(), &config(None, false));
    assert!(!html.contains("ox-markdown-source"), "{html}");
}
