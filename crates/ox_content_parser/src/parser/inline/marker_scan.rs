//! Where the next byte that can start an inline construct is.
//!
//! Split out of `inline.rs` so the walk itself stays readable: this file is
//! two cursors and the rule for when they have to be recomputed.

use memchr::memchr;

use super::scan::next_inline_special;

/// A memo for one forward byte scan over a fixed slice.
///
/// Both scans below are position-independent: when the next hit at or
/// after `origin` is `hit`, it is still `hit` for every position in
/// `origin..=hit`. Recomputing only when the cursor leaves that window is
/// what keeps the walk linear.
#[derive(Clone, Copy)]
pub(super) struct ForwardScan {
    origin: usize,
    hit: usize,
}

impl ForwardScan {
    pub(super) const fn new() -> Self {
        // `usize::MAX` cannot be a real origin, so the first lookup always
        // computes.
        Self { origin: usize::MAX, hit: 0 }
    }

    pub(super) fn hit(&mut self, from: usize, find: impl FnOnce(usize) -> usize) -> usize {
        if from < self.origin || from > self.hit {
            self.origin = from;
            self.hit = find(from);
        }
        self.hit
    }
}

/// Where the next byte that can start an inline construct is.
///
/// `{` is not in the classifier's byte set, so with MDX on it was found by
/// scanning to the next real marker first and then searching the span in
/// between. In prose whose only markers are braces that first scan runs to
/// the end of the content — for every brace — so a line of `{a} ` cost
/// O(n²): 32 KiB took 6.8 ms against 0.005 ms without MDX, growing x15 for
/// every x4 of input. Caching both scans costs two words and removes the
/// repeat.
pub(super) struct InlineMarkerScan {
    brace: ForwardScan,
    mdx: bool,
    special: ForwardScan,
}

impl InlineMarkerScan {
    pub(super) const fn new(mdx: bool) -> Self {
        Self { brace: ForwardScan::new(), mdx, special: ForwardScan::new() }
    }

    pub(super) fn next(&mut self, bytes: &[u8], from: usize) -> usize {
        let special = self.special.hit(from, |at| next_inline_special(bytes, at));
        if !self.mdx {
            return special;
        }
        let brace = self
            .brace
            .hit(from, |at| memchr(b'{', &bytes[at..]).map_or(bytes.len(), |rel| at + rel));
        special.min(brace)
    }
}
