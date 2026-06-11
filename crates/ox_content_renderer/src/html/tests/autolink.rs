use crate::html::{HtmlRenderer, HtmlRendererOptions};
use ox_content_allocator::Allocator;
use ox_content_parser::Parser;

#[test]
fn test_autolink_disabled_by_default() {
    let allocator = Allocator::new();
    let doc = Parser::new(&allocator, "see http://example.com here").parse().unwrap();
    let mut renderer = HtmlRenderer::new();
    let html = renderer.render(&doc);
    // No <a> tag is emitted unless the flag is on.
    assert!(!html.contains("<a "), "unexpected autolink in: {html}");
    assert!(html.contains("http://example.com"));
}

#[test]
fn test_autolink_basic_http_and_https() {
    let allocator = Allocator::new();
    let doc =
        Parser::new(&allocator, "see http://example.com and https://example.org").parse().unwrap();
    let mut renderer = HtmlRenderer::with_options(HtmlRendererOptions {
        autolink_urls: true,
        ..Default::default()
    });
    let html = renderer.render(&doc);
    assert!(
            html.contains(
                "<a href=\"http://example.com\" target=\"_blank\" rel=\"noopener noreferrer\">http://example.com</a>"
            ),
            "missing http autolink in: {html}"
        );
    assert!(
            html.contains(
                "<a href=\"https://example.org\" target=\"_blank\" rel=\"noopener noreferrer\">https://example.org</a>"
            ),
            "missing https autolink in: {html}"
        );
}

#[test]
fn test_autolink_target_blank_can_be_disabled() {
    let allocator = Allocator::new();
    let doc = Parser::new(&allocator, "go to https://example.com now").parse().unwrap();
    let mut renderer = HtmlRenderer::with_options(HtmlRendererOptions {
        autolink_urls: true,
        autolink_target_blank: false,
        ..Default::default()
    });
    let html = renderer.render(&doc);
    assert!(
        html.contains("<a href=\"https://example.com\">https://example.com</a>"),
        "expected bare anchor in: {html}"
    );
    assert!(!html.contains("target=\"_blank\""), "blank attr leaked: {html}");
}

#[test]
fn test_autolink_strips_trailing_punctuation() {
    let allocator = Allocator::new();
    let doc =
        Parser::new(&allocator, "find it at https://example.com. or (https://example.org) maybe")
            .parse()
            .unwrap();
    let mut renderer = HtmlRenderer::with_options(HtmlRendererOptions {
        autolink_urls: true,
        ..Default::default()
    });
    let html = renderer.render(&doc);
    assert!(html.contains(">https://example.com</a>."), "period leaked: {html}");
    assert!(html.contains("(<a href=\"https://example.org\""), "open paren lost: {html}");
    assert!(html.contains(">https://example.org</a>)"), "close paren lost: {html}");
}

#[test]
fn test_autolink_word_boundary_required() {
    let allocator = Allocator::new();
    // "shttp://x" must not match — the prefix is glued to a word char.
    let doc = Parser::new(&allocator, "shttp://x and http://y").parse().unwrap();
    let mut renderer = HtmlRenderer::with_options(HtmlRendererOptions {
        autolink_urls: true,
        ..Default::default()
    });
    let html = renderer.render(&doc);
    assert!(!html.contains("href=\"http://x\""), "unexpected glued autolink: {html}");
    assert!(html.contains("href=\"http://y\""), "missing real autolink: {html}");
}

#[test]
fn test_autolink_custom_pattern_registration() {
    let allocator = Allocator::new();
    let doc = Parser::new(&allocator, "email mailto:foo@example.com please").parse().unwrap();
    let mut renderer = HtmlRenderer::with_options(HtmlRendererOptions {
        autolink_urls: true,
        autolink_patterns: vec!["mailto:".to_string()],
        ..Default::default()
    });
    let html = renderer.render(&doc);
    assert!(
        html.contains("<a href=\"mailto:foo@example.com\""),
        "missing custom-pattern autolink: {html}"
    );
}

#[test]
fn test_autolink_many_patterns_uses_table_fallback() {
    // Five patterns with five distinct leading letters exceed the
    // three-needle SIMD fast path, exercising the `FirstByteIndex`
    // lookup-table fallback. All schemes must still autolink.
    let allocator = Allocator::new();
    let doc = Parser::new(
        &allocator,
        "a http://h.test b ftp://f.test c mailto:m@x d tel:123 e ssh://s.test f",
    )
    .parse()
    .unwrap();
    let mut renderer = HtmlRenderer::with_options(HtmlRendererOptions {
        autolink_urls: true,
        autolink_patterns: vec![
            "http://".to_string(),
            "ftp://".to_string(),
            "mailto:".to_string(),
            "tel:".to_string(),
            "ssh://".to_string(),
        ],
        ..Default::default()
    });
    let html = renderer.render(&doc);
    for href in ["http://h.test", "ftp://f.test", "mailto:m@x", "tel:123", "ssh://s.test"] {
        let mut needle = String::with_capacity(href.len() + 9);
        needle.push_str("<a href=\"");
        needle.push_str(href);
        needle.push('"');
        assert!(html.contains(&needle), "missing {href} in: {html}");
    }
}

#[test]
fn test_autolink_does_not_nest_inside_existing_link() {
    let allocator = Allocator::new();
    // The text inside the explicit markdown link contains a URL — the
    // builtin must not wrap that URL in a second <a>.
    let doc = Parser::new(&allocator, "[visit https://example.com here](/page)").parse().unwrap();
    let mut renderer = HtmlRenderer::with_options(HtmlRendererOptions {
        autolink_urls: true,
        ..Default::default()
    });
    let html = renderer.render(&doc);
    assert_eq!(html.matches("<a ").count(), 1, "nested anchor in: {html}");
    assert!(html.contains("href=\"/page\""), "outer link lost: {html}");
    assert!(html.contains("visit https://example.com here"), "inner text lost: {html}");
}

#[test]
fn test_autolink_escapes_query_string_safely() {
    let allocator = Allocator::new();
    // `&` inside the URL must be escaped both as href and as visible
    // text — otherwise the output would be parser-ambiguous HTML.
    let doc = Parser::new(&allocator, "see http://a.test/?q=foo&r=bar now").parse().unwrap();
    let mut renderer = HtmlRenderer::with_options(HtmlRendererOptions {
        autolink_urls: true,
        ..Default::default()
    });
    let html = renderer.render(&doc);
    assert!(html.contains("href=\"http://a.test/?q=foo&amp;r=bar\""), "href not escaped: {html}");
    assert!(
        html.contains(">http://a.test/?q=foo&amp;r=bar</a>"),
        "visible text not escaped: {html}"
    );
}
