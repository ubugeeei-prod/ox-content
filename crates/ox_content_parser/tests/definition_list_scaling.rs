//! A definition item is a run of term lines followed by a `: body` line, so
//! deciding whether one starts on a line means finding where the run that
//! line belongs to stops. The parser asked that at every block start and for
//! every line of a definition body, and each time it walked the rest of the
//! run, collecting the lines into the arena as it went. A run with no body
//! after it therefore cost O(n²) time and O(n²) memory: 16,000 `=` lines
//! (31 KB) took 2.4 s and 1.8 GB. Nothing unusual is needed for that, only
//! lines that could be terms and several blocks among them — setext
//! headings, or a definition whose text is wrapped without indentation.
//!
//! These tests pin the cost back to linear, in time and in arena bytes, and
//! pin the parse of the shapes whose answer now comes from a run that was
//! already walked.

use std::sync::mpsc;
use std::thread;
use std::time::{Duration, Instant};

use ox_content_allocator::Allocator;
use ox_content_parser::{Parser, ParserOptions};
use ox_content_renderer::HtmlRenderer;

#[path = "support/pretty.rs"]
mod pretty;

/// Generous enough that a slow shared runner never trips it. The comparisons
/// below are what catch a quadratic parse; this only keeps one that stopped
/// finishing from hanging the suite.
const BUDGET: Duration = Duration::from_secs(30);

/// Lines in the smaller input of each comparison; the larger has four times
/// as many.
///
/// Small on purpose. The linear parse of the larger input takes a
/// millisecond or two in a debug build, short enough that a busy runner
/// rarely interrupts it and skews the ratio. The quadratic one took half a
/// second or more and 5 to 11 MB for the same input, which fails the
/// comparison at once instead of running into the budget with gigabytes of
/// arena behind it.
const LINES: usize = 250;

fn options() -> ParserOptions {
    ParserOptions { definition_lists: true, ..ParserOptions::default() }
}

fn render(source: &str) -> String {
    let allocator = Allocator::new();
    let document =
        Parser::with_options(&allocator, source, options()).parse().expect("source should parse");
    HtmlRenderer::new().render(&document).trim().to_string()
}

fn tree(source: &str) -> String {
    let allocator = Allocator::new();
    let document =
        Parser::with_options(&allocator, source, options()).parse().expect("source should parse");
    let mut out = String::new();
    pretty::format_document(&document, source, &mut out);
    out
}

/// Inputs made of `lines` candidate term lines, one per way the parser comes
/// to ask whether a definition item starts inside a run of them.
fn shapes(lines: usize) -> [(&'static str, String); 8] {
    [
        // Each pair is a setext heading, so every other line starts a block.
        ("a run of `=` lines", "=\n".repeat(lines)),
        ("a run of `--` lines", "--\n".repeat(lines)),
        ("text over `=` underlines", "a\n=\n".repeat(lines / 2)),
        // A definition body asks about each line it might continue onto.
        ("a body wrapped without indent", format!("a\n: b\n{}", "c\n".repeat(lines))),
        ("the same after an indented line", format!("a\n: b\n    c\n{}", "d\n".repeat(lines))),
        // The verdict for a run includes what follows the blank lines after
        // it, so those must not be re-read for every line of the run either.
        ("a run before blank lines", format!("{}{}x\n", "=\n".repeat(lines), "\n".repeat(lines))),
        // The shapes that do build a list were linear already and have to
        // stay that way now that their terms are found by the same walk.
        ("many terms, one body", format!("{}: b\n", "a\n".repeat(lines))),
        ("many items", "a\n: b\n".repeat(lines / 2)),
    ]
}

struct Cost {
    time: Duration,
    arena: usize,
}

/// Parses on a worker thread so a regression fails the suite in bounded
/// time instead of hanging it.
fn parse_within_budget(name: &str, source: &str) -> Cost {
    let owned = source.to_string();
    let (sender, receiver) = mpsc::channel();
    thread::spawn(move || {
        let started = Instant::now();
        let allocator = Allocator::new();
        let parsed = Parser::with_options(&allocator, &owned, options()).parse().is_ok();
        let time = started.elapsed();
        // `allocated_bytes` counts whole chunks, and each chunk is twice the
        // size of the one before, so on its own it moves in steps of up to
        // 2x. Taking off the unused tail of the last chunk leaves what the
        // parse asked for.
        let arena = allocator.allocated_bytes() - allocator.bump().chunk_capacity();
        let _ = sender.send((parsed, Cost { time, arena }));
    });
    let (parsed, cost) = receiver
        .recv_timeout(BUDGET)
        .unwrap_or_else(|_| panic!("{name} should parse in bounded time"));
    assert!(parsed, "{name} should parse to a document");
    cost
}

/// Fastest of nine parses, so stalls on a busy runner cannot fail the build.
fn fastest_parse(name: &str, source: &str) -> Duration {
    (0..9).map(|_| parse_within_budget(name, source).time).min().unwrap_or(BUDGET)
}

#[test]
fn a_run_of_term_lines_costs_linear_time() {
    for ((name, small), (_, large)) in shapes(LINES).into_iter().zip(shapes(4 * LINES)) {
        let small = fastest_parse(name, &small);
        let large = fastest_parse(name, &large);

        let ratio = large.as_secs_f64() / small.as_secs_f64().max(1e-9);
        // 4x the input. Linear costs about 4x the time; quadratic costs
        // 16x or more, which is what the first six of these measured before
        // the run was remembered.
        assert!(
            ratio < 8.0,
            "{name}: {} lines took {large:?} against {small:?} for {LINES} (x{ratio:.1}); \
             linear is about x4, quadratic about x16",
            4 * LINES
        );
    }
}

#[test]
fn a_run_of_term_lines_costs_linear_arena() {
    for ((name, small), (_, large)) in shapes(LINES).into_iter().zip(shapes(4 * LINES)) {
        let small = parse_within_budget(name, &small).arena;
        let large = parse_within_budget(name, &large).arena;

        // The arena never frees, so whatever a probe allocates stays for
        // the life of the document. Unlike the timing this is exact: linear
        // is x4, and the probe that collected lines made it x15.
        assert!(
            large < 8 * small,
            "{name}: {} lines used {large} arena bytes against {small} for {LINES} (x{}); \
             linear is about x4, quadratic about x16",
            4 * LINES,
            large / small.max(1)
        );
    }
}

#[test]
fn a_run_without_a_body_stays_ordinary_text() {
    // The first block start walks the whole run; the later ones land inside
    // it and reuse that verdict, so each has to come out as it did when it
    // walked the run itself.
    for (source, expected) in [
        ("a\nb\nc", "<p>a\nb\nc</p>"),
        ("=\n=\n=\n=", "<h1 id=\"section\">=</h1>\n<h1 id=\"section-1\">=</h1>"),
        ("--\n--", "<h2 id=\"section\">--</h2>"),
        ("=\n=\nb\n\nc", "<h1 id=\"section\">=</h1>\n<p>b</p>\n<p>c</p>"),
        // Stopped by another block rather than by a blank line or the end.
        ("a\n# h\n: b", "<p>a</p>\n<h1 id=\"h\">h</h1>\n<p>: b</p>"),
        ("a\nb\n> q\n: c", "<p>a\nb</p>\n<blockquote>\n<p>q</p>\n</blockquote>\n<p>: c</p>"),
        // Four columns of indent make the `:` line text, not a body.
        ("a\n    : b", "<p>a\n: b</p>"),
        ("a `b\n: c", "<p>a `b</p>\n<p>: c</p>"),
    ] {
        assert_eq!(render(source), expected, "for {source:?}");
    }
}

#[test]
fn a_run_with_a_body_still_becomes_a_definition_item() {
    const OPEN: &str = "<dl class=\"ox-definition-list\">";
    for (source, expected) in [
        ("Term\n: Definition", "<dt>Term</dt>\n<dd>Definition</dd>"),
        ("A\nB\n: def", "<dt>A</dt>\n<dt>B</dt>\n<dd>def</dd>"),
        ("A\n\n\n: def", "<dt>A</dt>\n<dd>def</dd>"),
        ("a\r\nb\r\n: c\r\n", "<dt>a</dt>\n<dt>b</dt>\n<dd>c</dd>"),
        ("a\n  : b", "<dt>a</dt>\n<dd>b</dd>"),
        ("a\n: b\n: c", "<dt>a</dt>\n<dd>b</dd>\n<dd>c</dd>"),
        // A body claims the whole run above it, setext underlines included.
        ("a\n=\nb\n: c", "<dt>a</dt>\n<dt>=</dt>\n<dt>b</dt>\n<dd>c</dd>"),
        ("a\n=\n\n\n: b", "<dt>a</dt>\n<dt>=</dt>\n<dd>b</dd>"),
        // A line that is blank only to Unicode ends the run like any other.
        ("a\n\u{a0}\n: b", "<dt>a</dt>\n<dd>b</dd>"),
        // The body asks about `c`, and the list then asks again to collect
        // it: the second answer comes from the remembered run.
        ("a\n: b\nc\nd\n: e", "<dt>a</dt>\n<dd>b</dd>\n<dt>c</dt>\n<dt>d</dt>\n<dd>e</dd>"),
        (
            "a\n: b\n\n=\n=\nc\n: d",
            "<dt>a</dt>\n<dd>b</dd>\n<dt>=</dt>\n<dt>=</dt>\n<dt>c</dt>\n<dd>d</dd>",
        ),
        (
            "a\n: b\n\n    c\n\nd\n: e",
            "<dt>a</dt>\n<dd>\n<p>b</p>\n<p>c</p>\n</dd>\n<dt>d</dt>\n<dd>e</dd>",
        ),
    ] {
        assert_eq!(render(source), format!("{OPEN}\n{expected}\n</dl>"), "for {source:?}");
    }
}

#[test]
fn a_remembered_run_does_not_leak_past_its_end() {
    for (source, expected) in [
        // The first run has no body; the one after the blank line does.
        (
            "a\nb\n\nc\n: d",
            "<p>a\nb</p>\n<dl class=\"ox-definition-list\">\n<dt>c</dt>\n<dd>d</dd>\n</dl>",
        ),
        // And the other way round.
        (
            "a\n: b\n\nc\nd",
            "<dl class=\"ox-definition-list\">\n<dt>a</dt>\n<dd>b</dd>\n</dl>\n<p>c\nd</p>",
        ),
        // Unindented lines after a body continue it while their run has no
        // body of its own; the blank line ends both.
        (
            "a\n: b\nc\nd\n\ne",
            "<dl class=\"ox-definition-list\">\n<dt>a</dt>\n<dd>b\nc\nd</dd>\n</dl>\n<p>e</p>",
        ),
        (
            "a\n: b\nc\n=\n\nd",
            "<dl class=\"ox-definition-list\">\n<dt>a</dt>\n<dd>b\nc\n=</dd>\n</dl>\n<p>d</p>",
        ),
    ] {
        assert_eq!(render(source), expected, "for {source:?}");
    }
}

#[test]
fn a_list_marker_followed_by_a_tab_still_ends_the_run() {
    // Term lines are told from list items without building the item now,
    // and a tab after the marker is the case where building one copied its
    // text into the arena.
    for (source, expected) in [
        ("a\n-\tb\n: c", "<p>a</p>\n<ul>\n<li>b</li>\n</ul>\n<p>: c</p>"),
        ("a\n2.\tb\n: c", "<p>a\n2.\tb</p>\n<p>: c</p>"),
        (
            "a\n: b\n2.\tx\nc",
            "<dl class=\"ox-definition-list\">\n<dt>a</dt>\n<dd>b\n2.\tx\nc</dd>\n</dl>",
        ),
    ] {
        assert_eq!(render(source), expected, "for {source:?}");
    }
}

#[test]
fn term_lines_keep_their_spans() {
    // HTML alone would not catch a term whose span moved: each term covers
    // its own line without the terminator, and the list runs from the first
    // term to the end of the last body.
    assert_eq!(
        tree("a\n b \r\n\n: c\n"),
        "Document [0..12]\n  DefinitionList [0..12]\n    DefinitionListTerm [0..1]\n      \
         Text \"a\" [0..1]\n    DefinitionListTerm [2..5]\n      Text \"b\" [3..4]\n    \
         DefinitionListDefinition [8..12]\n      Paragraph [10..12]\n        \
         Text \"c\" [10..11]\n"
    );
}
