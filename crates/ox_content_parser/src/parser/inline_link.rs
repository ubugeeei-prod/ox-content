//! Inline link parsing.
//!
//! Split out of `inline_helpers` because CommonMark's "a link may not
//! contain a link" rule makes this the one inline construct that cannot
//! decide what it is from its own syntax: the answer depends on what its
//! text parses to. See `inline/brackets.rs` for how that is settled
//! without parsing the text more than once.

use memchr::memchr;
use ox_content_ast::{Link, Node, Span};

use super::Parser;
use super::inline::{BracketKind, InlineRun, OpenBracket};
use crate::error::ParseResult;
#[allow(unused_imports)]
use crate::profile_span_detail;

/// What follows the `]` of a bracket that can be a link or an image: where
/// it points, and where the whole construct ends.
pub(super) struct LinkTail<'a> {
    pub url: &'a str,
    pub title: Option<&'a str>,
    /// Byte index just past the construct.
    pub end: usize,
}

impl<'a> Parser<'a> {
    /// Handles the `[` at `pos`, an index into `content`.
    pub(super) fn parse_link(
        &self,
        run: &mut InlineRun<'a>,
        content: &'a str,
        offset: usize,
        pos: &mut usize,
    ) -> ParseResult<()> {
        profile_span_detail!("parser::inline_link");
        // Everything below reads the slice the walk is confined to — the
        // text of the innermost open bracket, or all of `content` — so a
        // bracket nested in link text is judged exactly as it was when
        // that text was parsed on its own.
        let (base, limit) = run.window(content.len());
        let view = &content[base..limit];
        let view_offset = offset + base;
        let bytes = view.as_bytes();
        let link_start = *pos - base;

        // `[^label]` is a footnote reference when the extension is on and
        // a definition exists; otherwise it falls through to normal link
        // handling and may still be a link label.
        if self.options.footnotes && bytes.get(link_start + 1) == Some(&b'^') {
            let mut relative = link_start;
            if self.try_parse_footnote_reference(
                view,
                view_offset,
                &mut run.children,
                &mut relative,
            ) {
                *pos = base + relative;
                return Ok(());
            }
        }

        // Every accepting branch below needs a `]`, and the balanced scan
        // only reports that there is none after walking to the end of the
        // content — once per bracket, which is quadratic over a run of
        // them. Settle it for the whole slice first.
        if !self.has_closer_from(view, link_start + 1, b']') {
            Self::push_text(&mut run.children, "[", offset + *pos, offset + *pos + 1);
            *pos += 1;
            return Ok(());
        }

        if self.options.wiki_links
            && bytes.get(link_start + 1) == Some(&b'[')
            && self.try_parse_wiki_link(run, content, offset, pos)?
        {
            return Ok(());
        }

        let text_start = link_start + 1;
        let close = self.matching_bracket(view, text_start);
        // Text an earlier pass found a link in keeps the bracket literal.
        if close < view.len() && !self.holds_link(view, text_start, close) {
            let link_text = &view[text_start..close];
            // What follows the `]` is checked before the text is looked at:
            // most brackets in prose are not followed by anything that
            // makes them a link, and those stay literal without their text
            // ever being parsed as link text.
            if let Some(tail) = self.link_tail(view, close, link_text) {
                // Links may not contain other links. Text with no `[` in
                // it cannot hold one, so it is parsed directly; anything
                // else is parsed in place with the bracket left open.
                if memchr(b'[', link_text.as_bytes()).is_none() {
                    let children = self.parse_inline(link_text, view_offset + text_start)?;
                    run.children.push(Node::Link(self.allocator.boxed(Link {
                        url: tail.url,
                        title: tail.title,
                        children,
                        span: Span::new((offset + *pos) as u32, (view_offset + tail.end) as u32),
                    })));
                    *pos = base + tail.end;
                    return Ok(());
                }

                run.open.push(OpenBracket {
                    open: *pos,
                    text_start: base + text_start,
                    text_end: base + close,
                    end: base + tail.end,
                    url: tail.url,
                    title: tail.title,
                    child_index: run.children.len(),
                    delimiter_index: run.delimiters.len(),
                    outer_height: core::mem::take(&mut run.height),
                    kind: BracketKind::Link,
                });
            }
        }

        // The bracket itself is literal text — for good when nothing can
        // make it a link, and until its text has been read otherwise.
        Self::push_text(&mut run.children, "[", offset + *pos, offset + *pos + 1);
        *pos += 1;
        Ok(())
    }

    /// Reads what follows the `]` at `close`: an inline `(dest "title")`,
    /// a full `[label]` or collapsed `[]` reference, or nothing at all when
    /// `text` is itself a defined label (the shortcut form).
    pub(super) fn link_tail(
        &self,
        content: &'a str,
        close: usize,
        text: &str,
    ) -> Option<LinkTail<'a>> {
        let bytes = content.as_bytes();

        if bytes.get(close + 1) == Some(&b'(')
            && let Some(target) = self.parse_link_target(content, close + 1)
        {
            return Some(LinkTail { url: target.url, title: target.title, end: target.end });
        }

        let mut well_formed_reference = false;
        if bytes.get(close + 1) == Some(&b'[') && self.has_closer_from(content, close + 2, b']') {
            let label_start = close + 2;
            let label_end = self.matching_bracket(content, label_start);
            if label_end < content.len() {
                well_formed_reference = true;
                let raw_label = &content[label_start..label_end];
                let key = if raw_label.trim().is_empty() { text } else { raw_label };
                if let Some(reference) = self.lookup_reference(key) {
                    return Some(LinkTail {
                        url: reference.url,
                        title: reference.title,
                        end: label_end + 1,
                    });
                }
            }
        }

        // Shortcut form: [label]. Suppressed when an explicit (but
        // unknown) [label] followed, which must stay literal.
        if !well_formed_reference && let Some(reference) = self.lookup_reference(text) {
            return Some(LinkTail { url: reference.url, title: reference.title, end: close + 1 });
        }
        None
    }
}
