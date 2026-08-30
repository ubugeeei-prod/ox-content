//! Streaming has to answer "what is safe to commit" after every chunk.
//!
//! That answer used to be recomputed from the whole pending buffer each
//! time, which is quadratic exactly where nothing commits — a long fenced
//! code block arriving in pieces, which is what streamed output usually
//! looks like. The scan now resumes where it left off, so these tests
//! check that resuming lands on the same commits a rescan would, and that
//! the cost stops squaring.

use std::time::{Duration, Instant};

use ox_content_incremental::{IncrementalParser, stable_prefix_len};
use ox_content_parser::ParserOptions;

/// xorshift64 — deterministic, so a failure reproduces from its seed.
struct Rng(u64);

impl Rng {
    fn next(&mut self) -> u64 {
        let mut x = self.0;
        x ^= x << 13;
        x ^= x >> 7;
        x ^= x << 17;
        self.0 = x;
        x
    }

    fn below(&mut self, n: usize) -> usize {
        (self.next() % n as u64) as usize
    }
}

/// Lines chosen for the states the scan carries: fences that open, close,
/// nest by length, or never close; blank lines of every whitespace shape;
/// indented continuations; CRLF; and fragments with no line ending at all.
const LINES: &[&str] = &[
    "text\n",
    "\n",
    "  \n",
    "\t\n",
    "```\n",
    "```rust\n",
    "~~~\n",
    "~~~~\n",
    "``` \n",
    "  ```\n",
    "    indented\n",
    "  two space\n",
    "- item\n",
    "  - nested\n",
    "> quote\n",
    "# heading\n",
    "|a|b|\n",
    "\r\n",
    "text\r\n",
    "```\r\n",
    "partial",
    "a",
    "   ",
    "1. one\n",
    "\u{3042}\n",
    "    ```\n",
    "`````\n",
];

fn split(source: &str, size: usize) -> Vec<String> {
    let mut chunks = Vec::new();
    let mut at = 0;
    while at < source.len() {
        let mut end = (at + size).min(source.len());
        while !source.is_char_boundary(end) {
            end += 1;
        }
        chunks.push(source[at..end].to_string());
        at = end;
    }
    chunks
}

/// What the old append loop did: rescan the whole pending buffer per chunk.
fn rescan_commits(chunks: &[String]) -> Vec<String> {
    let mut pending = String::new();
    let mut commits = Vec::new();
    for chunk in chunks {
        pending.push_str(chunk);
        let stable = stable_prefix_len(&pending);
        if stable > 0 {
            commits.push(pending[..stable].to_string());
            pending.drain(..stable);
        }
    }
    if !pending.is_empty() {
        commits.push(pending);
    }
    commits
}

fn streamed_commits(chunks: &[String]) -> Vec<String> {
    let mut parser = IncrementalParser::new(ParserOptions::gfm());
    let mut commits = Vec::new();
    for chunk in chunks {
        let outcome = parser
            .append(chunk, false, |markdown, _, _| markdown.to_string())
            .expect("chunk should parse");
        if let Some(committed) = outcome.committed {
            commits.push(committed);
        }
    }
    let outcome = parser.finish(|markdown, _, _| markdown.to_string()).expect("tail should parse");
    if let Some(committed) = outcome.committed {
        commits.push(committed);
    }
    commits
}

#[test]
fn resuming_the_scan_commits_what_a_rescan_would() {
    for seed in 1..40_000u64 {
        let mut rng = Rng(seed.wrapping_mul(0x9E37_79B9_7F4A_7C15) | 1);
        let mut document = String::new();
        for _ in 0..1 + rng.below(14) {
            document.push_str(LINES[rng.below(LINES.len())]);
        }
        let size = 1 + rng.below(9);
        let chunks = split(&document, size);
        assert_eq!(
            streamed_commits(&chunks),
            rescan_commits(&chunks),
            "seed {seed}, chunk size {size}, document {document:?}"
        );
    }
}

#[test]
fn a_chunk_boundary_inside_a_fence_marker_still_opens_the_fence() {
    // The trailing partial line is scored but not committed to, so "``"
    // becoming "```" on the next chunk has to open a fence.
    let chunks = ["a\n\n``".to_string(), "`\n".to_string(), "b\n\nc\n".to_string()];
    assert_eq!(streamed_commits(&chunks), rescan_commits(&chunks));
    assert_eq!(streamed_commits(&chunks)[0], "a\n\n");
}

#[test]
fn blank_lines_inside_a_streamed_fence_never_commit() {
    let mut document = String::from("```\n");
    for index in 0..200 {
        document.push_str("line ");
        document.push_str(&index.to_string());
        document.push_str("\n\n");
    }
    let commits = streamed_commits(&split(&document, 7));
    assert_eq!(commits.len(), 1, "an unclosed fence commits only at finish: {commits:?}");
    assert_eq!(commits[0], document);
}

#[test]
fn streaming_a_long_fenced_block_costs_linear_time() {
    // Nothing commits until the fence closes, so the pending buffer grows
    // for the whole stream — the shape the rescan turned quadratic.
    let build = |kilobytes: usize| {
        let mut document = String::from("```rust\n");
        while document.len() < kilobytes * 1024 {
            document.push_str("let value = compute(input);\n");
        }
        document.push_str("```\n");
        document
    };
    let measure = |document: &str| {
        let chunks = split(document, 1024);
        let start = Instant::now();
        let commits = streamed_commits(&chunks);
        let elapsed = start.elapsed();
        assert_eq!(commits.len(), 1);
        elapsed
    };

    let small = measure(&build(128)).max(Duration::from_micros(1));
    let large = measure(&build(512));
    assert!(large < small * 16, "512 KB took {large:?} against {small:?} for 128 KB");
}
