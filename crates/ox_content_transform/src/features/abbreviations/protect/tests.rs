use std::time::{Duration, Instant};

use super::{ProtectAhead, next_protected};

const PIECES: [&str; 22] = [
    "[",
    "]",
    "(",
    ")",
    "[a](b)",
    "[a][b]",
    "\\]",
    "\\)",
    "<",
    ">",
    "<a href=\"x\">",
    "</a>",
    "<code>",
    "</code>",
    "\"",
    "'",
    "<!--",
    "-->",
    "`",
    "a",
    " ",
    "\n",
];

fn spans(segment: &str, ahead: &mut ProtectAhead) -> Vec<Option<(usize, usize)>> {
    (0..segment.len())
        .map(|from| next_protected(segment, from, ahead).map(|span| (span.start, span.end)))
        .collect()
}

#[test]
fn remembered_searches_agree_with_searching_afresh() {
    let mut state: u64 = 0x9E37_79B9_7F4A_7C15;
    for round in 0..4000 {
        let pieces = 1 + round % 40;
        let mut segment = String::new();
        for _ in 0..pieces {
            state ^= state << 13;
            state ^= state >> 7;
            state ^= state << 17;
            segment.push_str(PIECES[(state % PIECES.len() as u64) as usize]);
        }
        assert_eq!(
            spans(&segment, &mut ProtectAhead::new(true)),
            spans(&segment, &mut ProtectAhead::new(false)),
            "for {segment:?}"
        );
    }
}

#[test]
fn unclosed_candidates_cost_linear_time() {
    // Each walks the rest of the segment to fail, then the walk moves on to
    // the next. The closer at the end keeps the "nothing closes at all"
    // shortcut from settling them.
    let time = |segment: &str| {
        (0..3)
            .map(|_| {
                let started = Instant::now();
                let mut ahead = ProtectAhead::new(true);
                let mut from = 0;
                while let Some(span) = next_protected(segment, from, &mut ahead) {
                    from = span.end.max(from + 1);
                }
                started.elapsed()
            })
            .min()
            .unwrap_or(Duration::MAX)
    };
    for (unit, suffix) in [("<!-- ", ">"), ("< \"\"", "\">"), ("[a](", "]"), ("<", ">")] {
        let build = |bytes: usize| format!("{}{suffix}", unit.repeat(bytes / unit.len()));
        let (small, large) = (build(16 * 1024), build(64 * 1024));
        let (small_time, large_time) = (time(&small), time(&large));
        // 4x the input: linear is about x4, quadratic about x16.
        assert!(
            large_time < small_time * 8 + Duration::from_millis(20),
            "{unit:?}: 64 KiB took {large_time:?} against {small_time:?} for 16 KiB"
        );
    }
}
