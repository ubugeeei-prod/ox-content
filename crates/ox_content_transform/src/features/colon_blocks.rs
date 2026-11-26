//! Where a `:::` block closes, for the passes that rewrite whole blocks
//! (cards, code groups).
//!
//! A block opened on line `i` closes on the first `:::` line that empties a
//! stack started as `[i]`: every `::: name` line after it pushes its colon
//! count, and a closer of `k` colons pops back through the innermost entry of
//! at most `k`. Lines inside a fence count for nothing. Searching like that
//! from every opener cost a walk to the end of the document for each one
//! that never closes, so a run of `::: card` lines took O(n²): 64 KiB took
//! 1.6 s, and 256 KiB 22 s.
//!
//! One search settles more than its own opener. Every entry it pushes sees,
//! above itself, exactly what its own search would see, until something
//! below it is popped: an entry that a closer pops as the innermost match
//! closes there in its own search too, and one still on the stack when the
//! search runs out never closes. Those answers are kept, so a run of
//! unclosed openers costs one search. An entry popped only because an entry
//! beneath it matched is left to search for itself; that work is capped, and
//! once the cap is spent such an opener stays unclosed, as it would if its
//! closer were never found.

use super::segments::{is_closing_fence, parse_opening_fence};

#[cfg(test)]
pub(super) mod tests;

/// Searches of their own may walk this many lines per line of the document
/// in total, plus [`EXTRA_STEPS`]. Documents have to be built for it to
/// matter: it takes a closer that pops an outer block out from under inner
/// ones that a narrower closer could not close.
const STEPS_PER_LINE: usize = 16;
const EXTRA_STEPS: usize = 4096;

#[derive(Clone, Copy)]
enum Line {
    /// Inside a fence, a fence line, or anything that is neither of below.
    Other,
    Opener(usize),
    Closer(usize),
}

#[derive(Clone, Copy)]
enum Known {
    Unknown,
    Closes(usize),
    Unclosed,
}

pub(super) struct ColonBlocks {
    lines: Vec<Line>,
    known: Vec<Known>,
    /// The widest closer at or after each line, so an opener wider than all
    /// of them is settled without a search.
    widest_closer_from: Vec<usize>,
    steps_left: usize,
}

impl ColonBlocks {
    /// Classifies `lines` once. `opener` and `closer` return the colon count
    /// of a line that opens or closes a block; both see the line with up to
    /// three spaces of indentation removed.
    pub(super) fn new(
        lines: &[(&str, &str)],
        opener: impl Fn(&str) -> Option<usize>,
        closer: impl Fn(&str) -> Option<usize>,
    ) -> Self {
        let mut kinds = Vec::with_capacity(lines.len());
        let mut fence: Option<(u8, usize)> = None;
        for (line, _) in lines {
            if let Some((fence_char, fence_len)) = fence {
                if is_closing_fence(line, fence_char, fence_len) {
                    fence = None;
                }
                kinds.push(Line::Other);
                continue;
            }
            if let Some(open) = parse_opening_fence(line) {
                fence = Some((open.fence_char, open.fence_len));
                kinds.push(Line::Other);
                continue;
            }
            let trimmed = trim_container_indent(line);
            kinds.push(if let Some(colons) = opener(trimmed) {
                Line::Opener(colons)
            } else if let Some(colons) = closer(trimmed) {
                Line::Closer(colons)
            } else {
                Line::Other
            });
        }

        let mut widest_closer_from = vec![0; kinds.len() + 1];
        for (index, kind) in kinds.iter().enumerate().rev() {
            let own = if let Line::Closer(colons) = kind { *colons } else { 0 };
            widest_closer_from[index] = own.max(widest_closer_from[index + 1]);
        }

        Self {
            known: vec![Known::Unknown; kinds.len()],
            steps_left: kinds.len().saturating_mul(STEPS_PER_LINE).saturating_add(EXTRA_STEPS),
            lines: kinds,
            widest_closer_from,
        }
    }

    /// The line that closes the block opened on `start`, if one does before
    /// `limit`. `start` must be outside a fence.
    pub(super) fn closer(&mut self, start: usize, limit: usize) -> Option<usize> {
        let Some(&Line::Opener(colons)) = self.lines.get(start) else {
            return None;
        };
        let known = match self.known[start] {
            Known::Unknown if self.widest_closer_from[start + 1] < colons => Known::Unclosed,
            Known::Unknown => self.search(start, colons),
            known => known,
        };
        match known {
            Known::Closes(end) if end < limit => Some(end),
            _ => None,
        }
    }

    fn search(&mut self, start: usize, colons: usize) -> Known {
        let mut stack = vec![(colons, start)];
        for index in start + 1..self.lines.len() {
            if self.steps_left == 0 {
                return Known::Unclosed;
            }
            self.steps_left -= 1;
            match self.lines[index] {
                Line::Other => {}
                Line::Opener(colons) => stack.push((colons, index)),
                Line::Closer(close) => {
                    let Some(matched) = stack.iter().rposition(|(open, _)| *open <= close) else {
                        continue;
                    };
                    self.known[stack[matched].1] = Known::Closes(index);
                    stack.truncate(matched);
                    if stack.is_empty() {
                        return Known::Closes(index);
                    }
                }
            }
        }
        for (_, line) in stack {
            self.known[line] = Known::Unclosed;
        }
        Known::Unclosed
    }
}

pub(super) fn trim_container_indent(line: &str) -> &str {
    let bytes = line.as_bytes();
    let mut indent = 0usize;
    while indent < bytes.len() && indent < 3 && bytes[indent] == b' ' {
        indent += 1;
    }
    &line[indent..]
}
