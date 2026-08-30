//! Footnote slug uniquification.
//!
//! Two footnotes whose identifiers slugify the same have to end up with
//! different ids, or the page carries duplicate anchors. Finding a free
//! slug used to scan every footnote already emitted, which made a page of
//! many footnotes quadratic; the ids it produced are what these tests pin.

#[path = "support/edge.rs"]
mod edge_support;

use std::time::{Duration, Instant};

use edge_support::render;
use ox_content_allocator::Allocator;
use ox_content_parser::{Parser, ParserOptions};
use ox_content_renderer::{HtmlRenderer, HtmlRendererOptions};

fn semantic(source: &str) -> String {
    render(
        source,
        ParserOptions::gfm(),
        HtmlRendererOptions { semantic_footnotes: true, ..HtmlRendererOptions::default() },
    )
}

/// The `fn-…` ids in document order.
fn footnote_ids(html: &str) -> Vec<String> {
    html.match_indices("id=\"fn-")
        .map(|(at, _)| {
            let rest = &html[at + 7..];
            rest[..rest.find('"').unwrap_or(0)].to_string()
        })
        .collect()
}

fn definitions(identifiers: &[&str]) -> String {
    let mut source = String::new();
    for identifier in identifiers {
        source.push_str("[^");
        source.push_str(identifier);
        source.push_str("]: body\n\n");
    }
    source
}

#[test]
fn colliding_slugs_are_numbered_in_document_order() {
    assert_eq!(footnote_ids(&semantic(&definitions(&["x!", "x?", "x."]))), ["x", "x-2", "x-3"]);
    assert_eq!(
        footnote_ids(&semantic(&definitions(&["p.q", "p!q", "p?q"]))),
        ["p-q", "p-q-2", "p-q-3"]
    );
}

#[test]
fn a_slug_that_is_already_a_suffixed_form_is_not_taken_twice() {
    // `x-2` arrives on its own, so the collision after it has to skip past
    // the id that footnote already holds.
    assert_eq!(footnote_ids(&semantic(&definitions(&["x!", "x-2", "x?"]))), ["x", "x-2", "x-3"]);
    assert_eq!(footnote_ids(&semantic(&definitions(&["x-2", "x!", "x?"]))), ["x-2", "x", "x-3"]);
    // And a suffixed form that collides in turn gets suffixed itself.
    assert_eq!(
        footnote_ids(&semantic(&definitions(&["x!", "x?", "x-2", "x."]))),
        ["x", "x-2", "x-2-2", "x-3"]
    );
}

#[test]
fn identifiers_without_a_usable_slug_fall_back_to_their_position() {
    assert_eq!(
        footnote_ids(&semantic(&definitions(&["!", "?", "@"]))),
        ["footnote-1", "footnote-2", "footnote-3"]
    );
}

#[test]
fn distinct_identifiers_keep_their_own_slugs() {
    let ids = footnote_ids(&semantic(&definitions(&["alpha", "beta", "gamma"])));
    assert_eq!(ids, ["alpha", "beta", "gamma"]);
}

#[test]
fn slugs_do_not_carry_over_between_renders() {
    let options =
        HtmlRendererOptions { semantic_footnotes: true, ..HtmlRendererOptions::default() };
    let mut renderer = HtmlRenderer::with_options(options);
    let source = definitions(&["x!", "x?"]);
    let allocator = Allocator::new();
    let document = Parser::with_options(&allocator, &source, ParserOptions::gfm())
        .parse()
        .expect("source should parse");

    let first = renderer.render(&document);
    let second = renderer.render(&document);
    assert_eq!(footnote_ids(&first), ["x", "x-2"]);
    assert_eq!(footnote_ids(&second), ["x", "x-2"], "a second render must start clean");
}

#[test]
fn a_provisional_fragment_does_not_claim_slugs() {
    // `render_provisional_fragment` is meant to be replaceable, so nothing
    // it renders may push the real render's ids along.
    let options =
        HtmlRendererOptions { semantic_footnotes: true, ..HtmlRendererOptions::default() };
    let mut renderer = HtmlRenderer::with_options(options);
    let source = definitions(&["x!", "x?"]);
    let allocator = Allocator::new();
    let document = Parser::with_options(&allocator, &source, ParserOptions::gfm())
        .parse()
        .expect("source should parse");

    let provisional = renderer.render_provisional_fragment(&document);
    let final_html = renderer.render_provisional_fragment(&document);
    assert_eq!(footnote_ids(&provisional), footnote_ids(&final_html));
}

#[test]
fn many_footnotes_cost_linear_time() {
    // Four times the footnotes used to cost sixteen times the work, because
    // each one scanned every footnote before it.
    let build = |count: usize| {
        let mut source = String::new();
        for index in 0..count {
            source.push_str("[^n");
            source.push_str(&index.to_string());
            source.push_str("]: body\n\n");
        }
        source
    };
    let measure = |source: &str| {
        let allocator = Allocator::new();
        let document = Parser::with_options(&allocator, source, ParserOptions::gfm())
            .parse()
            .expect("source should parse");
        let mut renderer = HtmlRenderer::with_options(HtmlRendererOptions {
            semantic_footnotes: true,
            ..HtmlRendererOptions::default()
        });
        let start = Instant::now();
        let html = renderer.render(&document);
        assert!(!html.is_empty());
        start.elapsed()
    };

    let small = measure(&build(2_000)).max(Duration::from_micros(1));
    let large = measure(&build(8_000));
    assert!(large < small * 8, "8,000 footnotes took {large:?} against {small:?} for 2,000");
}
