//! Remembered results of the scans in `braces.rs`, for one parser source.
//!
//! Each of those scans reports failure only after walking to the end of
//! its input, and a caller that gets a failure steps one byte and usually
//! asks again. A run of openers therefore paid one walk each: 31 KiB of
//! `<A {` lines took 0.15 s and grew x16 for every x4 of input. Every
//! method here returns exactly what the scan it is named after returns on
//! the same bytes; it only avoids repeating a walk whose outcome an earlier
//! call already fixed.

use std::cell::OnceCell;

use memchr::{memchr, memchr2, memmem, memrchr};
use rustc_hash::FxHashMap;
use smallvec::SmallVec;

use super::braces;

const UNCLOSED: u32 = u32::MAX;

pub(super) struct LexMemo {
    /// `{` offset to the offset after its `}`, or `UNCLOSED`.
    ///
    /// One walk settles every brace it passes: the ones that close get
    /// their end, the ones still open when the walk fails can never close.
    brace_ends: FxHashMap<u32, u32>,
    /// A group is scanned directly for this many bytes before anything is
    /// looked up, and walks shorter than this are left out of `brace_ends`:
    /// going that far again costs less than keeping an entry.
    short_walk: usize,
    open_braces: SmallVec<[u32; 8]>,
    last_close_brace: OnceCell<Option<usize>>,
    quotes: [Delimited; 3],
    block_comment: Delimited,
    /// A `//` comment start and the line end it ran to.
    line_comment: (usize, usize),
    /// A backtick offset and the end of the run it sits in.
    backtick_run: (usize, usize),
    /// A run end and the shortest opener known to find no closer after it.
    backtick_miss: (usize, usize),
}

/// What is known about one delimiter kind (`"…"`, `'…'`, `` `…` ``, `/*…*/`).
struct Delimited {
    /// Start and end of the last span that closed. A later opener of the
    /// same kind inside it closes at the same place.
    closed: (usize, usize),
    /// The first opener known not to close. No later one closes either.
    unclosed_from: usize,
}

impl Default for Delimited {
    fn default() -> Self {
        Self { closed: (0, 0), unclosed_from: usize::MAX }
    }
}

impl LexMemo {
    pub(super) fn new(short_walk: usize) -> Self {
        Self {
            brace_ends: FxHashMap::default(),
            short_walk,
            open_braces: SmallVec::new(),
            last_close_brace: OnceCell::new(),
            quotes: std::array::from_fn(|_| Delimited::default()),
            block_comment: Delimited::default(),
            line_comment: (0, 0),
            backtick_run: (0, 0),
            backtick_miss: (0, 0),
        }
    }

    /// `skip_braces(bytes, start)` for a `start` that holds `{`.
    pub(super) fn brace_end(&mut self, bytes: &[u8], start: usize) -> Option<usize> {
        // Nothing closes after the last `}`, so a walk stops there instead
        // of at the end of the source, and a brace past it needs no walk.
        let last_close = *self.last_close_brace.get_or_init(|| memrchr(b'}', bytes));
        let last_close = last_close.filter(|last| *last > start)?;
        if let Some(&end) = self.brace_ends.get(&(start as u32)) {
            return (end != UNCLOSED).then_some(end as usize);
        }
        // Most groups close within a few bytes, and a scan that is cut
        // off early still gives the full answer whenever it closes at all.
        let nearby = bytes.len().min(start + self.short_walk);
        if let Some(end) = braces::skip_braces(&bytes[..nearby], start) {
            return Some(end);
        }
        let mut open = std::mem::take(&mut self.open_braces);
        open.push(start as u32);
        let mut cursor = start + 1;
        let mut end = None;
        while cursor <= last_close {
            match bytes[cursor] {
                b'"' | b'\'' | b'`' => match self.quoted_end(bytes, cursor) {
                    Some(next) => cursor = next,
                    None => break,
                },
                b'/' if bytes.get(cursor + 1) == Some(&b'/') => {
                    cursor = self.line_comment_end(bytes, cursor);
                }
                b'/' if bytes.get(cursor + 1) == Some(&b'*') => {
                    match self.block_comment_end(bytes, cursor) {
                        Some(next) => cursor = next,
                        None => break,
                    }
                }
                b'{' => match self.brace_ends.get(&(cursor as u32)) {
                    // An earlier walk came through this brace in the same
                    // state. If it never closed, nothing around it does.
                    Some(&UNCLOSED) => break,
                    Some(&known) => cursor = known as usize,
                    None => {
                        open.push(cursor as u32);
                        cursor += 1;
                    }
                },
                b'}' => {
                    cursor += 1;
                    if let Some(opener) = open.pop() {
                        self.record_brace(opener, cursor, cursor as u32);
                    }
                    if open.is_empty() {
                        end = Some(cursor);
                        break;
                    }
                }
                _ => cursor += 1,
            }
        }
        for opener in open.drain(..) {
            self.record_brace(opener, cursor, UNCLOSED);
        }
        self.open_braces = open;
        end
    }

    /// Remembers what the walk found for `opener` by the time it got to
    /// `reached`, unless walking that far again is cheaper than the entry.
    fn record_brace(&mut self, opener: u32, reached: usize, end: u32) {
        if reached - opener as usize >= self.short_walk {
            self.brace_ends.insert(opener, end);
        }
    }

    /// `skip_quoted(bytes, start)` for a `start` that holds a quote byte.
    pub(super) fn quoted_end(&mut self, bytes: &[u8], start: usize) -> Option<usize> {
        let quote = bytes[start];
        let memo = &mut self.quotes[match quote {
            b'"' => 0,
            b'\'' => 1,
            _ => 2,
        }];
        if start >= memo.unclosed_from {
            return None;
        }
        // A quote byte inside a closed span was stepped over as an escape,
        // so a scan from it continues on the same bytes to the same end.
        if memo.closed.0 <= start && start + 1 < memo.closed.1 {
            return Some(memo.closed.1);
        }
        // Past `unclosed_from` the scan is back on the walk that already
        // ran out, so it does not need to be finished.
        let stop = bytes.len().min(memo.unclosed_from.saturating_add(1));
        let mut cursor = start + 1;
        while cursor < stop {
            if bytes[cursor] == b'\\' && cursor + 1 < bytes.len() {
                cursor += 2;
            } else if bytes[cursor] == quote {
                memo.closed = (start, cursor + 1);
                return Some(cursor + 1);
            } else {
                cursor += 1;
            }
        }
        memo.unclosed_from = start;
        None
    }

    /// `skip_block_comment(bytes, start)` for a `start` that holds `/*`.
    fn block_comment_end(&mut self, bytes: &[u8], start: usize) -> Option<usize> {
        let memo = &mut self.block_comment;
        if start >= memo.unclosed_from {
            return None;
        }
        if memo.closed.0 <= start && start + 4 <= memo.closed.1 {
            return Some(memo.closed.1);
        }
        // No `*/` starts at or after `unclosed_from + 2`.
        let stop = bytes.len().min(memo.unclosed_from.saturating_add(3));
        let Some(found) = memmem::find(&bytes[start + 2..stop], b"*/") else {
            memo.unclosed_from = start;
            return None;
        };
        let end = start + 2 + found + 2;
        memo.closed = (start, end);
        Some(end)
    }

    /// `skip_line_comment(bytes, start)` for a `start` that holds `//`.
    fn line_comment_end(&mut self, bytes: &[u8], start: usize) -> usize {
        let (from, end) = self.line_comment;
        if from <= start && start + 2 <= end {
            return end;
        }
        let end = memchr2(b'\n', b'\r', &bytes[start + 2..])
            .map_or(bytes.len(), |found| start + 2 + found);
        self.line_comment = (start, end);
        end
    }

    /// `skip_backticks(bytes, start)` for a `start` that holds a backtick.
    ///
    /// The closer is the first later run at least as long as the opener.
    pub(super) fn backticks_end(&mut self, bytes: &[u8], start: usize) -> Option<usize> {
        let (from, known_end) = self.backtick_run;
        let run_end = if from <= start && start < known_end {
            known_end
        } else {
            let end = backtick_run_end(bytes, start);
            self.backtick_run = (start, end);
            end
        };
        let open = run_end - start;
        // After a miss the caller retries one backtick further into the
        // same run, which only shortens the opener.
        if self.backtick_miss.0 == run_end && open >= self.backtick_miss.1 {
            return None;
        }
        let mut cursor = run_end;
        let mut longest = 0;
        while let Some(found) = memchr(b'`', &bytes[cursor..]) {
            let end = backtick_run_end(bytes, cursor + found);
            let len = end - (cursor + found);
            if len >= open {
                return Some(end);
            }
            longest = longest.max(len);
            cursor = end;
        }
        self.backtick_miss = (run_end, longest + 1);
        None
    }
}

fn backtick_run_end(bytes: &[u8], start: usize) -> usize {
    let mut end = start;
    while end < bytes.len() && bytes[end] == b'`' {
        end += 1;
    }
    end
}

#[cfg(kani)]
mod proofs;
