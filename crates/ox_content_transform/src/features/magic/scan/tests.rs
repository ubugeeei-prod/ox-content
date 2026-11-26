use std::time::{Duration, Instant};

use super::super::resolve;
use super::transform_with;
use crate::MagicLinkOptions;

fn options() -> super::super::ResolvedMagicLinks {
    resolve(Some(&MagicLinkOptions { enabled: Some(true), ..Default::default() })).expect("enabled")
}

/// Pieces whose mix leaves every kind of stop closed, unclosed, escaped, and
/// cut off by a line break.
const PIECES: [&str; 26] = [
    "{link:",
    "{link:a}",
    "{link:a|https://b.c}",
    "}",
    "{",
    "[",
    "]",
    "(",
    ")",
    "[a](b)",
    "\\]",
    "\\)",
    "<",
    ">",
    "<a href=\"x\">",
    "</a>",
    "<code>",
    "\"",
    "'",
    "<!--",
    "-->",
    "`",
    "a",
    " ",
    "\n",
    "\r\n",
];

#[test]
fn remembered_searches_agree_with_searching_afresh() {
    let options = options();
    let mut state: u64 = 0x9E37_79B9_7F4A_7C15;
    for round in 0..6000 {
        let pieces = if round < 5000 { 1 + round % 30 } else { 100 + round % 300 };
        let mut source = String::new();
        for _ in 0..pieces {
            state ^= state << 13;
            state ^= state >> 7;
            state ^= state << 17;
            source.push_str(PIECES[(state % PIECES.len() as u64) as usize]);
        }
        assert_eq!(
            transform_with(&source, &options, true),
            transform_with(&source, &options, false),
            "for {source:?}"
        );
    }
}

fn best_time(source: &str) -> Duration {
    let options = options();
    (0..3)
        .map(|_| {
            let started = Instant::now();
            std::hint::black_box(transform_with(source, &options, true));
            started.elapsed()
        })
        .min()
        .unwrap_or(Duration::MAX)
}

#[test]
fn stops_with_nothing_to_close_them_cost_linear_time() {
    // Measured before: 256 KiB of `<` took 99 s, of `< ""` before a quoted
    // `>` 17 s, of `[` 5.4 s, of `<!--` 3.9 s, of `{link:` 1.6 s.
    for (unit, suffix) in [
        ("{link:", ""),
        ("{link:a|", ""),
        ("[", ""),
        ("[a](", ""),
        ("<", ""),
        ("< \"\"", "\"\n>"),
        ("<!-- ", ""),
    ] {
        let build = |bytes: usize| {
            let mut source = String::from("{link:a} ");
            while source.len() < bytes {
                source.push_str(unit);
            }
            source.push_str(suffix);
            source
        };
        let (small, large) = (build(16 * 1024), build(64 * 1024));
        let (small_time, large_time) = (best_time(&small), best_time(&large));
        // 4x the input: linear is about x4, quadratic about x16.
        assert!(
            large_time < small_time * 8 + Duration::from_millis(20),
            "{unit:?}: 64 KiB took {large_time:?} against {small_time:?} for 16 KiB"
        );
    }
}
