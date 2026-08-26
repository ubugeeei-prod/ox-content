use super::resolve;
use crate::transformer::MarkdownTransformer;
use crate::{DefinitionListOptions, TransformOptions};

fn transform_html(source: &str, options: TransformOptions) -> String {
    MarkdownTransformer::from_options(&options).transform(source).html
}

fn lists_on() -> TransformOptions {
    lists_options(DefinitionListOptions { enabled: Some(true) })
}

fn lists_options(options: DefinitionListOptions) -> TransformOptions {
    TransformOptions { gfm: Some(true), definition_lists: Some(options), ..Default::default() }
}

#[test]
fn resolve_is_none_when_omitted_or_disabled() {
    assert!(resolve(None).is_none());
    assert!(resolve(Some(&DefinitionListOptions { enabled: Some(false) })).is_none());
}

#[test]
fn resolve_is_some_when_object_is_present() {
    assert!(resolve(Some(&DefinitionListOptions::default())).is_some());
    assert!(resolve(Some(&DefinitionListOptions { enabled: Some(true) })).is_some());
}

#[test]
fn disabled_by_default_leaves_source_as_paragraphs() {
    let source = "HTTP\n: Hypertext Transfer Protocol\n";
    let html = transform_html(source, TransformOptions::default());
    assert!(!html.contains("ox-definition-list"), "{html}");
    assert!(!html.contains("<dl"), "{html}");
    assert!(html.contains("HTTP"), "{html}");
    assert!(html.contains(": Hypertext Transfer Protocol"), "{html}");
}

#[test]
fn disabled_object_leaves_source_as_paragraphs() {
    let source = "HTTP\n: Hypertext Transfer Protocol\n";
    let html =
        transform_html(source, lists_options(DefinitionListOptions { enabled: Some(false) }));
    assert!(!html.contains("ox-definition-list"), "{html}");
    assert!(html.contains(": Hypertext Transfer Protocol"), "{html}");
}

#[test]
fn feature_off_matches_feature_on_when_markup_is_absent() {
    let source = "Hello **world** and a [link](https://example.com).\n";
    let off = transform_html(source, TransformOptions { gfm: Some(true), ..Default::default() });
    let on = transform_html(source, lists_on());
    assert_eq!(off, on);
}

#[test]
fn renders_term_and_definition() {
    let html = transform_html("HTTP\n: Hypertext Transfer Protocol\n", lists_on());
    assert!(html.contains("<dl class=\"ox-definition-list\">"), "{html}");
    assert!(html.contains("<dt>"), "{html}");
    assert!(html.contains("HTTP"), "{html}");
    assert!(html.contains("<dd>"), "{html}");
    assert!(html.contains("Hypertext Transfer Protocol"), "{html}");
    assert!(!html.contains(": Hypertext"), "{html}");
}

#[test]
fn renders_multiple_definitions_for_one_term() {
    let html = transform_html("Apple\n: A fruit\n: A computer company\n", lists_on());
    let dd_count = html.matches("<dd>").count();
    assert_eq!(dd_count, 2, "{html}");
    assert!(html.contains("A fruit"), "{html}");
    assert!(html.contains("A computer company"), "{html}");
}

#[test]
fn renders_multiple_items_in_one_list() {
    let html = transform_html(
        "HTTP\n: Hypertext Transfer Protocol\n\nTLS\n: Transport Layer Security\n",
        lists_on(),
    );
    assert_eq!(html.matches("<dl class=\"ox-definition-list\">").count(), 1, "{html}");
    assert_eq!(html.matches("<dt>").count(), 2, "{html}");
    assert!(html.contains("TLS"), "{html}");
}

#[test]
fn keeps_inline_markdown_in_terms_and_definitions() {
    let html = transform_html("**HTTP**\n: Hypertext *Transfer* `Protocol`\n", lists_on());
    assert!(html.contains("<strong>HTTP</strong>"), "{html}");
    assert!(html.contains("<em>Transfer</em>"), "{html}");
    assert!(html.contains("<code>Protocol</code>"), "{html}");
}

#[test]
fn accepts_blank_line_between_term_and_definition() {
    let html = transform_html("HTTP\n\n: Hypertext Transfer Protocol\n", lists_on());
    assert!(html.contains("<dl class=\"ox-definition-list\">"), "{html}");
    assert!(html.contains("Hypertext Transfer Protocol"), "{html}");
}

#[test]
fn skips_fenced_code() {
    let source = "```md\nHTTP\n: Hypertext Transfer Protocol\n```\n";
    let html = transform_html(source, lists_on());
    assert!(!html.contains("ox-definition-list"), "{html}");
    assert!(html.contains(": Hypertext Transfer Protocol"), "{html}");
}

#[test]
fn skips_inline_code() {
    let html = transform_html("Use `HTTP\n: not a list` in docs.\n", lists_on());
    assert!(!html.contains("ox-definition-list"), "{html}");
    assert!(html.contains(": not a list"), "{html}");
}

#[test]
fn skips_indented_code() {
    let html = transform_html("    HTTP\n    : Hypertext Transfer Protocol\n", lists_on());
    assert!(!html.contains("ox-definition-list"), "{html}");
}

#[test]
fn lone_definition_stays_a_paragraph() {
    let html = transform_html(": Hypertext Transfer Protocol\n", lists_on());
    assert!(!html.contains("ox-definition-list"), "{html}");
    assert!(html.contains(": Hypertext Transfer Protocol"), "{html}");
}

#[test]
fn list_item_followed_by_colon_stays_a_list() {
    let html = transform_html("- Apple\n: a fruit\n", lists_on());
    assert!(!html.contains("ox-definition-list"), "{html}");
    assert!(html.contains("<li>"), "{html}");
    assert!(html.contains(": a fruit"), "{html}");
}

#[test]
fn consecutive_terms_share_the_following_definitions() {
    let html = transform_html("Apple\nMalus\n: A fruit\n", lists_on());
    assert_eq!(html.matches("<dt>").count(), 2, "{html}");
    assert_eq!(html.matches("<dd>").count(), 1, "{html}");
    assert!(html.contains("Apple"), "{html}");
    assert!(html.contains("Malus"), "{html}");
}

#[test]
fn heading_followed_by_colon_stays_a_heading() {
    let html = transform_html("# HTTP\n: Hypertext Transfer Protocol\n", lists_on());
    assert!(!html.contains("ox-definition-list"), "{html}");
    assert!(html.contains("<h1"), "{html}");
}

#[test]
fn emoji_shortcode_line_is_not_a_definition() {
    let html = transform_html("Status\n:smile:\n", lists_on());
    assert!(!html.contains("ox-definition-list"), "{html}");
}

#[test]
fn container_opener_is_not_a_definition() {
    let html = transform_html("Note\n::: tip\nHello\n:::\n", lists_on());
    assert!(!html.contains("ox-definition-list"), "{html}");
}
