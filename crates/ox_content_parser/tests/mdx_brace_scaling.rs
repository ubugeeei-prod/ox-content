//! MDX braces used to cost O(n²), through two separate paths.
//!
//! `{` is not in the inline classifier's byte set, so with MDX on the scan
//! for it ran only *after* a scan for the next real marker — which, in
//! prose whose only markers are braces, walks to the end of the content.
//! Every brace paid that walk, so a line of `{a} ` cost quadratic time even
//! though each expression closed immediately.
//!
//! Separately, the balanced brace scan only reports that nothing closed
//! after walking to the end, so a run of *unclosed* braces paid one walk
//! each — inline and, through the flow-expression block check, per line.
//!
//! 32 KiB measured 0.0051s for `{a} `, 0.2195s for `{ ` and 0.2051s for
//! lines of `{`, all growing x16 for every x4 of input.

use std::sync::mpsc;
use std::thread;
use std::time::{Duration, Instant};

use ox_content_allocator::Allocator;
use ox_content_parser::{Parser, ParserOptions};

#[path = "support/pretty.rs"]
mod pretty;

const BUDGET: Duration = Duration::from_secs(30);

fn mdx_options() -> ParserOptions {
    let mut options = ParserOptions::gfm();
    options.mdx = true;
    options
}

fn repeat_to(unit: &str, bytes: usize) -> String {
    let mut out = String::with_capacity(bytes + unit.len());
    while out.len() < bytes {
        out.push_str(unit);
    }
    out
}

fn tree(source: &str) -> String {
    let allocator = Allocator::new();
    let document = Parser::with_options(&allocator, source, mdx_options())
        .parse()
        .expect("source should parse");
    let mut out = String::new();
    pretty::format_document(&document, source, &mut out);
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
            let parsed = Parser::with_options(&allocator, &owned, mdx_options()).parse().is_ok();
            let _ = sender.send((parsed, started.elapsed()));
        });
        let (parsed, elapsed) =
            receiver.recv_timeout(BUDGET).expect("MDX braces should parse in bounded time");
        assert!(parsed, "MDX braces should parse to a document");
        best = best.min(elapsed);
    }
    best
}

fn assert_linear(name: &str, unit: &str) {
    let small = parse_within_budget(&repeat_to(unit, 32 * 1024));
    let large = parse_within_budget(&repeat_to(unit, 128 * 1024));

    let ratio = large.as_secs_f64() / small.as_secs_f64().max(1e-9);
    // 4x the input. Linear costs about 4x the time; quadratic costs 16x,
    // which is what every one of these measured before the fix.
    assert!(
        ratio < 8.0,
        "{name}: 128 KiB took {large:?} against {small:?} for 32 KiB (x{ratio:.1}); \
         linear is about x4, quadratic about x16"
    );
}

#[test]
fn inline_expressions_cost_linear_time() {
    // Closed expressions never needed a long scan to begin with; they were
    // quadratic purely because finding the `{` was.
    for (name, unit) in
        [("closed", "{a} "), ("closed, empty", "{} "), ("closed, after text", "a{b} ")]
    {
        assert_linear(name, unit);
    }
}

#[test]
fn unclosed_inline_braces_cost_linear_time() {
    for (name, unit) in
        [("bare", "{ "), ("no separator", "{"), ("with text", "{a "), ("nested", "{a{b ")]
    {
        assert_linear(name, unit);
    }
}

#[test]
fn unclosed_flow_braces_cost_linear_time() {
    // A line that starts with `{` asks the block parser whether a flow
    // expression begins there, and that check scanned the rest of the
    // *whole source* — once per line.
    for (name, unit) in [
        ("bare line", "{\n"),
        ("line with text", "{a\n"),
        ("open string", "{\"s\n"),
        ("open template", "{`t\n"),
        ("open comment", "{/*\n"),
        ("nested", "{a{b\n"),
    ] {
        assert_linear(name, unit);
    }
}

#[test]
fn braces_still_parse_to_the_same_nodes() {
    // The guards short-circuit scans that ended in a literal `{`, and the
    // marker cache changes only when the scan runs, not what it finds. Pin
    // both the expression nodes and the literal fallbacks, with spans.
    for (source, expected) in [
        ("{a}", "Document [0..3]\n  MdxFlowExpression value=\"a\" [0..3]\n"),
        ("{}", "Document [0..2]\n  MdxFlowExpression value=\"\" [0..2]\n"),
        ("{a{b}}", "Document [0..6]\n  MdxFlowExpression value=\"a{b}\" [0..6]\n"),
        (
            "x{y}z",
            "Document [0..5]\n  Paragraph [0..5]\n    Text \"x\" [0..1]\n    \
             MdxTextExpression value=\"y\" [1..4]\n    Text \"z\" [4..5]\n",
        ),
        ("{ ", "Document [0..2]\n  Paragraph [0..2]\n    Text \"{\" [0..1]\n"),
        ("{\n", "Document [0..2]\n  Paragraph [0..2]\n    Text \"{\" [0..1]\n"),
        (
            "{a",
            "Document [0..2]\n  Paragraph [0..2]\n    Text \"{\" [0..1]\n    Text \"a\" [1..2]\n",
        ),
        (
            "a { b",
            "Document [0..5]\n  Paragraph [0..5]\n    Text \"a \" [0..2]\n    \
             Text \"{\" [2..3]\n    Text \" b\" [3..5]\n",
        ),
    ] {
        assert_eq!(tree(source), expected, "for {source:?}");
    }
}
