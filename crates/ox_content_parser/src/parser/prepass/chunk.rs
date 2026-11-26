//! The paragraph chunk a definition candidate is parsed from.
//!
//! A link reference definition may span lines but never a blank one, so the
//! pre-pass hands the definition parser everything from the candidate line to
//! the next blank line, with block quote markers stripped and every line
//! ending written as `\n`.
//!
//! That text used to be copied into the arena for every candidate line. The
//! arena never frees, and a chunk has no blank line to stop at when the
//! definitions are separated by headings, thematic breaks, or code fences
//! instead: `"[x]: /u\n***\n"` repeated 4,000 times (48 KB) copied the rest of
//! the document 4,000 times — half a second and 594 MB of arena, both growing
//! x16 for every x4 of input.
//!
//! Every candidate between two blank lines shares the same chunk end, and the
//! chunk of a later line is a suffix of the chunk of an earlier one. So the
//! window is found once and remembered, the same way `definition_region`
//! bounds the block parser's side, and it is copied only when the source bytes
//! are not already the text the definition parser needs. No quote marker to
//! strip and a bare `\n` after every line is the common case, and it borrows.

use memchr::{memchr, memchr_iter, memrchr};

use crate::parser::Parser;
use crate::parser::line_scan::{line_end as scan_line_end, line_terminator_end, next_line_start};
use crate::parser::reference::strip_quote_markers;

/// The chunk window the last definition candidate was resolved in.
///
/// Starts out empty, so the first candidate always scans.
#[derive(Default)]
pub(super) struct ChunkWindow<'a> {
    /// Source offset of the candidate line the window was scanned from.
    start: usize,
    /// Source offset of the blank line (or end of input) that ends the chunk.
    /// No line in `start..end` is blank once its quote markers are stripped,
    /// so every line start in that range has a chunk ending here too.
    end: usize,
    /// The stripped copy of `start..end`, when the source could not stand in.
    joined: Option<Joined<'a>>,
}

/// A stripped, `\n`-joined copy of a window's lines.
struct Joined<'a> {
    text: &'a str,
    /// A line start in the source and the offset of that line's copy in
    /// `text`. The pre-pass only moves forward, so this pair walks forward
    /// with it and the whole window is stepped over once.
    line: usize,
    offset: usize,
}

impl<'a> ChunkWindow<'a> {
    /// The chunk that starts with the line at `pos`: that line and the ones
    /// after it up to the next blank line, each with its block quote markers
    /// stripped and followed by `\n`.
    pub(super) fn text_at(&mut self, parser: &Parser<'a>, pos: usize) -> &'a str {
        let source = parser.source;
        if !(self.start <= pos && pos < self.end && self.seek(source.as_bytes(), pos)) {
            *self = Self::scan(parser, pos);
        }
        match &self.joined {
            Some(joined) => &joined.text[joined.offset..],
            None => &source[pos..self.end],
        }
    }

    /// Source offset of the line after the ones `consumed` covers, where
    /// `consumed` is a prefix of the text [`Self::text_at`] returned for
    /// `pos`. `None` when it does not reach the end of a line.
    pub(super) fn line_after(&self, bytes: &[u8], pos: usize, consumed: &str) -> Option<usize> {
        let last_newline = memrchr(b'\n', consumed.as_bytes())?;
        if self.joined.is_none() {
            // Borrowed text is the source itself, so its offsets carry over.
            return Some(pos + last_newline + 1);
        }
        // A copied line can be shorter than its source, so count lines.
        let mut next = pos;
        for _ in memchr_iter(b'\n', consumed.as_bytes()) {
            next = next_line_start(bytes, next);
        }
        Some(next)
    }

    /// Moves the copy's line cursor forward to the line at `pos`. `false`
    /// when `pos` is not a line start the cursor can still reach.
    fn seek(&mut self, bytes: &[u8], pos: usize) -> bool {
        let Some(joined) = &mut self.joined else {
            return true;
        };
        while joined.line < pos {
            let Some(newline) = memchr(b'\n', &joined.text.as_bytes()[joined.offset..]) else {
                return false;
            };
            joined.line = next_line_start(bytes, joined.line);
            joined.offset += newline + 1;
        }
        joined.line == pos
    }

    /// Finds the window that starts at `pos`, copying it only if it has to.
    fn scan(parser: &Parser<'a>, pos: usize) -> Self {
        let source = parser.source;
        let bytes = source.as_bytes();
        let mut borrowable = true;
        let mut joined_len = 0;
        let mut end = pos;
        while end < bytes.len() {
            let line_end = scan_line_end(bytes, end);
            let line = &source[end..line_end];
            let stripped = strip_quote_markers(line);
            if stripped.trim().is_empty() {
                break;
            }
            // The source can stand in for the joined text only while each
            // line is its own stripped form and ends in exactly one `\n`:
            // not `\r\n`, not a lone `\r`, and not the end of input.
            borrowable &= stripped.len() == line.len() && bytes.get(line_end) == Some(&b'\n');
            joined_len += stripped.len() + 1;
            end = line_terminator_end(bytes, line_end);
        }
        if borrowable {
            return Self { start: pos, end, joined: None };
        }

        // Sized by the scan above, so the copy is one allocation of exactly
        // the joined length and never grows.
        let mut text =
            ox_content_allocator::String::with_capacity_in(joined_len, parser.allocator.bump());
        let mut line_start = pos;
        while line_start < end {
            let line_end = scan_line_end(bytes, line_start);
            text.push_str(strip_quote_markers(&source[line_start..line_end]));
            text.push('\n');
            line_start = line_terminator_end(bytes, line_end);
        }
        let joined = Joined { text: text.into_bump_str(), line: pos, offset: 0 };
        Self { start: pos, end, joined: Some(joined) }
    }
}

#[cfg(test)]
mod tests;
