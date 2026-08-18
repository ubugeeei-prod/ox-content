//! Inline code spans (CommonMark "Code spans").
//!
//! The opener is a run of backticks; the closer is the next run of exactly
//! the same length. Everything in between is literal, so this is one of the
//! few inline constructs that can be skipped over in bulk rather than
//! walked byte by byte.

use memchr::memchr2;
use ox_content_allocator::Vec;
use ox_content_ast::{Node, Span};

use super::super::Parser;
#[allow(unused_imports)]
use crate::profile_span_detail;

impl<'a> Parser<'a> {
    pub(super) fn parse_inline_code(
        &self,
        content: &'a str,
        offset: usize,
        children: &mut Vec<'a, Node<'a>>,
        pos: &mut usize,
    ) {
        profile_span_detail!("parser::inline_code_span");
        let bytes = content.as_bytes();
        let open_len = Self::marker_run_len(bytes, *pos, b'`');
        let code_start = *pos + open_len;

        // The closer is the next backtick run of exactly the opener's
        // length (CommonMark "Code spans"). memchr jumps between runs, and
        // searching for the newline in the same pass answers the question
        // `normalize_code_span` would otherwise re-scan the whole span for.
        let mut cursor = code_start;
        let mut has_newline = false;
        while cursor < bytes.len() {
            let Some(off) = memchr2(b'`', b'\n', &bytes[cursor..]) else {
                break;
            };
            cursor += off;
            if bytes[cursor] == b'\n' {
                has_newline = true;
                cursor += 1;
                continue;
            }
            let run = Self::marker_run_len(bytes, cursor, b'`');
            if run == open_len {
                let span = Span::new((offset + *pos) as u32, (offset + cursor + run) as u32);
                children.push(Node::InlineCode(ox_content_ast::InlineCode {
                    value: self.normalize_code_span(&content[code_start..cursor], has_newline),
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
    ///
    /// `has_newline` comes from the closer scan, which already walked these
    /// bytes; re-deriving it here cost 1-3% of a parse because almost every
    /// code span is single-line and paid a whole extra scan to prove it.
    fn normalize_code_span(&self, raw: &'a str, has_newline: bool) -> &'a str {
        let value: &'a str = if has_newline {
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
