use memchr::memchr;
use ox_content_allocator::Vec;
use ox_content_ast::{Node, Span};

use super::Parser;
use crate::error::ParseResult;
#[allow(unused_imports)]
use crate::profile_span;

mod autolink;
mod emphasis;
mod entity;
mod link_target;
mod scan;

use self::scan::next_inline_special;

pub(in crate::parser) use self::autolink::autolink_end;
pub(in crate::parser) use self::link_target::{
    parse_destination as parse_link_destination, parse_title as parse_link_title,
};

impl<'a> Parser<'a> {
    pub(super) fn parse_inline(
        &self,
        content: &'a str,
        offset: usize,
    ) -> ParseResult<Vec<'a, Node<'a>>> {
        profile_span!("parser::parse_inline");
        let mut children = self.allocator.new_vec();
        let mut delimiters = self.allocator.new_vec();
        let mut pos = 0;
        let bytes = content.as_bytes();

        while pos < content.len() {
            let start = pos;
            // Plain text is the common inline case. Jump over bytes that
            // cannot start any inline construct, then push that entire run as
            // one Text node. This keeps the parser on bulk byte scans for
            // prose and only enters the slower match when a real marker byte
            // has been reached.
            pos = next_inline_special(bytes, pos);

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
                Self::push_delimiter_run(content, offset, children, delimiters, pos);
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
                children
                    .push(Node::Delete(ox_content_ast::Delete { children: inner_children, span }));
                *pos = inner_end + 2;
                return Ok(());
            }
            inner_end += 1;
        }

        Self::push_text(children, &content[*pos..*pos + 2], offset + *pos, offset + *pos + 2);
        *pos += 2;
        Ok(())
    }

    fn parse_inline_code(
        &self,
        content: &'a str,
        offset: usize,
        children: &mut Vec<'a, Node<'a>>,
        pos: &mut usize,
    ) {
        let bytes = content.as_bytes();
        let open_len = Self::marker_run_len(bytes, *pos, b'`');
        let code_start = *pos + open_len;

        // The closer is the next backtick run of exactly the opener's
        // length (CommonMark "Code spans"). memchr jumps between runs.
        let mut cursor = code_start;
        while cursor < bytes.len() {
            let Some(off) = memchr(b'`', &bytes[cursor..]) else {
                break;
            };
            cursor += off;
            let run = Self::marker_run_len(bytes, cursor, b'`');
            if run == open_len {
                let span = Span::new((offset + *pos) as u32, (offset + cursor + run) as u32);
                children.push(Node::InlineCode(ox_content_ast::InlineCode {
                    value: self.normalize_code_span(&content[code_start..cursor]),
                    span,
                }));
                *pos = cursor + run;
                return;
            }
            cursor += run;
        }

        // No closer: the opening run is literal text.
        Self::push_text(children, &content[*pos..code_start], offset + *pos, offset + code_start);
        *pos = code_start;
    }

    /// Applies the code span content rules: line endings become spaces,
    /// and one leading plus one trailing space is dropped when the content
    /// starts and ends with a space without being all spaces.
    fn normalize_code_span(&self, raw: &'a str) -> &'a str {
        let value: &'a str = if raw.contains('\n') {
            let mut converted = self.allocator.new_string();
            for ch in raw.chars() {
                converted.push(if ch == '\n' { ' ' } else { ch });
            }
            converted.into_bump_str()
        } else {
            raw
        };

        let stripped = value.starts_with(' ')
            && value.ends_with(' ')
            && value.len() >= 2
            && value.bytes().any(|byte| byte != b' ');
        if stripped {
            &value[1..value.len() - 1]
        } else {
            value
        }
    }
}
