//! A `[` can only open a link when a `]` follows it, but `scan_balanced`
//! answered that by walking to the end of the content — once for every
//! bracket. A run of brackets with no closer therefore cost O(n²): 64 KiB
//! of `[ ` took 1.0 s and grew x16 for every x4 of input, and `[[ ` took
//! 2.0 s. That is ordinary text — a shell snippet outside a fence, a log
//! paste, BibTeX, a glob pattern — not a crafted document.
//!
//! These tests pin the cost back to linear and pin the parse that the
//! early exit now short-circuits, down to the text nodes and their spans:
//! the bracket has to stay literal in exactly the shape the slow path
//! produced.

use std::sync::mpsc;
use std::thread;
use std::time::{Duration, Instant};

use ox_content_allocator::Allocator;
use ox_content_parser::{Parser, ParserOptions};
use ox_content_renderer::HtmlRenderer;

#[path = "support/pretty.rs"]
mod pretty;

/// Generous enough that a slow shared runner never trips it, and far below
/// what the quadratic path needed for these sizes.
const BUDGET: Duration = Duration::from_secs(30);

fn render(source: &str) -> String {
    let allocator = Allocator::new();
    let document = Parser::with_options(&allocator, source, ParserOptions::gfm())
        .parse()
        .expect("source should parse");
    HtmlRenderer::new().render(&document).trim().to_string()
}

fn tree(source: &str) -> String {
    let allocator = Allocator::new();
    let document = Parser::with_options(&allocator, source, ParserOptions::gfm())
        .parse()
        .expect("source should parse");
    let mut out = String::new();
    pretty::format_document(&document, source, &mut out);
    out
}

fn repeat_to(unit: &str, bytes: usize) -> String {
    let mut out = String::with_capacity(bytes + unit.len());
    while out.len() < bytes {
        out.push_str(unit);
    }
    out
}

/// Parses on a worker thread so a regression fails the suite in bounded
/// time instead of hanging it. Best of two, so one scheduling stall on a
/// busy runner cannot fail the build.
fn parse_within_budget(source: &str) -> Duration {
    let mut best = BUDGET;
    for _ in 0..2 {
        let owned = source.to_string();
        let (sender, receiver) = mpsc::channel();
        thread::spawn(move || {
            let started = Instant::now();
            let allocator = Allocator::new();
            let parsed =
                Parser::with_options(&allocator, &owned, ParserOptions::gfm()).parse().is_ok();
            let _ = sender.send((parsed, started.elapsed()));
        });
        let (parsed, elapsed) = receiver
            .recv_timeout(BUDGET)
            .expect("a run of unclosed brackets should parse in bounded time");
        assert!(parsed, "a run of unclosed brackets should parse to a document");
        best = best.min(elapsed);
    }
    best
}

#[test]
fn a_run_of_unclosed_brackets_costs_linear_time() {
    // Each shape reaches the guard through a different branch: a bare
    // opener, an image opener, a nested opener, a footnote-looking opener,
    // and an opener with text after it.
    for unit in ["[ ", "[", "![ ", "[[ ", "[^ ", "[a "] {
        let small = parse_within_budget(&repeat_to(unit, 32 * 1024));
        let large = parse_within_budget(&repeat_to(unit, 128 * 1024));

        let ratio = large.as_secs_f64() / small.as_secs_f64().max(1e-9);
        // 4x the input. Linear costs about 4x the time; quadratic costs
        // 16x, which is what every one of these measured before the fix.
        assert!(
            ratio < 8.0,
            "{unit:?}: 128 KiB took {large:?} against {small:?} for 32 KiB (x{ratio:.1}); \
             linear is about x4, quadratic about x16"
        );
    }
}

#[test]
fn an_unclosed_bracket_stays_literal() {
    // The early exit replaces a scan that ended in the literal-bracket
    // fallback, so every one of these has to render exactly as it did.
    for (source, expected) in [
        ("[", "<p>[</p>"),
        ("a [ b", "<p>a [ b</p>"),
        ("[ [ [", "<p>[ [ [</p>"),
        ("![", "<p>![</p>"),
        ("a ![ b", "<p>a ![ b</p>"),
        ("[^x", "<p>[^x</p>"),
        ("[a](b", "<p>[a](b</p>"),
        ("[a][b", "<p>[a][b</p>"),
        ("\\[ a", "<p>[ a</p>"),
        ("prefix [ suffix ] tail", "<p>prefix [ suffix ] tail</p>"),
    ] {
        assert_eq!(render(source), expected, "for {source:?}");
    }
}

#[test]
fn a_bracket_that_does_close_still_links() {
    // The guard must not swallow the cases that do have a closer, in any
    // of the four accepting forms.
    for (source, expected) in [
        ("[a]", "<p>[a]</p>"),
        ("[a](/u)", r#"<p><a href="/u">a</a></p>"#),
        ("[a][b]\n\n[b]: /u", r#"<p><a href="/u">a</a></p>"#),
        ("[a]\n\n[a]: /u", r#"<p><a href="/u">a</a></p>"#),
        ("[a][]\n\n[a]: /u", r#"<p><a href="/u">a</a></p>"#),
        ("![a](b.png)", r#"<p><img src="b.png" alt="a"></p>"#),
        ("![a][b]\n\n[b]: /u.png", r#"<p><img src="/u.png" alt="a"></p>"#),
        ("[`]`", "<p>[<code>]</code></p>"),
    ] {
        assert_eq!(render(source), expected, "for {source:?}");
    }
}

#[test]
fn the_literal_fallback_keeps_its_node_shape() {
    // HTML alone would not catch a changed split: `![` has to stay one
    // two-byte text node, not a `!` node followed by a `[` node, or spans
    // reported to editors and diagnostics would shift.
    assert_eq!(
        tree("a ![ b"),
        "Document [0..6]\n  Paragraph [0..6]\n    Text \"a \" [0..2]\n    \
         Text \"![\" [2..4]\n    Text \" b\" [4..6]\n"
    );
    assert_eq!(
        tree("[ [ ["),
        "Document [0..5]\n  Paragraph [0..5]\n    Text \"[\" [0..1]\n    \
         Text \" \" [1..2]\n    Text \"[\" [2..3]\n    Text \" \" [3..4]\n    \
         Text \"[\" [4..5]\n"
    );
}
