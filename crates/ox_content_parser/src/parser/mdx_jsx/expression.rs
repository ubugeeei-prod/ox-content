//! MDX `{expression}` parse: flow, text, JSX comments, and prose braces.
//!
//! Source inside the braces is stored. Nothing is evaluated.

use ox_content_ast::{MdxFlowExpression, MdxTextExpression, Node, Span};

use super::Parser;
use super::braces;
use super::scan;
use crate::error::ParseResult;

pub(super) fn looks_like_flow_expression(source: &str, at: usize) -> bool {
    let Some((_, brace_end)) = braces::scan_balanced_braces(source, at) else {
        return false;
    };
    scan::only_ws_until_eol(source.as_bytes(), brace_end)
}

impl<'a> Parser<'a> {
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
        let Some((value, brace_end)) = braces::scan_balanced_braces(self.source, trimmed_start)
        else {
            return Ok(None);
        };
        if !scan::only_ws_until_eol(self.source.as_bytes(), brace_end) {
            return Ok(None);
        }

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
        let (value, end) = braces::scan_balanced_braces(content, pos)?;
        Some((
            Node::MdxTextExpression(MdxTextExpression {
                value,
                span: Span::new((offset + pos) as u32, (offset + end) as u32),
            }),
            end,
        ))
    }
}
