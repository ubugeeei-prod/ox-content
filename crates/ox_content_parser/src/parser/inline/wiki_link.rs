//! Obsidian-style `[[target|label]]` links.
//!
//! Every `[[` in front of one `]]` is a candidate that ends there, so
//! whatever a candidate does that depends on that `]]` — finding it,
//! trimming in front of it, discovering a link in its label — is the same
//! work for the next candidate. A paragraph of `[[a|` candidates in front
//! of a single `]]` repeated all of it per candidate. [`WikiScan`]
//! remembers each answer for as long as the walk stays in front of the
//! same `]]`.

use memchr::memchr;
use ox_content_ast::{Link, Node, Span};
use smallvec::SmallVec;

use super::brackets::{BracketKind, InlineRun, OpenBracket};
use super::marker_scan::ForwardScan;
use crate::error::ParseResult;
use crate::parser::Parser;

/// What the wiki link candidates of one `parse_inline` call have learned.
/// Every position is an index into that call's content.
pub(in crate::parser) struct WikiScan {
    close: Option<CloseScan>,
    /// The next `|` and the next `[` in the content.
    pipe: ForwardScan,
    bracket: ForwardScan,
    /// A `]]` and the end of the trimmed text in front of it.
    trimmed: Option<(usize, usize)>,
    /// A `]]` whose candidate turned out to hold a link in its label, and
    /// where that link starts.
    rejected: Option<(usize, usize)>,
}

/// One search for `]]`: where it started, where the slice it read ended,
/// what it found, and the code spans it stepped over on the way.
struct CloseScan {
    from: usize,
    limit: usize,
    found: Option<usize>,
    /// `(opener, end)` of each code span skipped, in order. A search that
    /// starts inside one, or reads a slice that ends inside one, sees
    /// different text and has to look for itself.
    code_spans: SmallVec<[(usize, usize); 4]>,
}

impl CloseScan {
    /// Whether `pos` lies strictly inside a code span this search skipped.
    fn in_code_span(&self, pos: usize) -> bool {
        let after = self.code_spans.partition_point(|&(opener, _)| opener < pos);
        after.checked_sub(1).is_some_and(|index| pos < self.code_spans[index].1)
    }

    /// What a search of `..limit` starting at `from` finds, when this one
    /// already settles it.
    fn answer(&self, from: usize, limit: usize) -> Known {
        if from < self.from || limit > self.limit || self.in_code_span(from) {
            return Known::Unsettled;
        }
        match self.found {
            Some(close) if close < from => Known::Unsettled,
            // Everything up to a `]]` that fits was read the same way.
            Some(close) if close + 2 <= limit => Known::Close(close),
            // Nothing closer exists, unless cutting the slice short
            // reopens a code span that hid one.
            _ if limit == self.limit || !self.in_code_span(limit) => Known::NoClose,
            _ => Known::Unsettled,
        }
    }
}

/// What an earlier search says about a new one.
enum Known {
    Close(usize),
    NoClose,
    Unsettled,
}

impl WikiScan {
    pub(in crate::parser) const fn new() -> Self {
        Self {
            close: None,
            pipe: ForwardScan::new(),
            bracket: ForwardScan::new(),
            trimmed: None,
            rejected: None,
        }
    }

    /// Records that the candidate closing at `close` holds a link starting
    /// at `link_start` in its label.
    pub(in crate::parser) fn reject(&mut self, close: usize, link_start: usize) {
        self.rejected = Some((close, link_start));
    }

    /// The first `]]` in `bytes[from..limit]`.
    fn close(&mut self, bytes: &[u8], from: usize, limit: usize) -> Option<usize> {
        match self.close.as_ref().map_or(Known::Unsettled, |scan| scan.answer(from, limit)) {
            Known::Close(close) => return Some(close),
            Known::NoClose => return None,
            Known::Unsettled => {}
        }
        let mut code_spans = SmallVec::new();
        let found = scan_wiki_link_close(&bytes[..limit], from, &mut code_spans);
        self.close = Some(CloseScan { from, limit, found, code_spans });
        found
    }

    /// The first `|` in `bytes[from..close]`.
    fn pipe(&mut self, bytes: &[u8], from: usize, close: usize) -> Option<usize> {
        let pipe = self
            .pipe
            .hit(from, |at| memchr(b'|', &bytes[at..]).map_or(bytes.len(), |found| at + found));
        (pipe < close).then_some(pipe)
    }

    /// Whether `bytes[from..to]` holds a `[`.
    fn has_bracket(&mut self, bytes: &[u8], from: usize, to: usize) -> bool {
        let bracket = self
            .bracket
            .hit(from, |at| memchr(b'[', &bytes[at..]).map_or(bytes.len(), |found| at + found));
        bracket < to
    }

    /// Where `content[from..close]` ends once trailing whitespace is
    /// trimmed. `from` must be the start of text that is not all whitespace.
    fn trimmed_end(&mut self, content: &str, from: usize, close: usize) -> usize {
        if let Some((trimmed_close, end)) = self.trimmed
            && trimmed_close == close
            && from < end
        {
            return end;
        }
        let end = from + content[from..close].trim_end().len();
        self.trimmed = Some((close, end));
        end
    }
}

impl<'a> Parser<'a> {
    /// Tries the `[[target|label]]` form at `pos`. Returns whether it took
    /// over: either the link was pushed, or its label is now being parsed
    /// with the bracket left open.
    pub(in crate::parser) fn try_parse_wiki_link(
        &self,
        run: &mut InlineRun<'a>,
        content: &'a str,
        offset: usize,
        pos: &mut usize,
    ) -> ParseResult<bool> {
        let (_, limit) = run.window(content.len());
        let bytes = content.as_bytes();
        let open = *pos;
        let Some(close) = run.wiki.close(bytes, open + 2, limit) else {
            return Ok(false);
        };
        let pipe = run.wiki.pipe(bytes, open + 2, close);

        let inner_start = close - content[open + 2..close].trim_start().len();
        if inner_start == close || pipe == Some(inner_start) {
            // Nothing between the brackets, or nothing in front of the `|`.
            return Ok(false);
        }
        let inner_end = run.wiki.trimmed_end(content, inner_start, close);

        // The label is what follows the `|`, or the target when nothing
        // does. Its whitespace is trimmed further down: until then these
        // are only the bounds it lies within.
        let label_part = pipe.filter(|pipe| pipe + 1 < inner_end);
        let (label_from, label_to) = match label_part {
            Some(pipe) => (pipe + 1, inner_end),
            None => (inner_start, pipe.unwrap_or(inner_end)),
        };
        let has_bracket = run.wiki.has_bracket(bytes, label_from, label_to);

        // A label that parses to a link leaves the brackets literal. Once
        // candidates are being settled together (see
        // `settle_open_bracket`), a link found in front of this `]]` counts
        // against every candidate whose label reaches back past it.
        if has_bracket
            && let Some((rejected, link_start)) = run.wiki.rejected
            && rejected == close
            && label_from <= link_start
            && link_start < label_to
        {
            return Ok(false);
        }

        let target_end = match pipe {
            Some(pipe) => inner_start + content[inner_start..pipe].trim_end().len(),
            None => inner_end,
        };
        let target = &content[inner_start..target_end];
        let (label_start, label_end) = match label_part {
            Some(pipe) => (inner_end - content[pipe + 1..inner_end].trim_start().len(), inner_end),
            None => (inner_start, target_end),
        };
        let label = &content[label_start..label_end];

        let end = close + 2;
        if has_bracket && self.holds_link(content, label_start, label_end) {
            return Ok(false);
        }
        if !has_bracket {
            let children = self.parse_inline(label, offset + label_start)?;
            run.children.push(Node::Link(self.allocator.boxed(Link {
                url: target,
                title: None,
                children,
                span: Span::new((offset + open) as u32, (offset + end) as u32),
            })));
            *pos = end;
            return Ok(true);
        }

        run.open.push(OpenBracket {
            open,
            text_start: label_start,
            text_end: label_end,
            end,
            url: target,
            title: None,
            child_index: run.children.len(),
            delimiter_index: run.delimiters.len(),
            outer_height: core::mem::take(&mut run.height),
            kind: BracketKind::Wiki { close },
        });
        *pos = label_start;
        Ok(true)
    }
}

fn scan_wiki_link_close(
    bytes: &[u8],
    mut cursor: usize,
    code_spans: &mut SmallVec<[(usize, usize); 4]>,
) -> Option<usize> {
    while cursor + 1 < bytes.len() {
        match bytes[cursor] {
            b'\\' => {
                cursor += 2;
            }
            b'`' => {
                if let Some(end) = Parser::closed_code_span_end(bytes, cursor) {
                    code_spans.push((cursor, end));
                    cursor = end;
                } else {
                    cursor = cursor.saturating_add(1);
                }
            }
            b']' if bytes[cursor + 1] == b']' => return Some(cursor),
            _ => cursor += 1,
        }
    }
    None
}
