//! GFM strikethrough: `~~text~~`.

use memchr::memchr;
use ox_content_allocator::Vec;
use ox_content_ast::{Node, Span};

use crate::error::ParseResult;
use crate::parser::Parser;
#[allow(unused_imports)]
use crate::profile_span_detail;

impl<'a> Parser<'a> {
    pub(super) fn parse_strikethrough(
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
