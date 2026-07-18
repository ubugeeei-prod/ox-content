use crate::html::{HtmlRenderer, HtmlRendererOptions};
use ox_content_allocator::Allocator;
use ox_content_parser::Parser;

#[test]
fn test_autolink_enabled_by_default() {
    let allocator = Allocator::new();
    let doc = Parser::new(&allocator, "see http://example.com here").parse().unwrap();
    let mut renderer = HtmlRenderer::new();
    let html = renderer.render(&doc);
    insta::assert_snapshot!(html);
}

#[test]
fn test_autolink_can_be_disabled() {
    let allocator = Allocator::new();
    let doc = Parser::new(&allocator, "see http://example.com here").parse().unwrap();
    let mut renderer = HtmlRenderer::with_options(HtmlRendererOptions {
        autolink_urls: false,
        ..Default::default()
    });
    let html = renderer.render(&doc);
    insta::assert_snapshot!(html);
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
    insta::assert_snapshot!(html);
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
    insta::assert_snapshot!(html);
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
    insta::assert_snapshot!(html);
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
    insta::assert_snapshot!(html);
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
    insta::assert_snapshot!(html);
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
    insta::assert_snapshot!(html);
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
    insta::assert_snapshot!(html);
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
    insta::assert_snapshot!(html);
}

#[test]
fn test_autolink_prose_full_of_candidate_first_bytes_stays_plain() {
    let allocator = Allocator::new();
    // Every `h` here hits the first-byte index; the second-byte filter
    // must reject them all without producing links (and without panicking
    // when the candidate is the last byte of the text).
    let doc =
        Parser::new(&allocator, "the theory holds hereabouts, hush; ends with h").parse().unwrap();
    let mut renderer = HtmlRenderer::with_options(HtmlRendererOptions {
        autolink_urls: true,
        ..Default::default()
    });
    let html = renderer.render(&doc);
    assert!(!html.contains("<a "), "unexpected link in: {html}");
}

#[test]
fn test_autolink_uppercase_prefix_still_matches_through_filter() {
    let allocator = Allocator::new();
    let doc = Parser::new(&allocator, "loud HTTPS://CAPS.test here").parse().unwrap();
    let mut renderer = HtmlRenderer::with_options(HtmlRendererOptions {
        autolink_urls: true,
        ..Default::default()
    });
    let html = renderer.render(&doc);
    assert_eq!(html.matches("<a ").count(), 1, "missing uppercase autolink in: {html}");
}

#[test]
fn test_autolink_conflicting_second_bytes_disable_the_filter() {
    let allocator = Allocator::new();
    // Two patterns share the first byte but disagree on the second; the
    // filter must fall back to full prefix checks so both still match.
    let doc = Parser::new(&allocator, "a http://one.test b hxxp://two.test c").parse().unwrap();
    let mut renderer = HtmlRenderer::with_options(HtmlRendererOptions {
        autolink_urls: true,
        autolink_patterns: vec!["http://".to_string(), "hxxp://".to_string()],
        ..Default::default()
    });
    let html = renderer.render(&doc);
    assert_eq!(html.matches("<a ").count(), 2, "expected both schemes in: {html}");
}

#[test]
fn test_autolink_single_byte_pattern_bypasses_the_filter() {
    let allocator = Allocator::new();
    // A one-byte pattern has no second byte to filter on; candidates must
    // go straight to the prefix check.
    let doc = Parser::new(&allocator, "go htail now").parse().unwrap();
    let mut renderer = HtmlRenderer::with_options(HtmlRendererOptions {
        autolink_urls: true,
        autolink_patterns: vec!["h".to_string()],
        ..Default::default()
    });
    let html = renderer.render(&doc);
    assert_eq!(html.matches("<a ").count(), 1, "single-byte pattern should link in: {html}");
}
