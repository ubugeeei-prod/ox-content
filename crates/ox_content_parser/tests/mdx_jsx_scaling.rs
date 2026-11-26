//! Unclosed and mismatched MDX JSX used to cost O(n²), and O(n³) once an
//! attribute expression was involved.
//!
//! The search for a closing tag only reports that there is none after
//! walking to the end of its input. The tag then stays literal text, the
//! parser moves on to the next opener, and that one walks the same bytes
//! again. Properly nested elements paid the same way, one walk per level.
//! The brace, string and comment skips inside those walks fail the same
//! way, so `<A b={` in front of each opener multiplied the cost again.
//!
//! Measured before the fix, release build: 125 KiB of `<A>` took 20.7 s,
//! of `<>` 10.3 s, of `<A {` lines 3.4 s, and 8 KiB of `<A b={<A>` took
//! 3.8 s. Each now parses in a few milliseconds.
//!
//! These tests pin the cost to linear, and pin the trees those inputs parse
//! to: the scans are remembered now, not changed.

use std::sync::mpsc;
use std::thread;
use std::time::{Duration, Instant};

use ox_content_allocator::Allocator;
use ox_content_parser::{Parser, ParserOptions};

#[path = "support/pretty.rs"]
mod pretty;

/// Generous enough that a slow shared runner never trips it, and far below
/// what the quadratic path needed for these sizes.
const BUDGET: Duration = Duration::from_secs(30);

/// A 128 KiB parse this quick is off the quadratic path whatever its ratio
/// says: unoptimized, the old walks needed minutes for these inputs. The
/// ratio alone is too noisy on a loaded runner for the shapes that build
/// tens of thousands of nodes.
const NOISE: Duration = Duration::from_secs(2);

fn repeat_to(unit: &str, bytes: usize) -> String {
    let mut out = String::with_capacity(bytes + unit.len());
    while out.len() < bytes {
        out.push_str(unit);
    }
    out
}

fn tree(source: &str) -> String {
    let allocator = Allocator::new();
    let document = Parser::with_options(&allocator, source, ParserOptions::mdx())
        .parse()
        .expect("source should parse");
    let mut out = String::new();
    pretty::format_document(&document, source, &mut out);
    out
}

/// Parses on a worker thread so a regression fails the suite in bounded
/// time instead of hanging it. Best of two, so one scheduling stall on a
/// busy runner cannot fail the build.
///
/// The nesting limit turns some of these inputs into an error; that is a
/// finished parse too, and how long it took is what is measured.
fn parse_within_budget(source: &str) -> Duration {
    let mut best = BUDGET;
    for _ in 0..2 {
        let owned = source.to_string();
        let (sender, receiver) = mpsc::channel();
        thread::spawn(move || {
            let started = Instant::now();
            let allocator = Allocator::new();
            let _ = Parser::with_options(&allocator, &owned, ParserOptions::mdx()).parse();
            let _ = sender.send(started.elapsed());
        });
        let elapsed = receiver.recv_timeout(BUDGET).expect("MDX JSX should parse in bounded time");
        best = best.min(elapsed);
    }
    best
}

fn assert_linear(name: &str, build: impl Fn(usize) -> String) {
    let small = parse_within_budget(&build(32 * 1024));
    let large = parse_within_budget(&build(128 * 1024));

    let ratio = large.as_secs_f64() / small.as_secs_f64().max(1e-9);
    // 4x the input. Linear costs about 4x the time; quadratic costs 16x,
    // which is the least any of these measured before the fix.
    assert!(
        ratio < 8.0 || large < NOISE,
        "{name}: 128 KiB took {large:?} against {small:?} for 32 KiB (x{ratio:.1}); \
         linear is about x4, quadratic about x16"
    );
}

fn assert_units_linear(units: &[&str]) {
    for unit in units {
        // At the start of a line, where a flow element is tried first, and
        // inside a paragraph, where only a text element can start.
        assert_linear(&format!("{unit:?} lines"), |bytes| repeat_to(&format!("{unit}\n"), bytes));
        assert_linear(&format!("{unit:?} in text"), |bytes| {
            format!("a {}", repeat_to(unit, bytes))
        });
    }
}

#[test]
fn unclosed_tags_cost_linear_time() {
    assert_units_linear(&["<A>", "<>", "<A><B>", "<A></B>", "<A.b>", "<A> ", "<A>x"]);
}

#[test]
fn unclosed_attributes_cost_linear_time() {
    assert_units_linear(&[
        "<A {",
        "<A {...",
        "<A b={",
        "<A b={{",
        "<A b={\"}",
        "<A b={/*",
        "<A b={`",
        "<A b={<A>",
        "<A b=\"x\" c={",
    ]);
}

#[test]
fn unclosed_braces_around_tags_cost_linear_time() {
    assert_units_linear(&["{<A>", "<A>{", "<A>{\"", "<A>{/*", "<A>{//", "<A>`", "<A>{`"]);
}

#[test]
fn openers_inside_one_element_cost_linear_time() {
    // The element does close, far away, so every brace and tag inside it
    // is stepped over by the search for that closing tag.
    for unit in ["<A>", "<>", "<A {", "<A b={<A>", "{", "{{", "{\"", "{/*", "{//", "{`"] {
        assert_linear(&format!("{unit:?} in an element"), |bytes| {
            format!("a <Z>{}</Z>", repeat_to(unit, bytes))
        });
    }
}

#[test]
fn closers_that_match_nothing_cost_linear_time() {
    // A closer at the far end is enough to defeat a check for "no closer
    // at all": every opener still has to learn that it is not the match.
    for (open, close) in [
        ("<A>", "</B>"),
        ("<A.b>", "</A>"),
        ("<A {\n", "}"),
        ("<A b={", "}"),
        ("<A>{", "}"),
        ("{<A>", "}"),
        ("{", "}"),
        ("{\"", "\"}"),
        ("{/*", "*/}"),
    ] {
        let build = |bytes: usize, closers: usize| {
            let count = bytes / (open.len() + close.len());
            format!("{}{}", open.repeat(count), close.repeat(count / closers))
        };
        assert_linear(&format!("{open:?} then {close:?}"), |bytes| build(bytes, 1));
        assert_linear(&format!("{open:?} then half as many {close:?}"), |bytes| build(bytes, 2));
    }
}

#[test]
fn nested_elements_cost_linear_time() {
    // One element per line nests as flow elements, which share the walk
    // that closed the outermost one until the nesting limit stops them.
    for (open, close) in [("<A>\n", "</A>\n"), ("<>\n", "</>\n"), ("<A><B>\n", "</B></A>\n")] {
        assert_linear(&format!("{open:?} nested"), |bytes| {
            let count = bytes / (open.len() + close.len());
            format!("{}{}", open.repeat(count), close.repeat(count))
        });
    }
}

#[test]
fn lines_that_open_a_brace_inside_a_paragraph_cost_linear_time() {
    // Each such line asks whether a flow expression starts there, which
    // interrupts the paragraph. The closing brace at the end keeps the
    // cheap "no `}` follows" check from answering.
    assert_linear("brace lines", |bytes| format!("a\n{}}}", repeat_to("{\n", bytes)));
}

#[test]
fn unclosed_jsx_keeps_its_literal_shape() {
    // Every one of these ends in a search that finds nothing. The opener
    // has to stay exactly the text or HTML it was.
    for (source, expected) in [
        (
            "a <A> b",
            "Document [0..7]\n  Paragraph [0..7]\n    Text \"a \" [0..2]\n    \
             Html value=\"<A>\" [2..5]\n    Text \" b\" [5..7]\n",
        ),
        (
            "a <> b",
            "Document [0..6]\n  Paragraph [0..6]\n    Text \"a \" [0..2]\n    \
             Text \"<\" [2..3]\n    Text \"> b\" [3..6]\n",
        ),
        (
            "a <A></B>",
            "Document [0..9]\n  Paragraph [0..9]\n    Text \"a \" [0..2]\n    \
             Html value=\"<A>\" [2..5]\n    Html value=\"</B>\" [5..9]\n",
        ),
        (
            "a <A b={<A>",
            "Document [0..11]\n  Paragraph [0..11]\n    Text \"a \" [0..2]\n    \
             Text \"<\" [2..3]\n    Text \"A b=\" [3..7]\n    Text \"{\" [7..8]\n    \
             Html value=\"<A>\" [8..11]\n",
        ),
        (
            "<A {\n",
            "Document [0..5]\n  Paragraph [0..5]\n    Text \"<\" [0..1]\n    \
             Text \"A \" [1..3]\n    Text \"{\" [3..4]\n",
        ),
    ] {
        assert_eq!(tree(source), expected, "for {source:?}");
    }
}

#[test]
fn closed_jsx_still_parses_to_elements() {
    for (source, expected) in [
        (
            "a <A>b</A> c",
            "Document [0..12]\n  Paragraph [0..12]\n    Text \"a \" [0..2]\n    \
             MdxJsxTextElement name=Some(\"A\") self_closing=false [2..10]\n      \
             Text \"b\" [5..6]\n    Text \" c\" [10..12]\n",
        ),
        (
            // The inner element closes; the outer opener has no closer left.
            "a <A><A>b</A>",
            "Document [0..13]\n  Paragraph [0..13]\n    Text \"a \" [0..2]\n    \
             Html value=\"<A>\" [2..5]\n    \
             MdxJsxTextElement name=Some(\"A\") self_closing=false [5..13]\n      \
             Text \"b\" [8..9]\n",
        ),
        (
            // A closing tag inside a brace group does not close the element.
            "a <A>{</A>}</A>",
            "Document [0..15]\n  Paragraph [0..15]\n    Text \"a \" [0..2]\n    \
             MdxJsxTextElement name=Some(\"A\") self_closing=false [2..15]\n      \
             MdxTextExpression value=\"</A>\" [5..11]\n",
        ),
        (
            "<A>\n<A>\nb\n</A>\n</A>\n",
            "Document [0..20]\n  MdxJsxFlowElement name=Some(\"A\") self_closing=false [0..20]\n    \
             MdxJsxFlowElement name=Some(\"A\") self_closing=false [4..15]\n      \
             Paragraph [8..10]\n        Text \"b\" [8..9]\n",
        ),
    ] {
        assert_eq!(tree(source), expected, "for {source:?}");
    }
}

#[test]
fn long_blocks_parse_to_the_same_nodes_as_short_ones() {
    // Short blocks are scanned directly and long ones through the cache.
    // The same constructs with a long tail in the same paragraph have to
    // come out as the same nodes at the same offsets.
    let tail = "t".repeat(400);
    for source in [
        "a <A>b</A> <A>c <B>d</B></A> <A> e <B>f</A></B> {g} {h <A i={j} k=\"l\">m</A>",
        "a <A><A><A>b</A></A> <A.c>d</A.c> <>e</> <A f={<A>}>g</A> <A {",
        "a <A>{</A>}</A> <A>`</A>`</A> <A>{\"}</A>\"}</A> {{b}{c} {\"d",
    ] {
        let short = tree(source);
        let long = tree(&format!("{source} x\n{tail}"));
        let nodes =
            |tree: &str| -> Vec<String> { tree.lines().skip(2).map(str::to_string).collect() };
        let (short, long) = (nodes(&short), nodes(&long));
        // Everything before the last text node, which the tail extends.
        assert_eq!(long[..short.len() - 1], short[..short.len() - 1], "for {source:?}");
        assert!(short.len() > 6, "{source:?} should parse to several nodes");
    }
}
