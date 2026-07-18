use ox_content_allocator::Vec;
use ox_content_ast::{Node, Paragraph, Span};

use super::super::Parser;
use crate::error::ParseResult;

impl<'a> Parser<'a> {
    /// Builds the synthetic source used when a list item needs block parsing.
    ///
    /// Simple single-line list items take `parse_inline_list_item_children`
    /// instead and never allocate this buffer. When continuation lines, nested
    /// blocks, or block-looking inline text require the historical sub-parser
    /// path, the buffer is allocated directly in the bump arena so ownership
    /// matches the resulting AST and we avoid a system `String` followed by an
    /// arena copy.
    pub(super) fn init_list_item_source(
        &self,
        content: &'a str,
        consumed_newline: bool,
    ) -> ox_content_allocator::String<'a> {
        // Bump-allocate the per-item buffer so we don't go System → arena.
        // This is now only paid for list items that actually need block
        // parsing: multi-line items, nested blocks, or block-looking
        // single-line contents such as `# heading`.
        let mut source = ox_content_allocator::String::with_capacity_in(
            content.len() + usize::from(consumed_newline),
            self.allocator.bump(),
        );
        source.push_str(content);
        if consumed_newline {
            source.push('\n');
        }
        source
    }

    /// Returns true when a single-line list item can bypass the sub-parser.
    ///
    /// A list item like `- hello **world**` has exactly one paragraph child in
    /// the old implementation: the sub-parser parsed the item source as a
    /// document, then offset the paragraph spans back into the parent source.
    /// Reconstructing that paragraph directly removes a parser allocation and
    /// a second block-dispatch pass. Items whose first non-space byte can open
    /// a block stay on the old path so cases such as `- # heading`, nested
    /// lists, fenced code, thematic breaks, and HTML blocks keep their AST.
    pub(super) fn can_inline_parse_list_item(content: &str) -> bool {
        // Leading indentation of four or more columns means the item
        // starts with indented code — that needs the block sub-parser.
        let mut columns = 0usize;
        for byte in content.bytes() {
            match byte {
                b' ' => columns += 1,
                b'\t' => columns = (columns / 4 + 1) * 4,
                _ => break,
            }
        }
        if columns >= 4 {
            return false;
        }
        let Some(&first) = content.trim_start().as_bytes().first() else {
            return true;
        };

        // These are the same leading-byte families that `parse_block` may
        // treat as block syntax in a freshly spawned sub-parser. Keep those
        // on the old path so `- # heading`, nested lists, fenced code, raw
        // HTML blocks, etc. preserve their current AST.
        !matches!(first, b'#' | b'-' | b'*' | b'_' | b'>' | b'`' | b'~' | b'<' | b'+' | b'0'..=b'9')
    }

    /// Creates the direct AST for the single-paragraph list-item fast path.
    ///
    /// `content_offset` is the byte position of `content` in the original
    /// document, so inline spans are produced with their final coordinates on
    /// the first parse. `item_end` remains the list-item line end to preserve
    /// the paragraph span that callers observed when this went through the
    /// sub-parser.
    pub(super) fn parse_inline_list_item_children(
        &self,
        content: &'a str,
        content_offset: usize,
        item_end: usize,
    ) -> ParseResult<Vec<'a, Node<'a>>> {
        let mut children = self.allocator.new_vec();
        let inline = content.trim();
        if inline.is_empty() {
            return Ok(children);
        }

        let paragraph_children = self.parse_inline(inline, content_offset)?;
        children.push(Node::Paragraph(Paragraph {
            children: paragraph_children,
            span: Span::new(content_offset as u32, item_end as u32),
        }));
        Ok(children)
    }
}
