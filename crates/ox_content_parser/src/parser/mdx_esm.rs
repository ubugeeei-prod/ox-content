//! MDX ESM parse: block-level `import` / `export`.

use ox_content_ast::{MdxjsEsm, Node, Span};

use super::Parser;
use crate::error::ParseResult;

mod scan;

#[cfg(test)]
mod tests;

pub(super) fn looks_like_esm(source: &str, at: usize) -> bool {
    scan::scan_esm(source, at).is_some()
}

impl<'a> Parser<'a> {
    /// Parses a block-level `import` / `export` starting at the current line.
    ///
    /// On failure the cursor is left unchanged so paragraph dispatch can run.
    /// The statement source is stored on the node; nothing is executed or read
    /// from the filesystem.
    pub(super) fn try_parse_mdx_esm(
        &mut self,
        start: usize,
        trimmed_start: usize,
    ) -> ParseResult<Option<Node<'a>>> {
        if !self.options.mdx {
            return Ok(None);
        }
        let Some(scanned) = scan::scan_esm(self.source, trimmed_start) else {
            return Ok(None);
        };

        self.position = scanned.end;
        Ok(Some(Node::MdxjsEsm(MdxjsEsm {
            value: scanned.value,
            span: Span::new(start as u32, scanned.end as u32),
        })))
    }
}
