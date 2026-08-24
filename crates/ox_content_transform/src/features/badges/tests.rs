use super::resolve;
use crate::transformer::MarkdownTransformer;
use crate::{BadgeOptions, TransformOptions};

const ALLOWED: &[&str] =
    &["tip", "note", "info", "warning", "danger", "success", "deprecated", "required"];

fn transform_html(source: &str, options: TransformOptions) -> String {
    MarkdownTransformer::from_options(&options).transform(source).html
}

fn badges_on() -> TransformOptions {
    TransformOptions {
        gfm: Some(true),
        badges: Some(BadgeOptions { enabled: Some(true) }),
        ..Default::default()
    }
}

fn authoring(variant: &str, text: &str) -> String {
    format!("{{badge:{variant}}}{text}{{/badge}}")
}

fn expected_span(variant: &str, text: &str) -> String {
    format!(r#"<span class="ox-badge ox-badge--{variant}">{text}</span>"#)
}

#[test]
fn resolve_is_false_when_option_is_omitted() {
    assert!(!resolve(None));
}

#[test]
fn resolve_is_false_when_explicitly_disabled() {
    assert!(!resolve(Some(&BadgeOptions { enabled: Some(false) })));
}

#[test]
fn resolve_is_true_when_object_is_present() {
    assert!(resolve(Some(&BadgeOptions { enabled: None })));
    assert!(resolve(Some(&BadgeOptions { enabled: Some(true) })));
}

#[test]
fn disabled_by_default() {
    let source = authoring("tip", "Beta");
    let html = transform_html(&source, TransformOptions::default());
    assert!(!html.contains("ox-badge"), "default transform must not emit badges:\n{html}");
    assert!(html.contains(&source), "source must remain literal:\n{html}");
}

#[test]
fn disabled_object_leaves_source_literal() {
    let source = authoring("tip", "Beta");
    let html = transform_html(
        &source,
        TransformOptions {
            badges: Some(BadgeOptions { enabled: Some(false) }),
            ..Default::default()
        },
    );
    assert!(!html.contains("ox-badge"), "{html}");
    assert!(html.contains(&source), "{html}");
}

#[test]
fn happy_path_each_allowed_variant() {
    for variant in ALLOWED {
        let source = authoring(variant, "TEXT");
        let html = transform_html(&source, badges_on());
        let span = expected_span(variant, "TEXT");
        assert!(html.contains(&span), "variant {variant} missing {span}:\n{html}");
        assert!(!html.contains(&source), "authoring form must be rewritten:\n{html}");
    }
}

#[test]
fn skips_fenced_code() {
    let source = authoring("tip", "Beta");
    let html = transform_html(&format!("```md\n{source}\n```\n"), badges_on());
    assert!(!html.contains(r#"class="ox-badge"#), "fenced code must stay literal:\n{html}");
    assert!(html.contains(&source), "{html}");
}

#[test]
fn skips_inline_code() {
    let source = authoring("tip", "Beta");
    let html = transform_html(&format!("Use `{source}` in docs.\n"), badges_on());
    assert!(!html.contains(r#"class="ox-badge"#), "inline code must stay literal:\n{html}");
    assert!(html.contains(&source), "{html}");
}

#[test]
fn skips_indented_code() {
    let source = authoring("tip", "Beta");
    let html = transform_html(&format!("    {source}\n"), badges_on());
    assert!(!html.contains(r#"class="ox-badge"#), "indented code must stay literal:\n{html}");
    assert!(html.contains(&source), "{html}");
}

#[test]
fn unknown_variant_stays_literal() {
    let source = authoring("spaceship", "X");
    let html = transform_html(&source, badges_on());
    assert!(!html.contains("ox-badge"), "{html}");
    assert!(html.contains(&source), "{html}");
}

#[test]
fn unclosed_stays_literal() {
    let source = concat!("{", "badge:tip}X").to_string();
    let html = transform_html(&source, badges_on());
    assert!(!html.contains("ox-badge"), "{html}");
    assert!(html.contains(&source), "{html}");
}

#[test]
fn empty_variant_stays_literal() {
    let source = authoring("", "X");
    let html = transform_html(&source, badges_on());
    assert!(!html.contains("ox-badge"), "{html}");
    assert!(html.contains(&source), "{html}");
}

#[test]
fn hostile_text_escaped() {
    let html = transform_html(&authoring("tip", "<script>alert(1)</script>"), badges_on());
    assert!(html.contains(r#"<span class="ox-badge ox-badge--tip">"#), "{html}");
    assert!(html.contains("&lt;script&gt;"), "{html}");
    assert!(!html.contains("<script>"), "user text must not emit a raw script tag:\n{html}");
    assert!(!html.contains("style="), "{html}");
    assert!(!html.contains("onerror"), "{html}");
}

#[test]
fn uppercase_variant_stays_literal() {
    let source = authoring("TIP", "X");
    let html = transform_html(&source, badges_on());
    assert!(!html.contains("ox-badge"), "{html}");
    assert!(html.contains(&source), "{html}");
}
