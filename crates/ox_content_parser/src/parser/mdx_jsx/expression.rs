//! MDX `{expression}` parse: flow, text, JSX comments, and prose braces.
//!
//! Source inside the braces is stored. Nothing is evaluated.

use ox_content_ast::{MdxFlowExpression, MdxTextExpression, Node, Span};

use super::Parser;
use super::scan;
use crate::error::ParseResult;

impl<'a> Parser<'a> {
    /// Reports whether a flow `{expression}` starts at `at`: a balanced
    /// brace group followed only by line whitespace.
    ///
    /// The paragraph loop asks this for every continuation line that starts
    /// with `{`, so the answer has to come from the shared scan cache; a
    /// direct scan walked the rest of the source for each such line.
    pub(in crate::parser) fn mdx_flow_expression_starts_at(&self, at: usize) -> bool {
        self.mdx_flow_expression_end(at).is_some()
    }

    /// Parses a block `{expression}` or `{/* comment */}` at the current line.
    ///
    /// Succeeds only when the closing `}` is followed by line whitespace.
    /// On failure the cursor is left unchanged.
    pub(in crate::parser) fn try_parse_mdx_flow_expression(
        &mut self,
        start: usize,
        trimmed_start: usize,
    ) -> ParseResult<Option<Node<'a>>> {
        if !self.options.mdx {
            return Ok(None);
        }
        // Same bound as the block-start check: without it every line that
        // opens a brace and never closes it walks the rest of the source.
        let source = self.source;
        if !self.has_closer_from(source, trimmed_start + 1, b'}') {
            return Ok(None);
        }
        let Some((value, brace_end)) = self.mdx_flow_expression_end(trimmed_start) else {
            return Ok(None);
        };

        self.position = scan::after_trailing_line_ws(self.source.as_bytes(), brace_end);
        Ok(Some(Node::MdxFlowExpression(MdxFlowExpression {
            value,
            span: Span::new(start as u32, self.position as u32),
        })))
    }

    /// Parses an inline `{expression}` or `{/* comment */}` at `pos`.
    ///
    /// Used for document-level prose and for JSX children.
    pub(in crate::parser) fn try_parse_mdx_text_expression(
        &self,
        content: &'a str,
        pos: usize,
        offset: usize,
    ) -> Option<(Node<'a>, usize)> {
        if !self.options.mdx {
            return None;
        }
        // `skip_braces` only reports that nothing closed after walking to
        // the end of the content, so a run of unclosed braces would pay one
        // walk each.
        if !self.has_closer_from(content, pos + 1, b'}') {
            return None;
        }
        let (value, end) = self.scan_mdx_braces(content, pos)?;
        Some((
            Node::MdxTextExpression(MdxTextExpression {
                value,
                span: Span::new((offset + pos) as u32, (offset + end) as u32),
            }),
            end,
        ))
    }

    /// Inner source and end of the flow expression that starts at `at`.
    fn mdx_flow_expression_end(&self, at: usize) -> Option<(&'a str, usize)> {
        let source = self.source;
        self.scan_mdx_braces(source, at)
            .filter(|(_, end)| scan::only_ws_until_eol(source.as_bytes(), *end))
    }
}
