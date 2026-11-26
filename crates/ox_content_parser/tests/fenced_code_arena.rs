//! A fenced code block whose opening fence is indented strips that
//! indentation from its lines, so its body is built in the arena rather than
//! borrowed from the source. The buffer for it used to be reserved up front as
//! "whatever source is left, up to 8 KiB" — per block, however little the
//! block went on to hold. 9,600 repetitions of `x` and an indented fence
//! (67 KB) are 4,800 two-byte code blocks, and took 38 MB of arena.
//!
//! The body is now measured before it is copied. These tests pin the arena to
//! what the blocks hold; `output.rs` pins what indented fences render to,
//! since the pass that finds the closing fence was split from the pass that
//! copies the lines.

use std::sync::mpsc;
use std::thread;
use std::time::{Duration, Instant};

use ox_content_allocator::Allocator;
use ox_content_parser::{Parser, ParserOptions};
use ox_content_renderer::HtmlRenderer;

#[path = "fenced_code_arena/output.rs"]
mod output;
#[path = "support/pretty.rs"]
mod pretty;

/// Generous enough that a slow shared runner never trips it.
const BUDGET: Duration = Duration::from_secs(30);

fn render(source: &str) -> String {
    let allocator = Allocator::new();
    let document = Parser::with_options(&allocator, source, ParserOptions::default())
        .parse()
        .expect("source should parse");
    HtmlRenderer::new().render(&document).trim().to_string()
}

fn tree(source: &str) -> String {
    let allocator = Allocator::new();
    let document = Parser::with_options(&allocator, source, ParserOptions::default())
        .parse()
        .expect("source should parse");
    let mut out = String::new();
    pretty::format_document(&document, source, &mut out);
    out
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
fn parse_within_budget(source: &str) -> Cost {
    let mut best = Cost { time: BUDGET, arena: usize::MAX };
    for _ in 0..3 {
        let owned = source.to_string();
        let (sender, receiver) = mpsc::channel();
        thread::spawn(move || {
            let started = Instant::now();
            let allocator = Allocator::new();
            let parsed = Parser::new(&allocator, &owned).parse().is_ok();
            let time = started.elapsed();
            let arena = allocator.allocated_bytes() - allocator.bump().chunk_capacity();
            let _ = sender.send((parsed, time, arena));
        });
        let (parsed, time, arena) = receiver
            .recv_timeout(BUDGET)
            .expect("a run of fenced code blocks should parse in bounded time");
        assert!(parsed, "a run of fenced code blocks should parse to a document");
        best.time = best.time.min(time);
        best.arena = best.arena.min(arena);
    }
    best
}

fn assert_linear(unit: &str) -> (Cost, usize) {
    let small_source = unit.repeat(2_400);
    let large_source = unit.repeat(9_600);
    let small = parse_within_budget(&small_source);
    let large = parse_within_budget(&large_source);

    let ratio = large.time.as_secs_f64() / small.time.as_secs_f64().max(1e-9);
    assert!(
        ratio < 8.0,
        "{unit:?}: 9,600 repetitions took {:?} against {:?} for 2,400 (x{ratio:.1}); \
         linear is about x4",
        large.time,
        small.time
    );
    assert!(
        large.arena < 6 * small.arena,
        "{unit:?}: 9,600 repetitions used {} arena bytes against {} for 2,400; \
         linear is about x4",
        large.arena,
        small.arena
    );
    (large, large_source.len())
}

#[test]
fn a_run_of_indented_fences_uses_arena_for_what_the_blocks_hold() {
    for unit in [
        // `x`, then a fence that the next repetition's fence closes.
        "x\n ```\n",
        "x\n  ```\n",
        "x\n   ```\n",
        // Empty blocks, opened and closed in one repetition or two.
        "x\n ```\n ```\n",
        "   ~~~\n",
        " ~~~\n",
        // Blocks with a body, stripped and not, and with CRLF line endings.
        " ```\nabc\n ```\n",
        "  ```\n  abc\n  ```\n",
        "   ```\r\n   abc\r\n   ```\r\n",
        // Indented inside a block quote's content.
        ">  ```\n",
        // Not indented, so borrowed from the source: the cost to match.
        "x\n```\n```\n",
        "> ```\n",
    ] {
        let (large, source_len) = assert_linear(unit);
        // The nodes cost a couple of dozen bytes per source byte on blocks
        // this small; the reservations cost four to eight hundred.
        assert!(
            large.arena < 48 * source_len,
            "{unit:?}: {} arena bytes for a {source_len} byte document",
            large.arena
        );
    }
}

#[test]
fn fences_inside_list_items_cost_linear_arena() {
    // Each of these is a list item holding one unclosed fence. The item's
    // own bookkeeping dominates both costs, so only the arena's growth is
    // pinned here.
    for unit in ["- ```\n", "-  ```\n", "1.  ~~~\n"] {
        let small = parse_within_budget(&unit.repeat(2_400));
        let large = parse_within_budget(&unit.repeat(9_600));
        assert!(
            large.arena < 6 * small.arena,
            "{unit:?}: 9,600 repetitions used {} arena bytes against {} for 2,400",
            large.arena,
            small.arena
        );
    }
}

#[test]
fn an_indented_fence_costs_its_body_and_nothing_else() {
    // The same document twice, except that one fence is indented by a space
    // (and the other closes with two more backticks, to keep the length and
    // every length-derived reservation equal). The unindented body is
    // borrowed, the indented one is copied, so the two parses may differ by
    // that four-byte copy and its alignment — not by a buffer sized from the
    // 20 KB of prose that follows.
    let prose = "word ".repeat(4_000);
    let indented = format!(" ```\nabc\n ```\n{prose}");
    let plain = format!("```\nabc\n`````\n{prose}");
    assert_eq!(indented.len(), plain.len());
    assert_eq!(render(&indented), render(&plain));

    let used = |source: &str| {
        // One chunk holds the whole parse, so the difference is exact.
        let allocator = Allocator::with_capacity(64 * source.len());
        let parsed = Parser::new(&allocator, source).parse().is_ok();
        assert!(parsed, "source should parse");
        allocator.allocated_bytes() - allocator.bump().chunk_capacity()
    };
    let extra = used(&indented).saturating_sub(used(&plain));
    assert!(extra <= 16, "an indented four-byte code block cost {extra} extra arena bytes");
}

#[test]
fn an_unclosed_indented_fence_copies_the_rest_of_the_document_once() {
    for (opener, line) in [(" ```\n", "a\n"), ("  ~~~\n", "  some code\n"), ("   ```\n", "\n")] {
        let small_source = opener.to_string() + &line.repeat(8_000);
        let large_source = opener.to_string() + &line.repeat(32_000);
        let small = parse_within_budget(&small_source);
        let large = parse_within_budget(&large_source);

        let ratio = large.time.as_secs_f64() / small.time.as_secs_f64().max(1e-9);
        assert!(
            ratio < 8.0,
            "{opener:?} + {line:?}: 32,000 lines took {:?} against {:?} for 8,000 (x{ratio:.1})",
            large.time,
            small.time
        );
        // One copy of the body plus the document's own fixed overhead.
        assert!(
            large.arena < 3 * large_source.len(),
            "{opener:?} + {line:?}: {} arena bytes for a {} byte document",
            large.arena,
            large_source.len()
        );
    }
}

#[test]
fn a_long_indented_block_keeps_every_line() {
    // The body is sized by one pass and filled by another; the two have to
    // agree on every line, including the last one's missing line ending.
    let mut source = String::from("  ```text\n");
    let mut expected = String::new();
    for index in 0..2_000 {
        let line = format!("{}line {index}", " ".repeat(index % 5));
        source.push_str(&line);
        source.push_str(if index % 3 == 0 { "\r\n" } else { "\n" });
        // Up to two columns of indentation go; anything more stays.
        expected.push_str(&line[(index % 5).min(2)..]);
        expected.push('\n');
    }
    let closed = format!("{source}  ```\n");
    assert_eq!(
        render(&closed),
        format!("<pre><code class=\"language-text\">{expected}</code></pre>")
    );
    // Unclosed, the block runs to the end of input, and a last line without
    // a line ending contributes none.
    assert_eq!(
        render(source.trim_end()),
        format!("<pre><code class=\"language-text\">{}</code></pre>", expected.trim_end())
    );
}
