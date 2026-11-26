//! Closer searches for math, remembered so that no opener repeats one.
//!
//! An opening `$` or `$$` is math only when something closes it, and the
//! only way to find that out is to scan forward. An opener that is never
//! closed scans to the end of what it is in — the block's inline content for
//! `$`, the whole source for a `$$` line — and stays literal, so the next
//! opener starts the same scan a few bytes further on. A run of them is
//! O(n²): 125 KB of `$a ` took 3.6 s, of `$$a` lines 19 s, both growing x16
//! or more for every x4 of input. Prices do this (`$5 and $10`), and so do
//! shell variables.
//!
//! Each search below is a pure function of where it starts, so its answer
//! can be kept and handed to every later opener that would retrace it.

use memchr::{memchr, memchr2, memrchr};
use smallvec::SmallVec;

use super::{Parser, can_close_inline, is_escaped_marker, is_line_end};

/// Slices this short are searched directly: the walk is cheaper than the
/// memo, and it bounds any repeated search to a small fixed amount of text.
/// Same threshold as `has_closer_from`.
const SHORT: usize = 256;

/// Keys for what these searches keep in the parser's closer memo, which is
/// indexed by a slice and the closer byte asked about (`has_closer_from`).
///
/// The delimiter scans ask about ASCII bytes. These are UTF-8 continuation
/// bytes, which no markup rule can ask about, so the entries cannot collide
/// with theirs; among themselves they are told apart by value.
mod key {
    /// The first inline closer in a slice, for a one- and a two-dollar
    /// opener.
    pub(super) const INLINE: [u8; 2] = [0x80, 0x81];
    /// The last `$$` in a slice that can close a display block.
    pub(super) const BLOCK: u8 = 0x82;
    /// From where a slice holds no `*` or `_`.
    pub(super) const NO_EMPHASIS: u8 = 0x83;
}

/// An inline search checks for an answer, and leaves its own, on the first
/// run it meets in each block of `1 << MEMO_BLOCK_SHIFT` bytes.
///
/// Two searches on the same path enter a block on the same run, so the later
/// one reads the earlier one's answer within a block of joining it. One
/// entry per 32 bytes keeps a single long search close to the cost of the
/// plain walk.
const MEMO_BLOCK_SHIFT: u32 = 5;

// The inline searches take `&'a` bytes for the same reason `has_closer_from`
// takes `&'a str`: an address names a byte range only for as long as the
// range lives, and the source and the arena outlive the memo.
impl<'a> Parser<'a> {
    /// The `$` run that closes inline math opened by `open_len` dollars, for
    /// a search that stands at `from`.
    ///
    /// The search hops between `$` and backtick runs: a code span is skipped
    /// whole, an unclosed backtick run and a `$` that cannot close are
    /// stepped over. Its only state is the run it stands on, so the first
    /// closer is a function of that run alone — whichever opener the search
    /// began from, and however it got there. That is what makes the answer
    /// reusable: an opener whose search reaches a run with an answer on it
    /// would retrace the search that left it, hop for hop.
    ///
    /// The answer is deliberately not "the last closing `$` in the content".
    /// Which code spans a search skips depends on where it starts, so a `$`
    /// can close for one opener and sit inside a code span for another; only
    /// a search that stands on the same run sees the same code spans.
    pub(super) fn inline_math_closer(
        &self,
        bytes: &'a [u8],
        from: usize,
        open_len: usize,
    ) -> Option<usize> {
        let remember = bytes.len() > SHORT;
        let mut cursor = from;
        let mut block = from >> MEMO_BLOCK_SHIFT;
        let mut marks: SmallVec<[usize; 8]> = SmallVec::new();

        let closer = loop {
            let Some(relative) = memchr2(b'$', b'`', &bytes[cursor..]) else {
                break None;
            };
            let run = cursor + relative;
            if remember && run >> MEMO_BLOCK_SHIFT != block {
                block = run >> MEMO_BLOCK_SHIFT;
                // An earlier search stood on this run: its answer, closer or
                // none, is this search's too.
                if let Some(known) =
                    self.last_closer.borrow().get(&inline_key(bytes, run, open_len))
                {
                    break known.map(|distance| run + distance);
                }
                marks.push(run);
            }

            if bytes[run] == b'`' {
                cursor = self
                    .closed_code_span_end(bytes, run)
                    .unwrap_or_else(|| run + Self::marker_run_len(bytes, run, b'`'));
            } else if closes_inline(bytes, run, open_len) {
                break Some(run);
            } else {
                cursor = run + 1;
            }
        };

        // Every run the search stood on leads to the same closer. A search
        // that stayed within a block or two is cheaper to repeat than to
        // keep, which is every ordinary `$x$`.
        if marks.len() > 1 {
            let mut memo = self.last_closer.borrow_mut();
            for run in marks {
                memo.insert(inline_key(bytes, run, open_len), closer.map(|closer| closer - run));
            }
        }
        closer
    }

    /// Whether `bytes[from..closer]` holds a `*` or `_`, for the opener
    /// whose first inner byte is `from` and whose closer is `closer`.
    ///
    /// A digit-prefixed opener that finds none stays literal, and the next
    /// one in front of the same closer asks about a range that ends at the
    /// same byte. What is kept is how far back from that closer the text is
    /// already known to hold neither, so only the bytes in front of that are
    /// ever read.
    pub(super) fn has_emphasis_marker_before(
        &self,
        bytes: &'a [u8],
        from: usize,
        closer: usize,
    ) -> bool {
        if closer - from <= SHORT {
            return memchr2(b'*', b'_', &bytes[from..closer]).is_some();
        }

        let key = (bytes.as_ptr() as usize, closer, key::NO_EMPHASIS);
        let clear_from = self.last_closer.borrow().get(&key).copied().flatten().unwrap_or(closer);
        if clear_from <= from {
            return false;
        }
        if memchr2(b'*', b'_', &bytes[from..clear_from]).is_some() {
            return true;
        }
        self.last_closer.borrow_mut().insert(key, Some(from));
        false
    }

    /// Whether a `$$` that can close a display math block sits at or after
    /// `from` in the source.
    ///
    /// Whether a `$$` can close does not depend on the line that opens the
    /// block, so the position of the last one settles the question for every
    /// `$$` line in the source at once.
    pub(super) fn has_math_block_close_from(&self, from: usize) -> bool {
        let bytes = self.source.as_bytes();
        if bytes.len() <= SHORT {
            return math_block_close(bytes, from).is_some();
        }

        let key = (bytes.as_ptr() as usize, bytes.len(), key::BLOCK);
        let cached = self.last_closer.borrow().get(&key).copied();
        let last = if let Some(last) = cached {
            last
        } else {
            let last = last_math_block_close(bytes);
            self.last_closer.borrow_mut().insert(key, last);
            last
        };
        last.is_some_and(|last| last >= from)
    }
}

/// Memo key for the first inline closer of a search that stands on `run`:
/// the rest of the content from that run on, and the opener's length. The
/// entry holds the closer's distance from the run, or `None` when nothing
/// closes.
///
/// The little a search reads in front of a `$` it is judging — the byte
/// before it, the backslashes escaping it — lies after the opener the search
/// began from, so it is the same in every content slice that can reach this
/// run.
fn inline_key(bytes: &[u8], run: usize, open_len: usize) -> (usize, usize, u8) {
    let rest = &bytes[run..];
    (rest.as_ptr() as usize, rest.len(), key::INLINE[open_len - 1])
}

/// The first `$$` at or after `cursor` that can close a display math block.
pub(super) fn math_block_close(bytes: &[u8], mut cursor: usize) -> Option<usize> {
    while let Some(relative) = memchr(b'$', &bytes[cursor..]) {
        let dollar = cursor + relative;
        if is_math_block_close(bytes, dollar) {
            return Some(dollar);
        }
        cursor = dollar + 1;
    }
    None
}

fn last_math_block_close(bytes: &[u8]) -> Option<usize> {
    let mut end = bytes.len();
    while let Some(dollar) = memrchr(b'$', &bytes[..end]) {
        if is_math_block_close(bytes, dollar) {
            return Some(dollar);
        }
        end = dollar;
    }
    None
}

/// An unescaped `$$` with nothing but spaces and tabs after it on its line.
fn is_math_block_close(bytes: &[u8], dollar: usize) -> bool {
    bytes.get(dollar + 1) == Some(&b'$')
        && is_line_end(bytes, dollar + 2)
        && !is_escaped_marker(bytes, dollar)
}

/// Whether the `$` at `index` closes inline math opened by `open_len`
/// dollars: unescaped, at least as many dollars as the opener, and placed
/// where a closer may stand.
///
/// Only the opener's worth of dollars is read. Measuring the whole run made
/// a line of nothing but `$` quadratic, since every closer in it re-counted
/// the rest of the line.
fn closes_inline(bytes: &[u8], index: usize, open_len: usize) -> bool {
    !is_escaped_marker(bytes, index)
        && bytes.get(index..index + open_len).is_some_and(|run| run.iter().all(|&b| b == b'$'))
        && can_close_inline(bytes, index, open_len)
}
