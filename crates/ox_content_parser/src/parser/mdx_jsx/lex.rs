//! Brace, string and code-span skips for one scanned slice.
//!
//! [`Lex::plain`] runs the reference scans in `braces.rs` directly. A lexer
//! built with [`Lex::shared`] returns the same answers, but reads them from
//! a [`LexMemo`] that covers the whole parser source: a skip that has been
//! walked once is not walked again by the next opener in the same run.
//!
//! The memo always scans the full source and then cuts the answer down to
//! the slice in hand. That is exact because none of these scans looks past
//! the end it reports: a skip that closes inside the slice closes at the
//! same place in the source, and one that does not close in the source
//! cannot close in a prefix of it. Code-span runs are the one construct
//! that peeks a byte further, so callers do not share a slice whose end
//! splits a backtick run.

use super::braces;
use super::lex_memo::LexMemo;

/// Skips over one slice. Every offset taken and returned is relative to it.
pub(super) struct Lex<'m> {
    bytes: &'m [u8],
    shared: Option<Shared<'m>>,
}

struct Shared<'m> {
    memo: &'m mut LexMemo,
    /// The parser source `bytes` was cut from.
    source: &'m [u8],
    /// Offset of `bytes` in `source`.
    base: usize,
}

impl<'m> Lex<'m> {
    pub(super) fn plain(bytes: &'m [u8]) -> Self {
        Self { bytes, shared: None }
    }

    /// `bytes` must be `source[base..base + bytes.len()]`.
    pub(super) fn shared(
        bytes: &'m [u8],
        memo: &'m mut LexMemo,
        source: &'m [u8],
        base: usize,
    ) -> Self {
        Self { bytes, shared: Some(Shared { memo, source, base }) }
    }

    pub(super) fn bytes(&self) -> &'m [u8] {
        self.bytes
    }

    pub(super) fn skip_braces(&mut self, start: usize) -> Option<usize> {
        let Some(shared) = &mut self.shared else {
            return braces::skip_braces(self.bytes, start);
        };
        if self.bytes.get(start) != Some(&b'{') {
            return None;
        }
        let end = shared.memo.brace_end(shared.source, shared.base + start);
        shared.within(self.bytes, end)
    }

    pub(super) fn skip_quoted(&mut self, start: usize) -> Option<usize> {
        let Some(shared) = &mut self.shared else {
            return braces::skip_quoted(self.bytes, start);
        };
        if !matches!(self.bytes.get(start), Some(b'"' | b'\'' | b'`')) {
            return braces::skip_quoted(self.bytes, start);
        }
        let end = shared.memo.quoted_end(shared.source, shared.base + start);
        shared.within(self.bytes, end)
    }

    pub(super) fn skip_backticks(&mut self, start: usize) -> Option<usize> {
        let Some(shared) = &mut self.shared else {
            return braces::skip_backticks(self.bytes, start);
        };
        if self.bytes.get(start) != Some(&b'`') {
            return None;
        }
        let end = shared.memo.backticks_end(shared.source, shared.base + start);
        shared.within(self.bytes, end)
    }
}

impl Shared<'_> {
    /// Cuts an end offset in the source down to the slice.
    fn within(&self, bytes: &[u8], end: Option<usize>) -> Option<usize> {
        end.filter(|end| *end <= self.base + bytes.len()).map(|end| end - self.base)
    }
}
