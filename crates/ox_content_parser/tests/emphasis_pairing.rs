//! Emphasis pairing rewrites `children` in place: paired nodes are lifted
//! out and empty text is left in their slots, so `node_index` never moves
//! and spent delimiters are retired by zeroing their count rather than by
//! erasing them from the vector.
//!
//! Both of those are invisible in the HTML when they work and catastrophic
//! when they do not, so these tests check the tree as well as the output —
//! no placeholder may survive, and pairing must stay linear.

use std::time::{Duration, Instant};

use ox_content_allocator::Allocator;
use ox_content_ast::Node;
use ox_content_parser::{Parser, ParserOptions};
use ox_content_renderer::HtmlRenderer;

fn render(source: &str) -> String {
    let allocator = Allocator::new();
    let document = Parser::with_options(&allocator, source, ParserOptions::gfm())
        .parse()
        .expect("source should parse");
    HtmlRenderer::new().render(&document).trim().to_string()
}

fn children_of<'a, 'b: 'a>(node: &'a Node<'b>) -> Option<&'a [Node<'b>]> {
    match node {
        Node::Paragraph(n) => Some(&n.children),
        Node::Heading(n) => Some(&n.children),
        Node::Emphasis(n) => Some(&n.children),
        Node::Strong(n) => Some(&n.children),
        Node::Delete(n) => Some(&n.children),
        Node::Link(n) => Some(&n.children),
        Node::BlockQuote(n) => Some(&n.children),
        _ => None,
    }
}

/// Empty text nodes are placeholders left behind by pairing; none may
/// reach a consumer, at any depth.
fn assert_no_empty_text(nodes: &[Node<'_>], source: &str) {
    for node in nodes {
        if let Node::Text(text) = node {
            assert!(!text.value.is_empty(), "empty text node survived pairing in {source:?}");
        }
        if let Some(children) = children_of(node) {
            assert_no_empty_text(children, source);
        }
    }
}

fn assert_spans_inside(nodes: &[Node<'_>], limit: u32, source: &str) {
    for node in nodes {
        if let Node::Emphasis(n) = node {
            assert!(n.span.start <= n.span.end && n.span.end <= limit, "bad span in {source:?}");
        }
        if let Node::Strong(n) = node {
            assert!(n.span.start <= n.span.end && n.span.end <= limit, "bad span in {source:?}");
        }
        if let Some(children) = children_of(node) {
            assert_spans_inside(children, limit, source);
        }
    }
}

fn check_tree(source: &str) {
    let allocator = Allocator::new();
    let document = Parser::with_options(&allocator, source, ParserOptions::gfm())
        .parse()
        .expect("source should parse");
    assert_no_empty_text(&document.children, source);
    assert_spans_inside(&document.children, source.len() as u32, source);
}

fn time_parse(source: &str) -> Duration {
    let allocator = Allocator::new();
    let start = Instant::now();
    let parsed = Parser::with_options(&allocator, source, ParserOptions::gfm()).parse();
    let elapsed = start.elapsed();
    assert!(parsed.is_ok(), "source should parse");
    elapsed
}

const NESTING_CASES: [(&str, &str); 12] = [
    ("*a **b** c*", "<p><em>a <strong>b</strong> c</em></p>"),
    ("**a *b* c**", "<p><strong>a <em>b</em> c</strong></p>"),
    ("***abc***", "<p><em><strong>abc</strong></em></p>"),
    ("*a *b* c*", "<p><em>a <em>b</em> c</em></p>"),
    ("_a_b_c_", "<p><em>a_b_c</em></p>"),
    ("a*b*c*d*e", "<p>a<em>b</em>c<em>d</em>e</p>"),
    ("**a**b**c**", "<p><strong>a</strong>b<strong>c</strong></p>"),
    ("*(**foo**)*", "<p><em>(<strong>foo</strong>)</em></p>"),
    ("foo***bar***baz", "<p>foo<em><strong>bar</strong></em>baz</p>"),
    ("*a `b` c*", "<p><em>a <code>b</code> c</em></p>"),
    ("**[l](u)**", "<p><strong><a href=\"u\">l</a></strong></p>"),
    ("*foo **bar** baz*", "<p><em>foo <strong>bar</strong> baz</em></p>"),
];

#[test]
fn nested_pairs_keep_their_shape() {
    for (source, expected) in NESTING_CASES {
        assert_eq!(render(source), expected, "for {source:?}");
        check_tree(source);
    }
}

#[test]
fn retired_delimiters_never_pair_across_a_finished_span() {
    // The inner `*` runs are consumed by the inner pair; if they stayed
    // available they would reach past the closing `*` and re-pair.
    assert_eq!(render("*a *b* c* d*"), "<p><em>a <em>b</em> c</em> d*</p>");
    assert_eq!(render("**a *b* c** d*"), "<p><strong>a <em>b</em> c</strong> d*</p>");
    check_tree("*a *b* c* d*");
}

#[test]
fn a_failed_search_does_not_hide_a_later_opener() {
    // The first `*` cannot close; the search that fails for it must not
    // bound the search for the closers that follow.
    assert_eq!(render("* a *b* c"), "<ul>\n<li>a <em>b</em> c</li>\n</ul>");
    assert_eq!(render("a* b *c* d"), "<p>a* b <em>c</em> d</p>");
    assert_eq!(render("_a _b_ c"), "<p>_a <em>b</em> c</p>");
    // Rule of three: `**` cannot close `*`, but the `*` after it can.
    assert_eq!(render("*foo**bar*"), "<p><em>foo**bar</em></p>");
}

#[test]
fn every_pair_in_a_long_sequence_is_formed() {
    let source = "**bold** and *ital* ".repeat(400);
    let rendered = render(&source);
    assert_eq!(rendered.matches("<strong>").count(), 400);
    assert_eq!(rendered.matches("<em>").count(), 400);
    check_tree(&source);
}

#[test]
fn a_long_sequence_of_unpairable_delimiters_stays_literal() {
    // Space on both sides means each run can neither open nor close, so
    // every opener search fails — the path `openers_bottom` short-circuits.
    let source = "x * ".repeat(400);
    let rendered = render(&source);
    assert_eq!(rendered.matches("<em>").count(), 0);
    assert_eq!(rendered.matches("<strong>").count(), 0);
    assert_eq!(rendered.matches('*').count(), 400, "every marker stays literal");
    check_tree(&source);

    // Intraword `_` cannot open or close either.
    let source = "a_b".repeat(400);
    let rendered = render(&source);
    assert_eq!(rendered.matches("<em>").count(), 0);
    assert_eq!(rendered.matches('_').count(), 400);
    check_tree(&source);
}

#[test]
fn emphasis_pairing_costs_linear_time() {
    // Four times the delimiters used to cost sixteen times the work, both
    // when they pair and when they cannot. Anything under eight proves the
    // per-pairing vector walk is gone without pinning an absolute time.
    let builds: [fn(usize) -> String; 3] =
        [|n| "**bold** and *ital* ".repeat(n), |n| "x * ".repeat(n), |n| "*_".repeat(n) + "a"];
    for build in builds {
        let small = time_parse(&build(1_000)).max(Duration::from_micros(1));
        let large = time_parse(&build(4_000));
        assert!(large < small * 8, "4x the delimiters took {large:?} against {small:?}");
    }
}
