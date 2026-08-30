//! MDX JSX parse: PascalCase / member names, fragments, spreads, expressions.

use ox_content_ast::{MdxJsxFlowElement, MdxJsxTextElement, Node, Span};

use super::Parser;
use crate::error::ParseResult;

mod braces;
mod children;
mod expression;
mod scan;

pub(super) fn looks_like_jsx_open(bytes: &[u8], at: usize) -> bool {
    scan::looks_like_jsx_open(bytes, at)
}

pub(super) fn looks_like_flow_expression(source: &str, at: usize) -> bool {
    expression::looks_like_flow_expression(source, at)
}

impl<'a> Parser<'a> {
    /// Parses a flow JSX element starting at the current line.
    ///
    /// On failure the cursor is left unchanged so HTML / paragraph dispatch
    /// can run. Unclosed tags do not panic or emit a half-parsed node.
    pub(super) fn try_parse_mdx_jsx_flow(
        &mut self,
        start: usize,
        trimmed_start: usize,
    ) -> ParseResult<Option<Node<'a>>> {
        if !self.options.mdx || !scan::looks_like_jsx_open(self.source.as_bytes(), trimmed_start) {
            return Ok(None);
        }

        let mut attributes = self.allocator.new_vec();
        let Some(open) = scan::scan_jsx_open(self.source, trimmed_start, 0, &mut attributes) else {
            return Ok(None);
        };

        let (self_closing, children, element_end) = if open.self_closing {
            if !scan::only_ws_until_eol(self.source.as_bytes(), open.end) {
                return Ok(None);
            }
            (true, self.allocator.new_vec(), open.end)
        } else {
            let Some((close_start, close_end)) =
                scan::find_matching_close(self.source, open.end, open.name)
            else {
                return Ok(None);
            };
            let children = self.parse_jsx_flow_children(open.end, close_start)?;
            (false, children, close_end)
        };

        self.position = scan::after_trailing_line_ws(self.source.as_bytes(), element_end);
        Ok(Some(Node::MdxJsxFlowElement(self.allocator.boxed(MdxJsxFlowElement {
            name: open.name,
            attributes,
            children,
            self_closing,
            span: Span::new(start as u32, self.position as u32),
        }))))
    }

    /// Parses a text JSX element at `pos` inside inline `content`.
    pub(super) fn try_parse_mdx_jsx_text(
        &self,
        content: &'a str,
        pos: usize,
        offset: usize,
    ) -> ParseResult<Option<(Node<'a>, usize)>> {
        if !self.options.mdx || !scan::looks_like_jsx_open(content.as_bytes(), pos) {
            return Ok(None);
        }

        let mut attributes = self.allocator.new_vec();
        let Some(open) = scan::scan_jsx_open(content, pos, offset, &mut attributes) else {
            return Ok(None);
        };

        let (self_closing, children, end) = if open.self_closing {
            (true, self.allocator.new_vec(), open.end)
        } else {
            let Some((close_start, close_end)) =
                scan::find_matching_close(content, open.end, open.name)
            else {
                return Ok(None);
            };
            let children =
                self.parse_jsx_phrasing(&content[open.end..close_start], offset + open.end)?;
            (false, children, close_end)
        };

        Ok(Some((
            Node::MdxJsxTextElement(self.allocator.boxed(MdxJsxTextElement {
                name: open.name,
                attributes,
                children,
                self_closing,
                span: Span::new((offset + pos) as u32, (offset + end) as u32),
            })),
            end,
        )))
    }

    fn parse_jsx_phrasing(
        &self,
        content: &'a str,
        offset: usize,
    ) -> ParseResult<ox_content_allocator::Vec<'a, Node<'a>>> {
        self.parse_inline(content, offset)
    }

    fn parse_jsx_flow_children(
        &self,
        inner_start: usize,
        inner_end: usize,
    ) -> ParseResult<ox_content_allocator::Vec<'a, Node<'a>>> {
        if inner_start >= inner_end || self.source[inner_start..inner_end].trim().is_empty() {
            return Ok(self.allocator.new_vec());
        }
        let inner = &self.source[inner_start..inner_end];
        let child_source = children::normalize_indentation(self.allocator, inner);
        let sub =
            self.sub_parser_with_lazy_lines(child_source.source, rustc_hash::FxHashSet::default());
        let mut children = sub.parse()?.children;
        for child in &mut children {
            if let Some(offsets) = &child_source.offsets {
                children::remap_node_spans(child, inner_start as u32, offsets);
            } else {
                Self::offset_node_spans(child, inner_start as u32);
            }
        }
        Ok(children)
    }
}
