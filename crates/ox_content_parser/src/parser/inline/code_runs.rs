//! Where a code span closes, remembered so that no opener repeats a search
//! that already failed.
//!
//! A code span closes at the next backtick run of exactly its opener's
//! length, and an opener with no such run stays literal — after a walk to
//! the end of the content. Backtick runs of many lengths, none closed,
//! therefore cost one walk each: `` ` `` then ``` `` ``` then ```` ``` ````
//! and so on took O(n^1.5), and every scan that steps over code spans (for
//! link text, math, wiki links, sub- and superscript) paid it again.
//!
//! A walk to the end has seen every run after the place it started, so its
//! failure is kept as the last run of each length from there on (cmark keeps
//! the same table). A later opener of a length whose last run lies behind it
//! cannot close; one whose last run lies ahead does, and its walk ends there.

use memchr::memchr3;
use rustc_hash::FxHashMap;

use super::super::Parser;
use super::super::line_scan::{is_line_ending_byte, line_terminator_end};

/// Content this short is walked directly: the walk is cheaper than the memo,
/// and it bounds any repeated walk to a small fixed amount of text. Same
/// threshold as `has_closer_from`.
const SHORT: usize = 256;

/// Failed closer walks, keyed by the address and length of the content they
/// walked.
///
/// Every slice the inline parser sees lives in the source or the arena,
/// both of which outlive the parser, so an address names one byte range for
/// as long as the table exists.
pub(in crate::parser) type CodeRunMemo = FxHashMap<(usize, usize), CodeRuns>;

/// The backtick runs of one content slice from `from` on.
pub(in crate::parser) struct CodeRuns {
    /// Where the walk that recorded them started. Never inside a run.
    from: usize,
    /// For each run length, where the last run of that length starts.
    last: FxHashMap<usize, usize>,
}

/// The run that closes a code span, and whether a line ending comes first.
#[derive(Clone, Copy)]
pub(in crate::parser) struct CodeSpanCloser {
    pub(in crate::parser) start: usize,
    pub(in crate::parser) has_newline: bool,
}

impl<'a> Parser<'a> {
    /// Returns the byte after a closed code span that opens at `start`.
    ///
    /// Inline constructs outside code spans use this to skip over backtick
    /// regions while scanning for their own closing delimiter. An unmatched
    /// opener stays literal and therefore is not skipped.
    pub(in crate::parser) fn closed_code_span_end(
        &self,
        bytes: &'a [u8],
        start: usize,
    ) -> Option<usize> {
        let open_len = Self::marker_run_len(bytes, start, b'`');
        self.code_span_closer(bytes, start + open_len, open_len)
            .map(|closer| closer.start + open_len)
    }

    /// The backtick run of exactly `open_len` that closes a code span whose
    /// content starts at `from`.
    ///
    /// `from` follows the opening run, so it is not a backtick and every run
    /// after it is a whole run: which runs a walk meets does not depend on
    /// where it started.
    pub(in crate::parser) fn code_span_closer(
        &self,
        bytes: &'a [u8],
        from: usize,
        open_len: usize,
    ) -> Option<CodeSpanCloser> {
        let key = (bytes.as_ptr() as usize, bytes.len());
        let remember = bytes.len() > SHORT;
        if remember {
            let memo = self.code_runs.borrow();
            if let Some(runs) = memo.get(&key)
                && from >= runs.from
                && runs.last.get(&open_len).is_none_or(|&last| last < from)
            {
                return None;
            }
        }

        let closer = walk_to_closer(bytes, from, open_len);
        if closer.is_none() && remember {
            let mut memo = self.code_runs.borrow_mut();
            if memo.get(&key).is_none_or(|runs| from < runs.from) {
                memo.insert(key, record_runs(bytes, from));
            }
        }
        closer
    }
}

fn walk_to_closer(bytes: &[u8], from: usize, open_len: usize) -> Option<CodeSpanCloser> {
    let mut cursor = from;
    let mut has_newline = false;
    while cursor < bytes.len() {
        cursor += memchr3(b'`', b'\n', b'\r', &bytes[cursor..])?;
        if is_line_ending_byte(bytes[cursor]) {
            has_newline = true;
            cursor = line_terminator_end(bytes, cursor);
            continue;
        }
        let run = Parser::marker_run_len(bytes, cursor, b'`');
        if run == open_len {
            return Some(CodeSpanCloser { start: cursor, has_newline });
        }
        cursor += run;
    }
    None
}

fn record_runs(bytes: &[u8], from: usize) -> CodeRuns {
    let mut last = FxHashMap::default();
    let mut cursor = from;
    while let Some(relative) = memchr::memchr(b'`', &bytes[cursor..]) {
        let start = cursor + relative;
        let run = Parser::marker_run_len(bytes, start, b'`');
        last.insert(run, start);
        cursor = start + run;
    }
    CodeRuns { from, last }
}
