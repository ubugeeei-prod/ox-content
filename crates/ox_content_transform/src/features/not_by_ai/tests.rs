use super::{DEFAULT_HREF, DEFAULT_LABEL, resolve};
use crate::transformer::MarkdownTransformer;
use crate::{NotByAiOptions, TransformOptions};

fn transform_html(source: &str, options: TransformOptions) -> String {
    MarkdownTransformer::from_options(&options).transform(source).html
}

fn not_by_ai_on() -> TransformOptions {
    TransformOptions {
        gfm: Some(true),
        not_by_ai: Some(NotByAiOptions { enabled: Some(true), label: None, href: None }),
        ..Default::default()
    }
}

fn assert_badge(html: &str) {
    assert!(html.contains("class=\"ox-not-by-ai\""), "{html}");
    assert!(html.contains(&format!("href=\"{DEFAULT_HREF}\"")), "{html}");
    assert!(html.contains(&format!("aria-label=\"{DEFAULT_LABEL}\"")), "{html}");
    assert!(html.contains("target=\"_blank\""), "{html}");
    assert!(html.contains("rel=\"noopener noreferrer\""), "{html}");
    assert!(html.contains("ox-not-by-ai__badge--light"), "{html}");
    assert!(html.contains("ox-not-by-ai__badge--dark"), "{html}");
    assert!(html.contains("aria-hidden=\"true\""), "{html}");
    assert!(!html.contains("<script"), "{html}");
    assert!(!html.contains("data-ox-island"), "{html}");
}

#[test]
fn resolve_is_none_when_omitted_or_disabled() {
    assert!(resolve(None).is_none());
    assert!(
        resolve(Some(&NotByAiOptions { enabled: Some(false), label: None, href: None })).is_none()
    );
}

#[test]
fn resolve_is_some_when_object_is_present() {
    assert!(resolve(Some(&NotByAiOptions { enabled: None, label: None, href: None })).is_some());
    assert!(
        resolve(Some(&NotByAiOptions { enabled: Some(true), label: None, href: None })).is_some()
    );
}

#[test]
fn disabled_by_default() {
    let html = transform_html("<NotByAI />\n", TransformOptions::default());
    assert!(!html.contains("ox-not-by-ai"), "{html}");
    assert!(html.contains("NotByAI"), "{html}");
}

#[test]
fn compact_and_spaced_self_closing_forms() {
    for source in ["<NotByAI />\n", "<NotByAI/>\n", "<NotByAI  />\n"] {
        let html = transform_html(source, not_by_ai_on());
        assert_badge(&html);
        assert!(!html.contains("<NotByAI"), "{html}");
    }
}

#[test]
fn multiple_badges_on_one_line() {
    let html = transform_html("A <NotByAI /> and <NotByAI/>.\n", not_by_ai_on());
    assert_eq!(html.matches("class=\"ox-not-by-ai\"").count(), 2, "{html}");
}

#[test]
fn renders_in_paragraph_blockquote_list_and_callout() {
    let source = [
        "In a paragraph <NotByAI /> here.",
        "",
        "> quoted <NotByAI />",
        "",
        "- listed <NotByAI />",
        "",
        "> [!NOTE]",
        "> callout <NotByAI />",
        "",
    ]
    .join("\n");
    let html = transform_html(&source, not_by_ai_on());
    assert_badge(&html);
    assert!(html.contains("<blockquote>"), "{html}");
    assert!(html.contains("<li>"), "{html}");
    assert_eq!(html.matches("class=\"ox-not-by-ai\"").count(), 4, "{html}");
}

#[test]
fn skips_fenced_inline_and_indented_code() {
    let tag = "<NotByAI />";
    let html = transform_html(
        &format!("```md\n{tag}\n```\n\nUse `{tag}` here.\n\n    {tag}\n"),
        not_by_ai_on(),
    );
    assert!(!html.contains("ox-not-by-ai"), "{html}");
    assert_eq!(html.matches("&lt;NotByAI /&gt;").count(), 3, "{html}");
}

#[test]
fn skips_html_comments_including_multiline_and_reopen() {
    let html = transform_html(
        "<!-- <NotByAI /> --> visible <NotByAI /> <!--\n<NotByAI />\n-->\n",
        not_by_ai_on(),
    );
    assert_eq!(html.matches("class=\"ox-not-by-ai\"").count(), 1, "{html}");
    assert!(html.contains("&lt;NotByAI") || html.contains("<!--"), "{html}");
}

#[test]
fn malformed_tags_stay_literal() {
    for source in ["<NotByAI>\n", "<NotByAI></NotByAI>\n", "<NotByAI foo />\n", "<notbyai />\n"] {
        let html = transform_html(source, not_by_ai_on());
        assert!(!html.contains("ox-not-by-ai"), "{source} => {html}");
    }
}

#[test]
fn configured_label_and_href_are_escaped() {
    let html = transform_html(
        "<NotByAI />\n",
        TransformOptions {
            gfm: Some(true),
            not_by_ai: Some(NotByAiOptions {
                enabled: Some(true),
                label: Some("Human & \"safe\"".into()),
                href: Some("https://example.com/about?q=1".into()),
            }),
            ..Default::default()
        },
    );
    assert!(html.contains("aria-label=\"Human &amp; &quot;safe&quot;\""), "{html}");
    assert!(html.contains("href=\"https://example.com/about?q=1\""), "{html}");
}

#[test]
fn unsafe_href_falls_back_to_official_url() {
    for href in [
        "javascript:alert(1)",
        "data:text/html,x",
        "vbscript:msgbox(1)",
        "//evil.example",
        "https://example.com/\"onclick=alert(1)",
    ] {
        let html = transform_html(
            "<NotByAI />\n",
            TransformOptions {
                not_by_ai: Some(NotByAiOptions {
                    enabled: Some(true),
                    label: None,
                    href: Some(href.into()),
                }),
                ..Default::default()
            },
        );
        assert!(html.contains(&format!("href=\"{DEFAULT_HREF}\"")), "{href} => {html}");
        assert!(!html.contains("javascript:"), "{html}");
        assert!(!html.contains("data:"), "{html}");
    }
}

#[test]
fn md_and_mdx_emit_identical_static_markup() {
    let source = "Authored <NotByAI />.\n";
    let markdown = transform_html(source, not_by_ai_on());
    let mdx = transform_html(
        source,
        TransformOptions {
            gfm: Some(true),
            mdx: Some(true),
            not_by_ai: Some(NotByAiOptions { enabled: Some(true), label: None, href: None }),
            ..Default::default()
        },
    );
    assert_badge(&markdown);
    assert_eq!(markdown, mdx);
}

#[test]
fn vendored_svgs_survive_sanitization() {
    let html = transform_html("<NotByAI />\n", not_by_ai_on());
    assert!(
        html.contains("<svg class=\"ox-not-by-ai__badge ox-not-by-ai__badge--light\""),
        "{html}"
    );
    assert!(
        html.contains("<svg class=\"ox-not-by-ai__badge ox-not-by-ai__badge--dark\""),
        "{html}"
    );
    assert!(html.contains("viewBox=\"0 0 131 42\""), "{html}");
}
