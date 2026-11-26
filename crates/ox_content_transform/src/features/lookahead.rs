//! What a forward walk over Markdown source has found out about the text
//! ahead of it.
//!
//! The inline rewrites (magic links, keyboard keys, abbreviations) walk the
//! source and stop at `{`, `[`, `<` and the like, and several of those stops
//! search forward for something that closes them: a `}` on the same line, a
//! `]` or `)`, a `>` outside quotes, a `-->`. A stop whose search fails
//! stays literal and the walk moves one byte on, so a run of them searched
//! the same text once each: O(n²) on a line of `[` with no `]`, a run of `<`
//! with no `>`, or of `<!--`. Every one of these searches is a function of
//! where it starts, and the walks only move forward, so each answer is kept
//! for the stops after it.

/// The first match at or after a position, for positions that only grow.
pub(in crate::features) struct Ahead {
    remember: bool,
    known: Option<(usize, Option<usize>)>,
}

impl Ahead {
    pub(in crate::features) const fn new(remember: bool) -> Self {
        Self { remember, known: None }
    }

    /// The first match at or after `from`, by `find` when not already known.
    /// `find(from)` must return the first match at or after `from`.
    pub(in crate::features) fn find(
        &mut self,
        from: usize,
        find: impl FnOnce(usize) -> Option<usize>,
    ) -> Option<usize> {
        if self.remember
            && let Some((searched, found)) = self.known
            && searched <= from
            && found.is_none_or(|found| found >= from)
        {
            return found;
        }
        let found = find(from);
        self.known = Some((from, found));
        found
    }
}

/// Where raw HTML that starts ahead of a walk ends: tags, built on first use
/// and then answered for every `<`, and comments.
#[derive(Default)]
pub(in crate::features) struct HtmlAhead {
    tags: Option<TagEnds>,
    /// Where the last `-->` starts, once `comment_close_known`.
    last_comment_close: Option<usize>,
    comment_close_known: bool,
}

impl HtmlAhead {
    /// The byte after the `>` that ends the tag opened at `start`.
    pub(in crate::features) fn tag_end(&mut self, source: &str, start: usize) -> Option<usize> {
        self.tags.get_or_insert_with(|| TagEnds::new(source.as_bytes())).end(start)
    }

    /// The byte after the first `-->` at or after `from`.
    pub(in crate::features) fn comment_end(&mut self, source: &str, from: usize) -> Option<usize> {
        let bytes = source.as_bytes();
        if !self.comment_close_known {
            self.last_comment_close = memchr::memmem::rfind(bytes, b"-->");
            self.comment_close_known = true;
        }
        if self.last_comment_close.is_none_or(|last| last < from) {
            return None;
        }
        memchr::memmem::find(&bytes[from..], b"-->").map(|at| from + at + 3)
    }
}

/// Where the tag opened by the `<` at each position ends, for every `<` in
/// the source: the byte after the first `>` outside quotes.
///
/// A tag scan's only state is whether it is inside quotes, and which kind,
/// so the answers for all three states can be carried from the end of the
/// source to the start in one walk.
pub(in crate::features) struct TagEnds {
    /// `(position of '<', end)`, in order of position.
    ends: Vec<(usize, Option<usize>)>,
}

impl TagEnds {
    pub(in crate::features) fn new(bytes: &[u8]) -> Self {
        // The end a scan reaches from here: outside quotes, inside `"`,
        // inside `'`.
        let mut end: [Option<usize>; 3] = [None; 3];
        let mut ends = Vec::new();
        for at in (0..bytes.len()).rev() {
            // A scan from a `<` starts on the byte after it.
            if bytes[at] == b'<' {
                ends.push((at, end[0]));
            }
            let after = end;
            end = match bytes[at] {
                b'"' => [after[1], after[0], after[2]],
                b'\'' => [after[2], after[1], after[0]],
                b'>' => [Some(at + 1), after[1], after[2]],
                _ => after,
            };
        }
        ends.reverse();
        Self { ends }
    }

    /// Where the tag opened at `start`, which holds a `<`, ends.
    pub(in crate::features) fn end(&self, start: usize) -> Option<usize> {
        self.ends
            .binary_search_by_key(&start, |(at, _)| *at)
            .ok()
            .and_then(|index| self.ends[index].1)
    }
}
