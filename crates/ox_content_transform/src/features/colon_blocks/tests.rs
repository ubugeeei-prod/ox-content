use std::time::{Duration, Instant};

use super::super::segments::{is_closing_fence, parse_opening_fence};
use super::{ColonBlocks, trim_container_indent};

/// Lines whose mix exercises nesting, closers of every width, fences, and
/// indentation.
pub(in crate::features) const LINES: [&str; 18] = [
    "::: card",
    ":::: card",
    "::::: card[T]",
    "::: card-grid",
    ":::: card-grid",
    "::: link-card[G]{/g}",
    "::: code-group",
    ":::: code-group",
    "::: tip",
    ":::::: tip",
    ":::",
    "::::",
    ":::::",
    "   :::",
    "```js [a]",
    "```",
    "~~~",
    "text",
];

/// Deterministic xorshift, so a failure names a document that fails again.
pub(in crate::features) struct Random(pub(in crate::features) u64);

impl Random {
    pub(in crate::features) fn below(&mut self, bound: usize) -> usize {
        self.0 ^= self.0 << 13;
        self.0 ^= self.0 >> 7;
        self.0 ^= self.0 << 17;
        (self.0 % bound as u64) as usize
    }

    pub(in crate::features) fn document(&mut self) -> String {
        let mut source = String::new();
        for _ in 0..1 + self.below(40) {
            source.push_str(LINES[self.below(LINES.len())]);
            source.push('\n');
        }
        source
    }
}

/// The search every opener used to run for itself.
pub(in crate::features) fn reference_closer(
    lines: &[(&str, &str)],
    start: usize,
    colon_count: usize,
    opener: impl Fn(&str) -> Option<usize>,
    closer: impl Fn(&str) -> Option<usize>,
) -> Option<usize> {
    let mut fence: Option<(u8, usize)> = None;
    let mut stack = vec![colon_count];
    for (offset, (line, _)) in lines.iter().enumerate().skip(start + 1) {
        if let Some((fence_char, fence_len)) = fence {
            if is_closing_fence(line, fence_char, fence_len) {
                fence = None;
            }
            continue;
        }
        if let Some(open) = parse_opening_fence(line) {
            fence = Some((open.fence_char, open.fence_len));
            continue;
        }
        let trimmed = trim_container_indent(line);
        if let Some(open) = opener(trimmed) {
            stack.push(open);
            continue;
        }
        if let Some(close) = closer(trimmed)
            && let Some(index) = stack.iter().rposition(|open| *open <= close)
        {
            stack.truncate(index);
            if stack.is_empty() {
                return Some(offset);
            }
        }
    }
    None
}

fn colons(line: &str) -> Option<usize> {
    let count = line.bytes().take_while(|byte| *byte == b':').count();
    (count >= 3).then_some(count)
}

fn opener(line: &str) -> Option<usize> {
    colons(line).filter(|count| !line[*count..].trim().is_empty())
}

fn closer(line: &str) -> Option<usize> {
    colons(line).filter(|count| line[*count..].trim().is_empty())
}

fn split(source: &str) -> Vec<(&str, &str)> {
    source.split_inclusive('\n').map(|line| (line.trim_end_matches('\n'), "\n")).collect()
}

/// Whether line `index` is an opener outside a fence.
fn openers(lines: &[(&str, &str)]) -> Vec<Option<usize>> {
    let mut fence: Option<(u8, usize)> = None;
    let mut found = Vec::new();
    for (line, _) in lines {
        if let Some((fence_char, fence_len)) = fence {
            if is_closing_fence(line, fence_char, fence_len) {
                fence = None;
            }
            found.push(None);
        } else if let Some(open) = parse_opening_fence(line) {
            fence = Some((open.fence_char, open.fence_len));
            found.push(None);
        } else {
            found.push(opener(trim_container_indent(line)));
        }
    }
    found
}

#[test]
fn remembered_closers_agree_with_the_search_they_replaced() {
    let mut random = Random(0x9E37_79B9_7F4A_7C15);
    for _ in 0..4000 {
        let source = random.document();
        let lines = split(&source);
        let mut blocks = ColonBlocks::new(&lines, opener, closer);
        // In document order, the way the passes ask, and then in reverse,
        // which leans on answers other searches left behind.
        let order: Vec<usize> = (0..lines.len()).chain((0..lines.len()).rev()).collect();
        let starts = openers(&lines);
        for start in order {
            let Some(colon_count) = starts[start] else { continue };
            let expected = reference_closer(&lines, start, colon_count, opener, closer);
            assert_eq!(blocks.closer(start, lines.len()), expected, "line {start} of {source:?}");
            let limit = random.below(lines.len() + 1);
            let bounded = expected.filter(|end| *end < limit);
            assert_eq!(blocks.closer(start, limit), bounded, "line {start} of {source:?}");
        }
    }
}

fn time_closers(source: &str) -> Duration {
    let lines = split(source);
    let starts = openers(&lines);
    let started = Instant::now();
    let mut blocks = ColonBlocks::new(&lines, opener, closer);
    for (start, open) in starts.iter().enumerate() {
        if open.is_some() {
            std::hint::black_box(blocks.closer(start, lines.len()));
        }
    }
    started.elapsed()
}

#[test]
fn unclosed_openers_cost_linear_time() {
    // Nothing closes; one wide closer at the end that only closes the last
    // opener; and closers that close an outer block out from under the
    // inner ones, which have to search for themselves.
    for (prefix, unit, suffix) in [
        ("", "::: card\n", ""),
        ("", "::: card\n", "::::::::\n"),
        ("::::::: card\n", "::: tip\n::::: card\n::::\n", ""),
    ] {
        let build = |lines: usize| format!("{prefix}{}{suffix}", unit.repeat(lines));
        let (small, large) = (build(4_000), build(16_000));
        let mut best = f64::INFINITY;
        for _ in 0..5 {
            let small_time = time_closers(&small).as_secs_f64();
            let large_time = time_closers(&large).as_secs_f64();
            if large_time < 0.002 {
                best = 0.0;
                break;
            }
            best = best.min(large_time / small_time.max(1e-9));
        }
        // 4x the lines: linear is about x4, quadratic about x16.
        assert!(best < 8.0, "{unit:?}: x{best:.1} for 4x the lines");
    }
}
