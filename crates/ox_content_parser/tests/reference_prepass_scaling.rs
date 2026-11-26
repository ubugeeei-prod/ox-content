//! The pre-pass that collects link reference definitions parses each
//! candidate line together with the lines after it, up to the next blank
//! line. It used to copy that whole chunk into the arena for every candidate.
//! Definitions separated by a heading, a thematic break, or a code fence have
//! no blank line between them, so each one copied the rest of the document:
//! 4,000 repetitions of `[x]: /u` and `***` (48 KB) took half a second and
//! 594 MB of arena, x16 for every x4 of input.
//!
//! These tests pin the cost back to linear in time and in arena. The chunk is
//! now found once and shared by every candidate inside it, so each of them
//! still has to see exactly the text it would have been handed on its own:
//! `output.rs` pins what the documents around the change render to.

use std::fmt::Write as _;
use std::sync::mpsc;
use std::thread;
use std::time::{Duration, Instant};

use ox_content_allocator::Allocator;
use ox_content_parser::{Parser, ParserOptions};
use ox_content_renderer::HtmlRenderer;

#[path = "reference_prepass_scaling/output.rs"]
mod output;

/// Generous enough that a slow shared runner never trips it, and far below
/// what the quadratic path needed for these sizes.
const BUDGET: Duration = Duration::from_secs(30);

fn render_with(source: &str, options: ParserOptions) -> String {
    let allocator = Allocator::new();
    let document =
        Parser::with_options(&allocator, source, options).parse().expect("source should parse");
    HtmlRenderer::new().render(&document).trim().to_string()
}

fn render(source: &str) -> String {
    render_with(source, ParserOptions::default())
}

struct Cost {
    time: Duration,
    /// Arena bytes the parse used. `allocated_bytes` alone counts whole
    /// chunks, and chunks double, so a perfectly linear parse of a x4 input
    /// reads about x7 there; what is left of the last chunk is taken off.
    arena: usize,
}

/// Parses on a worker thread so a regression fails the suite in bounded
/// time instead of hanging it. Best of three, so one scheduling stall on a
/// busy runner cannot fail the build.
fn parse_within_budget(source: &str, options: &ParserOptions) -> Cost {
    let mut best = Cost { time: BUDGET, arena: usize::MAX };
    for _ in 0..3 {
        let owned = source.to_string();
        let options = options.clone();
        let (sender, receiver) = mpsc::channel();
        thread::spawn(move || {
            let started = Instant::now();
            let allocator = Allocator::new();
            let parsed = Parser::with_options(&allocator, &owned, options).parse().is_ok();
            let time = started.elapsed();
            let arena = allocator.allocated_bytes() - allocator.bump().chunk_capacity();
            let _ = sender.send((parsed, time, arena));
        });
        let (parsed, time, arena) = receiver
            .recv_timeout(BUDGET)
            .expect("a run of reference definitions should parse in bounded time");
        assert!(parsed, "a run of reference definitions should parse to a document");
        best.time = best.time.min(time);
        best.arena = best.arena.min(arena);
    }
    best
}

/// One definition and the block that separates it from the next one. Nothing
/// here is a blank line, so a whole document of any of these is one chunk.
const SEPARATED_DEFINITIONS: &[&str] = &[
    "[x]: /u\n***\n",
    "[x]: /u\n# h\n",
    "[x]: /u\n```\n```\n",
    "[x]: /u\nh\n---\n",
    "[x]: /u\nh\n===\n",
    "[x]: /u\n  'title'\n***\n",
    // A candidate that is not a definition, after one that is.
    "[x]: /u\n[y]\n***\n",
    // Quote markers have to be stripped, so these chunks are copied — once.
    "> [x]: /u\n> ***\n",
    "> [x]: /u\n***\n",
    "[x]: /u\n> ***\n",
];

/// The unit as written, with CRLF line endings, and with every line indented
/// by one, two, and three spaces.
fn variants(unit: &str) -> Vec<String> {
    let mut all = vec![unit.to_string(), unit.replace('\n', "\r\n")];
    for width in 1..=3 {
        let mut indented = String::new();
        for line in unit.split_inclusive('\n') {
            indented.push_str(&" ".repeat(width));
            indented.push_str(line);
        }
        all.push(indented);
    }
    all
}

fn assert_linear(unit: &str, options: &ParserOptions) {
    // Small enough that the per-candidate copy fails on the ratio below
    // rather than on the time budget, even in an unoptimized build.
    let small_source = unit.repeat(500);
    let large_source = unit.repeat(2_000);
    let small = parse_within_budget(&small_source, options);
    let large = parse_within_budget(&large_source, options);

    // 4x the input. Linear costs about 4x the time; the per-candidate copy
    // cost 16x or more, for every one of these.
    let ratio = large.time.as_secs_f64() / small.time.as_secs_f64().max(1e-9);
    assert!(
        ratio < 8.0,
        "{unit:?}: 2,000 repetitions took {:?} against {:?} for 500 (x{ratio:.1}); \
         linear is about x4, quadratic about x16",
        large.time,
        small.time
    );
    assert!(
        large.arena < 6 * small.arena,
        "{unit:?}: 2,000 repetitions used {} arena bytes against {} for 500; \
         linear is about x4, quadratic about x16",
        large.arena,
        small.arena
    );
    // The tree itself costs a few dozen bytes per source byte on documents
    // this dense in blocks; the copies cost thousands.
    assert!(
        large.arena < 64 * large_source.len(),
        "{unit:?}: {} arena bytes for a {} byte document",
        large.arena,
        large_source.len()
    );
}

#[test]
fn definitions_separated_by_other_blocks_cost_linear_time_and_arena() {
    for unit in SEPARATED_DEFINITIONS {
        for variant in variants(unit) {
            assert_linear(&variant, &ParserOptions::default());
        }
    }
}

#[test]
fn footnote_definitions_share_the_linear_pre_pass() {
    // Footnote labels are collected by the same pass, and a `[^x]:` line is a
    // definition candidate the reference side has to look at and reject.
    for unit in ["[^x]: a\n***\n", "[x]: /u\n[^y]: a\n***\n", "[x]: /u\n***\n"] {
        for variant in variants(unit) {
            assert_linear(&variant, &ParserOptions::gfm());
        }
    }
}

#[test]
fn a_chunk_that_needs_no_stripping_is_not_copied() {
    // One candidate, then a paragraph with no blank line: the whole document
    // is its chunk. With `\n` line endings and no quote markers the source
    // is already the text to parse, so the arena holds little besides the
    // paragraph's nodes — less than the document itself, where the copy and
    // its line table took fifteen times that.
    let source = "[x]: /u\n".to_string() + &"a\n".repeat(16_000);
    let cost = parse_within_budget(&source, &ParserOptions::default());
    assert!(
        cost.arena < 2 * source.len(),
        "{} arena bytes for a {} byte document",
        cost.arena,
        source.len()
    );
}

#[test]
fn every_definition_in_a_long_separated_run_still_resolves() {
    // Each definition is read out of the shared chunk at its own offset, so a
    // slip in that bookkeeping shows up as a wrong or missing destination.
    for (prefix, line_ending) in [("", "\n"), ("", "\r\n"), ("> ", "\n"), ("  ", "\r")] {
        let mut source = String::new();
        for index in 0..500 {
            write!(source, "{prefix}[r{index}]: /u{index}{line_ending}{prefix}***{line_ending}")
                .expect("writing to a String cannot fail");
        }
        source.push_str(line_ending);
        source.push_str("[a][r0] [b][r1] [c][r250] [d][r499] [e][r500]");
        let html = render(&source);
        assert!(
            html.ends_with(
                "<p><a href=\"/u0\">a</a> <a href=\"/u1\">b</a> <a href=\"/u250\">c</a> \
                 <a href=\"/u499\">d</a> [e][r500]</p>"
            ),
            "for prefix {prefix:?} and line ending {line_ending:?}: {}",
            &html[html.len().saturating_sub(160)..]
        );
    }
}
