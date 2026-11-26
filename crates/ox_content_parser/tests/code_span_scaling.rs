//! A backtick run opens a code span only when a run of the same length
//! follows, and an opener with none stays literal after a search to the end
//! of the content. Runs of many lengths, none of them closed — `` ` ``,
//! ``` `` ```, ```` ``` ```` and so on — paid that search each, and so did
//! every scan that steps over code spans (link text, math, wiki links,
//! superscript). That is O(n^1.5) rather than O(n²), since n bytes hold
//! about √(2n) distinct lengths: 1 MiB took 0.6 to 1.3 s, against 2 ms now.
//!
//! The growth is too slow for a 4x size ratio to tell apart from noise, so
//! each shape is timed against a control of the same size and structure
//! whose code spans all close.

use std::time::{Duration, Instant};

use ox_content_allocator::Allocator;
use ox_content_parser::{Parser, ParserOptions};
use ox_content_renderer::HtmlRenderer;

const BYTES: usize = 512 * 1024;

fn options() -> ParserOptions {
    ParserOptions { math: true, superscript: true, wiki_links: true, ..ParserOptions::gfm() }
}

/// `prefix`, then runs of one, two, three... backticks, each between
/// `before` and a space.
fn ramp(prefix: &str, before: &str) -> String {
    let mut out = String::from(prefix);
    let mut run = 1;
    while out.len() < BYTES {
        out.push_str(before);
        out.extend(std::iter::repeat_n('`', run));
        out.push(' ');
        run += 1;
    }
    out
}

/// The same shape, with every code span closed.
fn control(prefix: &str, before: &str) -> String {
    let mut out = String::from(prefix);
    while out.len() < BYTES {
        out.push_str(before);
        out.push_str("`x` ");
    }
    out
}

fn best_parse(source: &str) -> Duration {
    let mut arena = Allocator::new();
    let mut best = Duration::MAX;
    for _ in 0..3 {
        arena.reset();
        let started = Instant::now();
        let parsed = Parser::with_options(&arena, source, options()).parse().is_ok();
        best = best.min(started.elapsed());
        assert!(parsed, "the input should parse");
    }
    best
}

#[test]
fn backtick_runs_of_many_lengths_cost_about_what_closed_spans_do() {
    for (prefix, before) in [
        ("x ", ""),
        ("x ", "\\"),
        ("x [", ""),
        ("x ", "["),
        ("x [[", ""),
        ("x ", "$a "),
        ("x ^a ", ""),
    ] {
        let (ramp, control) = (ramp(prefix, before), control(prefix, before));
        let (ramp_time, control_time) = (best_parse(&ramp), best_parse(&control));
        assert!(
            ramp_time < control_time * 10 + Duration::from_millis(50),
            "{prefix:?} with {before:?} before each run: {ramp_time:?} against \
             {control_time:?} for closed spans"
        );
    }
}

fn render(source: &str) -> String {
    let allocator = Allocator::new();
    let document =
        Parser::with_options(&allocator, source, options()).parse().expect("source should parse");
    HtmlRenderer::new().render(&document).trim().to_string()
}

#[test]
fn code_spans_close_as_they_did() {
    // Each case is checked as written, and again behind enough text to take
    // the remembered search.
    let padding = "p ".repeat(200);
    for (source, expected) in [
        ("`a`", "<code>a</code>"),
        ("` `` `", "<code>``</code>"),
        ("`a ``b`", "<code>a ``b</code>"),
        ("`` a ` b ``", "<code>a ` b</code>"),
        ("`a", "`a"),
        ("` `` ``` `", "<code>`` ```</code>"),
        ("``` `` ` ``", "``` <code>`</code>"),
        ("` `` ` ``", "<code>``</code> ``"),
        ("\\`a`", "`a`"),
        ("\\``a`", "`<code>a</code>"),
        ("`a\nb`", "<code>a b</code>"),
        ("``a\r\nb``", "<code>a b</code>"),
        ("[`a](b)`", "[<code>a](b)</code>"),
        ("[a`](b)", "<a href=\"b\">a`</a>"),
        ("[[a`|b]]`", "[[a<code>|b]]</code>"),
        (
            "$a `$` b$",
            "<span class=\"ox-math ox-math-inline\" data-ox-tex=\"a `$` b\"><math><mtext>a `$` b</mtext></math></span>",
        ),
        ("^a `^` b^", "<sup>a <code>^</code> b</sup>"),
        ("^a ``` b^", "<sup>a ``` b</sup>"),
    ] {
        assert_eq!(render(source), format!("<p>{expected}</p>"), "for {source:?}");
        let long = format!("{padding}{source}");
        assert_eq!(render(&long), format!("<p>{padding}{expected}</p>"), "for {long:?}");
    }
}
