use ox_content_allocator::Vec;
use ox_content_ast::{Node, Span};

use super::Parser;
use crate::error::ParseResult;
#[allow(unused_imports)]
use crate::{profile_span, profile_span_detail};

mod angle;
mod autolink;
mod bracket_match;
mod brackets;
mod code_runs;
mod code_span;
mod emphasis;
mod entity;
mod gfm_autolink;
mod line_break;
mod link_target;
mod marker_scan;
mod nesting;
mod scan;
mod script_span;
mod smart_punctuation;
mod strikethrough;
mod wiki_link;

pub(in crate::parser) use self::bracket_match::BracketMatches;
pub(in crate::parser) use self::brackets::{BracketKind, InlineRun, LinkText, OpenBracket};
pub(in crate::parser) use self::code_runs::CodeRunMemo;
use self::marker_scan::InlineMarkerScan;
use self::script_span::same_marker_neighbor;
use super::line_scan::{is_line_ending_byte, line_terminator_end};

pub(in crate::parser) use self::autolink::autolink_end;
pub(in crate::parser) use self::link_target::{
    parse_destination as parse_link_destination, parse_title as parse_link_title,
};

impl<'a> Parser<'a> {
    /// Parses the inline content of a block-level construct (paragraph,
    /// heading, table cell, list item paragraph) and runs the block-scoped
    /// post-passes on the result — today, the GFM autolink rewrite.
    ///
    /// Nested inline contexts (link text, image alt, strikethrough
    /// interiors) call [`Self::parse_inline`] directly instead: the autolink
    /// pass itself recurses through emphasis-like containers, so running it
    /// per nested sequence both re-scanned the same nodes and — for link
    /// text, which GFM excludes from autolinking — made nested `<a>`s.
    pub(super) fn parse_inline_block(
        &self,
        content: &'a str,
        offset: usize,
    ) -> ParseResult<Vec<'a, Node<'a>>> {
        let mut children = self.parse_inline(content, offset)?;
        let scan =
            self.options.autolinks.then(|| gfm_autolink::may_contain_autolink(content)).flatten();
        if let Some(scan) = scan {
            self.apply_gfm_autolinks(&mut children, scan);
        }
        if self.options.smart_punctuation {
            self.apply_smart_punctuation(&mut children);
        }
        Ok(children)
    }

    fn allows_mdx_text_expression(&self) -> bool {
        self.options.mdx
    }

    pub(super) fn parse_inline(
        &self,
        content: &'a str,
        offset: usize,
    ) -> ParseResult<Vec<'a, Node<'a>>> {
        profile_span!("parser::parse_inline");
        if self.top_level_only {
            return Ok(self.allocator.new_vec());
        }
        let bytes = content.as_bytes();
        let mut markers = InlineMarkerScan::new(
            self.allows_mdx_text_expression(),
            self.options.superscript,
            self.options.math,
        );
        let first_special = markers.next(bytes, 0);

        // Plain text is both the most common inline shape and exactly one AST
        // node. Reserving the general four-node floor here wasted three
        // full Node slots for every prose block and plain table cell. The
        // scan is required by the normal loop anyway, so use its no-marker
        // result to build the exact one-slot representation and return.
        if first_special == content.len() {
            if content.is_empty() {
                return Ok(self.allocator.new_vec());
            }
            let mut children = self.allocator.new_vec_with_capacity(1);
            Self::push_text(&mut children, content, offset, offset + content.len());
            return Ok(children);
        }

        // Only content that can nest is counted, so prose never pays for it.
        let span = Span::new(offset as u32, (offset + content.len()) as u32);
        let collected = self.enter_inline(span)?;
        let mut run = InlineRun::new(self, content.len());
        let height = self.parse_inline_run(&mut run, content, offset, markers, first_special);
        self.leave_inline(collected, height, span)?;
        Ok(run.children)
    }

    /// Walks `content`, returning the height of the tree left in `run`.
    fn parse_inline_run(
        &self,
        run: &mut InlineRun<'a>,
        content: &'a str,
        offset: usize,
        mut markers: InlineMarkerScan,
        first_special: usize,
    ) -> ParseResult<u32> {
        let bytes = content.as_bytes();
        let mut pos = 0;
        let mut first_scan = Some(first_special);

        loop {
            // Inside an open bracket the walk reads its text as a slice of
            // its own, so nothing in it can see past either end.
            let (base, limit) = run.window(content.len());
            if pos >= limit {
                if run.open.is_empty() {
                    break;
                }
                let link_start = run.open.last().map_or(pos, |bracket| bracket.open);
                self.close_bracket(run, offset, &mut pos)?;
                if !run.open.is_empty() {
                    self.settle_open_bracket(run, content, link_start, &mut pos);
                }
                continue;
            }

            let start = pos;
            // Plain text is the common inline case. Jump over bytes that
            // cannot start any inline construct, then push that entire run as
            // one Text node. This keeps the parser on bulk byte scans for
            // prose and only enters the slower match when a real marker byte
            // has been reached.
            pos = first_scan.take().unwrap_or_else(|| markers.next(bytes, pos)).min(limit);

            // Fold soft line breaks into the running text node. A newline
            // with non-whitespace on both sides is a soft break with nothing
            // to strip, so its rendered form is the literal `\n` already
            // inside the source run — emitting `"line"`, `"\n"`, `"next"` as
            // three nodes only slowed every later pass. This is also the
            // shape remark produces (mdast has no softbreak node; line
            // endings live inside `text` values). Prose is dominated by this
            // case; a newline touching spaces or tabs still takes
            // `parse_line_break` below for hard-break detection and
            // whitespace stripping.
            while pos > start
                && pos + 1 < limit
                && bytes[pos] == b'\n'
                && !matches!(bytes[pos - 1], b' ' | b'\t')
                && !matches!(bytes[pos + 1], b' ' | b'\t' | b'\n')
            {
                pos = markers.next(bytes, pos + 1).min(limit);
            }

            if pos > start {
                Self::push_text(
                    &mut run.children,
                    &content[start..pos],
                    offset + start,
                    offset + pos,
                );
            }
            if pos >= limit {
                continue;
            }

            let special = pos;
            let pushed = run.children.len();
            if bytes[pos] == b'[' {
                self.parse_link(run, content, offset, &mut pos)?;
            } else {
                let mut relative = pos - base;
                self.parse_inline_special(
                    &content[base..limit],
                    offset + base,
                    &mut run.children,
                    &mut run.delimiters,
                    &mut relative,
                )?;
                pos = base + relative;
            }
            run.height = run.height.max(self.take_inline_height());

            if !run.open.is_empty()
                && run.children.get(pushed..).is_some_and(brackets::contains_link)
            {
                self.settle_open_bracket(run, content, special, &mut pos);
            }
        }

        let emphasis = if run.delimiters.is_empty() {
            0
        } else {
            self.process_emphasis(&mut run.children, &mut run.delimiters)?
        };
        Ok(run.height.saturating_add(emphasis))
    }

    fn parse_inline_special(
        &self,
        content: &'a str,
        offset: usize,
        children: &mut Vec<'a, Node<'a>>,
        delimiters: &mut Vec<'a, emphasis::Delimiter>,
        pos: &mut usize,
    ) -> ParseResult<()> {
        let bytes = content.as_bytes();
        match bytes[*pos] {
            b'\\' if *pos + 1 < content.len() && is_line_ending_byte(bytes[*pos + 1]) => {
                let end = line_terminator_end(bytes, *pos + 1);
                let span = Span::new((offset + *pos) as u32, (offset + end) as u32);
                children.push(Node::Break(ox_content_ast::Break { span }));
                *pos = end;
                // Leading whitespace of the next line is not content.
                while *pos < content.len() && matches!(bytes[*pos], b' ' | b'\t') {
                    *pos += 1;
                }
            }
            byte if is_line_ending_byte(byte) => {
                Self::parse_line_break(content, offset, children, pos);
            }
            b'&' => {
                profile_span_detail!("parser::inline_entity");
                // Entity / numeric character references decode to literal
                // text (the result can never open or close markup).
                if let Some((value, len)) = entity::scan_entity(&content[*pos..]) {
                    let end = *pos + len;
                    let text: &'a str = match value {
                        entity::EntityValue::Named(expansion) => expansion,
                        entity::EntityValue::Char(ch) => {
                            let mut buf = [0u8; 4];
                            self.allocator.alloc_str(ch.encode_utf8(&mut buf))
                        }
                    };
                    Self::push_text(children, text, offset + *pos, offset + end);
                    *pos = end;
                } else {
                    Self::push_text(children, "&", offset + *pos, offset + *pos + 1);
                    *pos += 1;
                }
            }
            b'{' if self.allows_mdx_text_expression() => {
                if let Some((node, end)) = self.try_parse_mdx_text_expression(content, *pos, offset)
                {
                    children.push(node);
                    *pos = end;
                } else {
                    Self::push_text(children, "{", offset + *pos, offset + *pos + 1);
                    *pos += 1;
                }
            }
            b'<' => self.parse_inline_html_or_text(content, offset, children, pos)?,
            b'\\' if *pos + 1 < content.len() && bytes[*pos + 1].is_ascii_punctuation() => {
                // A backslash escapes only ASCII punctuation (CommonMark
                // "Backslash escapes"). The escaped character is emitted as
                // literal text so it can't open any inline construct.
                *pos += 1;
                let span_start = offset + *pos - 1;
                Self::push_text(children, &content[*pos..*pos + 1], span_start, offset + *pos + 1);
                *pos += 1;
            }
            b'\\' => {
                // Backslash before anything else (letters, digits, spaces,
                // multibyte characters, or end of input) is a literal
                // backslash; the following character is parsed normally.
                Self::push_text(children, "\\", offset + *pos, offset + *pos + 1);
                *pos += 1;
            }
            b'~' if self.options.strikethrough
                && *pos + 1 < content.len()
                && bytes[*pos + 1] == b'~' =>
            {
                self.parse_strikethrough(content, offset, children, pos)?;
            }
            b'~' if self.options.subscript && !same_marker_neighbor(bytes, *pos, b'~') => {
                self.parse_subscript_span(content, offset, children, pos)?;
            }
            b'^' if self.options.superscript && !same_marker_neighbor(bytes, *pos, b'^') => {
                self.parse_superscript_span(content, offset, children, pos)?;
            }
            b'$' if self.options.math => {
                self.parse_inline_math(content, offset, children, pos);
            }
            b'*' | b'_' => {
                self.push_delimiter_run(content, offset, children, delimiters, pos);
            }
            b'`' => self.parse_inline_code(content, offset, children, pos),
            b'!' => self.parse_image(content, offset, children, pos)?,
            _ => {
                Self::push_text(
                    children,
                    &content[*pos..*pos + 1],
                    offset + *pos,
                    offset + *pos + 1,
                );
                *pos += 1;
            }
        }
        Ok(())
    }
}
