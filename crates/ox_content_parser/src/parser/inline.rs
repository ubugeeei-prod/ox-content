use memchr::memchr;
use ox_content_allocator::Vec;
use ox_content_ast::{Node, Span};

use super::Parser;
use crate::error::ParseResult;
#[allow(unused_imports)]
use crate::{profile_span, profile_span_detail};

mod autolink;
mod code_span;
mod emphasis;
mod entity;
mod gfm_autolink;
mod link_target;
mod scan;

use self::scan::next_inline_special;

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
        Ok(children)
    }

    pub(super) fn parse_inline(
        &self,
        content: &'a str,
        offset: usize,
    ) -> ParseResult<Vec<'a, Node<'a>>> {
        profile_span!("parser::parse_inline");
        let bytes = content.as_bytes();
        let first_special = next_inline_special(bytes, 0);

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

        let mut children =
            self.allocator.new_vec_with_capacity(Self::inline_children_capacity(content.len()));
        let mut delimiters = self.allocator.new_vec();
        let mut pos = 0;
        let mut first_scan = Some(first_special);

        while pos < content.len() {
            let start = pos;
            // Plain text is the common inline case. Jump over bytes that
            // cannot start any inline construct, then push that entire run as
            // one Text node. This keeps the parser on bulk byte scans for
            // prose and only enters the slower match when a real marker byte
            // has been reached.
            pos = first_scan.take().unwrap_or_else(|| next_inline_special(bytes, pos));

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
                && pos + 1 < content.len()
                && bytes[pos] == b'\n'
                && !matches!(bytes[pos - 1], b' ' | b'\t')
                && !matches!(bytes[pos + 1], b' ' | b'\t' | b'\n')
            {
                pos = next_inline_special(bytes, pos + 1);
            }

            if pos > start {
                Self::push_text(&mut children, &content[start..pos], offset + start, offset + pos);
            }
            if pos >= content.len() {
                break;
            }

            self.parse_inline_special(content, offset, &mut children, &mut delimiters, &mut pos)?;
        }

        if !delimiters.is_empty() {
            self.process_emphasis(&mut children, &mut delimiters);
        }
        Ok(children)
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
            b'\\' if *pos + 1 < content.len() && bytes[*pos + 1] == b'\n' => {
                let span = Span::new((offset + *pos) as u32, (offset + *pos + 2) as u32);
                children.push(Node::Break(ox_content_ast::Break { span }));
                *pos += 2;
                // Leading whitespace of the next line is not content.
                while *pos < content.len() && matches!(bytes[*pos], b' ' | b'\t') {
                    *pos += 1;
                }
            }
            b'\n' => Self::parse_line_break(content, offset, children, pos),
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
            b'<' => self.parse_inline_html_or_text(content, offset, children, pos),
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
            b'*' | b'_' => {
                self.push_delimiter_run(content, offset, children, delimiters, pos);
            }
            b'`' => self.parse_inline_code(content, offset, children, pos),
            b'[' => self.parse_link(content, offset, children, pos)?,
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

    /// Handles a newline inside inline content. Two or more trailing
    /// spaces on the previous line make a hard break; otherwise the
    /// newline is a soft break. Either way the spaces around the break —
    /// trailing on the previous line, leading on the next — are stripped
    /// (CommonMark "Hard line breaks" / "Soft line breaks").
    fn parse_line_break(
        content: &'a str,
        offset: usize,
        children: &mut Vec<'a, Node<'a>>,
        pos: &mut usize,
    ) {
        profile_span_detail!("parser::inline_line_break");
        let bytes = content.as_bytes();
        let mut hard = false;
        let mut trim_to = None;
        if let Some(Node::Text(text)) = children.last() {
            let trimmed_len = text.value.trim_end_matches(' ').len();
            if trimmed_len < text.value.len() {
                hard = text.value.len() - trimmed_len >= 2;
                trim_to = Some(trimmed_len);
            }
        }
        if let Some(new_len) = trim_to {
            if new_len == 0 {
                children.pop();
            } else if let Some(Node::Text(text)) = children.last_mut() {
                let removed = (text.value.len() - new_len) as u32;
                text.value = &text.value[..new_len];
                text.span = Span::new(text.span.start, text.span.end - removed);
            }
        }

        let newline_pos = *pos;
        *pos += 1;
        while *pos < content.len() && matches!(bytes[*pos], b' ' | b'\t') {
            *pos += 1;
        }

        let span = Span::new((offset + newline_pos) as u32, (offset + newline_pos + 1) as u32);
        if hard {
            children.push(Node::Break(ox_content_ast::Break { span }));
        } else {
            Self::push_text(children, "\n", offset + newline_pos, offset + newline_pos + 1);
        }
    }

    fn parse_inline_html_or_text(
        &self,
        content: &'a str,
        offset: usize,
        children: &mut Vec<'a, Node<'a>>,
        pos: &mut usize,
    ) {
        if let Some((link, end)) = self.parse_autolink(content, *pos, offset) {
            children.push(link);
            *pos = end;
        } else if let Some((html, end)) = Self::parse_inline_html(content, *pos, offset) {
            children.push(Node::Html(html));
            *pos = end;
        } else {
            Self::push_text(children, "<", offset + *pos, offset + *pos + 1);
            *pos += 1;
        }
    }

    fn parse_strikethrough(
        &self,
        content: &'a str,
        offset: usize,
        children: &mut Vec<'a, Node<'a>>,
        pos: &mut usize,
    ) -> ParseResult<()> {
        profile_span_detail!("parser::inline_strikethrough");
        let bytes = content.as_bytes();
        let inner_start = *pos + 2;
        let mut inner_end = inner_start;

        while inner_end + 1 < content.len() {
            // Restrict the scan to `..content.len() - 1` so any `~` memchr finds
            // has a valid `inner_end + 1` byte to test — preserving the original
            // `inner_end + 1 < content.len()` guard exactly.
            match memchr(b'~', &bytes[inner_end..content.len() - 1]) {
                Some(off) => inner_end += off,
                None => break,
            }
            if bytes[inner_end + 1] == b'~' {
                let inner_children =
                    self.parse_inline(&content[inner_start..inner_end], offset + inner_start)?;
                let span = Span::new((offset + *pos) as u32, (offset + inner_end + 2) as u32);
                children.push(Node::Delete(
                    self.allocator.boxed(ox_content_ast::Delete { children: inner_children, span }),
                ));
                *pos = inner_end + 2;
                return Ok(());
            }
            inner_end += 1;
        }

        Self::push_text(children, &content[*pos..*pos + 2], offset + *pos, offset + *pos + 2);
        *pos += 2;
        Ok(())
    }
}
