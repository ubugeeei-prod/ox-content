//! Where a `[` closes.
//!
//! Link text, image alt text and reference labels all end at the `]` that
//! balances their `[`. Finding it means walking the bracketed text, and a
//! bracket that turns out not to be a link hands its text back to the
//! caller, which meets the brackets inside it and walks the same bytes
//! again — once per nesting level. `[[[[…]]]]` therefore cost O(n²) in
//! scanning alone.
//!
//! One walk already balances every bracket it passes, so the long ones are
//! recorded here and answered from the table afterwards.

use memchr::memchr;
use rustc_hash::FxHashMap;
use smallvec::SmallVec;

use crate::parser::Parser;

/// Brackets recorded so far, keyed by the address of the `[`.
///
/// Every slice the inline parser sees lives in the source or the arena,
/// both of which outlive the parser, so an address names one byte for as
/// long as the table exists.
#[derive(Default)]
pub(in crate::parser) struct BracketMatches {
    matches: FxHashMap<usize, BracketMatch>,
    /// For each recording scan, the `(start, end)` addresses of the
    /// constructs it stepped over whole, in order.
    jumps: SmallVec<[Jumps; 1]>,
}

/// `(start, end)` addresses of the constructs one scan stepped over whole.
type Jumps = SmallVec<[(usize, usize); 2]>;

#[derive(Clone, Copy)]
struct BracketMatch {
    /// Address of the matching `]`, or `None` when the scan ran out first.
    close: Option<usize>,
    /// Address one past the content the scan was allowed to read.
    content_end: usize,
    /// Index of the recording scan in [`BracketMatches::jumps`].
    scan: usize,
}

impl BracketMatches {
    /// What a scan of the same `[` in content ending at `content_end`
    /// finds, when the recorded one already settles it.
    ///
    /// The same `[` is looked at through slices that end in different
    /// places: link text is scanned as its own slice. Up to the end of the
    /// shorter slice, both scans make the same decisions — except where the
    /// longer one stepped over a code span, autolink or raw HTML that runs
    /// past that end, which the shorter one cannot see the end of. Without
    /// such a crossing, a match inside the shorter slice holds as it is,
    /// and a match beyond it (or none at all) means none inside it.
    fn answer(&self, key: usize, content_end: usize) -> Recorded {
        let Some(recorded) = self.matches.get(&key) else {
            return Recorded::Unknown;
        };
        if content_end > recorded.content_end {
            return Recorded::Unknown;
        }
        if let Some(close) = recorded.close
            && close < content_end
        {
            return Recorded::Close(close);
        }
        if content_end == recorded.content_end {
            return Recorded::Unclosed;
        }
        let Some(jumps) = self.jumps.get(recorded.scan) else {
            return Recorded::Unknown;
        };
        let after = jumps.partition_point(|&(start, _)| start < content_end);
        let crossed = after.checked_sub(1).is_some_and(|index| jumps[index].1 > content_end);
        if crossed { Recorded::Unknown } else { Recorded::Unclosed }
    }
}

enum Recorded {
    Close(usize),
    Unclosed,
    Unknown,
}

/// Brackets this short are scanned directly: the common `[text](url)` then
/// never touches the table, and a miss costs at most this many bytes.
const DIRECT_SCAN_BYTES: usize = 128;

enum Step {
    Open,
    Close,
    /// Something that binds tighter than brackets ends here.
    Jump(usize),
}

impl Parser<'_> {
    /// Index of the `]` matching the bracket opened just before `start`, or
    /// `content.len()` when nothing closes it.
    ///
    /// Constructs that bind tighter than brackets are skipped whole:
    /// backslash escapes, code spans (an unmatched opener stays literal),
    /// autolinks, and inline raw HTML. This is what makes
    /// `[not a `link](/foo`)` a code span instead of a link.
    pub(in crate::parser) fn matching_bracket(&self, content: &str, start: usize) -> usize {
        debug_assert!(start > 0, "`start` follows the opening bracket");
        let base = content.as_ptr().addr();
        let content_end = base + content.len();
        let key = base + start - 1;

        let known = {
            let matches = self.bracket_matches.borrow();
            if matches.matches.is_empty() {
                Recorded::Unknown
            } else {
                matches.answer(key, content_end)
            }
        };
        match known {
            Recorded::Close(close) => return close - base,
            Recorded::Unclosed => return content.len(),
            Recorded::Unknown => {}
        }

        if let Some(close) = Self::scan_bracket_directly(content, start) {
            return close;
        }
        self.scan_and_record_brackets(content, start)
    }

    /// The plain scan, abandoned once it has read [`DIRECT_SCAN_BYTES`].
    fn scan_bracket_directly(content: &str, start: usize) -> Option<usize> {
        let bytes = content.as_bytes();
        let give_up = start.saturating_add(DIRECT_SCAN_BYTES);
        let mut depth = 1usize;
        let mut cursor = start;
        while cursor < bytes.len() {
            if cursor >= give_up {
                return None;
            }
            match Self::bracket_step(content, cursor) {
                Step::Open => {
                    depth += 1;
                    cursor += 1;
                }
                Step::Close => {
                    depth -= 1;
                    if depth == 0 {
                        return Some(cursor);
                    }
                    cursor += 1;
                }
                Step::Jump(to) => cursor = to,
            }
        }
        Some(bytes.len())
    }

    /// The same scan, recording every bracket it balances on the way.
    fn scan_and_record_brackets(&self, content: &str, start: usize) -> usize {
        let bytes = content.as_bytes();
        let base = content.as_ptr().addr();
        let content_end = base + content.len();
        let mut table = self.bracket_matches.borrow_mut();
        let scan = table.jumps.len();
        let mut jumps = Jumps::new();
        let mut open: SmallVec<[usize; 16]> = SmallVec::new();
        open.push(start - 1);

        let mut cursor = start;
        let mut found = None;
        while cursor < bytes.len() {
            match Self::bracket_step(content, cursor) {
                Step::Open => {
                    open.push(cursor);
                    cursor += 1;
                }
                Step::Close => {
                    if let Some(opener) = open.pop() {
                        table.matches.insert(
                            base + opener,
                            BracketMatch { close: Some(base + cursor), content_end, scan },
                        );
                    }
                    if open.is_empty() {
                        found = Some(cursor);
                        break;
                    }
                    cursor += 1;
                }
                Step::Jump(to) => {
                    if to > cursor + 1 {
                        jumps.push((base + cursor, base + to));
                    }
                    cursor = to;
                }
            }
        }

        // Whatever is still open ran to the end of the content, and would
        // again from its own start.
        for opener in open {
            table.matches.insert(base + opener, BracketMatch { close: None, content_end, scan });
        }
        table.jumps.push(jumps);
        found.unwrap_or(bytes.len())
    }

    /// Classifies the byte at `cursor` for bracket balancing.
    fn bracket_step(content: &str, cursor: usize) -> Step {
        let bytes = content.as_bytes();
        match bytes[cursor] {
            b'\\' => {
                // An escaped ASCII punctuation byte (which covers both
                // delimiters) is inert for bracket matching.
                let escapes_next =
                    cursor + 1 < bytes.len() && bytes[cursor + 1].is_ascii_punctuation();
                Step::Jump(cursor + if escapes_next { 2 } else { 1 })
            }
            b'`' => {
                let run = Self::marker_run_len(bytes, cursor, b'`');
                let after_opener = cursor + run;
                let mut scan = after_opener;
                while scan < bytes.len() {
                    let Some(off) = memchr(b'`', &bytes[scan..]) else {
                        break;
                    };
                    scan += off;
                    let closer = Self::marker_run_len(bytes, scan, b'`');
                    if closer == run {
                        return Step::Jump(scan + closer);
                    }
                    scan += closer;
                }
                Step::Jump(after_opener)
            }
            b'<' => {
                if let Some(end) = super::autolink_end(content, cursor) {
                    Step::Jump(end)
                } else if let Some((_, end)) = Parser::parse_inline_html(content, cursor, 0) {
                    Step::Jump(end)
                } else {
                    Step::Jump(cursor + 1)
                }
            }
            b'[' => Step::Open,
            b']' => Step::Close,
            _ => Step::Jump(cursor + 1),
        }
    }
}
