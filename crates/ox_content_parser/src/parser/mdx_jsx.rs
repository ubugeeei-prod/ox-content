//! MDX JSX parse: PascalCase / member names, fragments, spreads, expressions.

use ox_content_allocator::Vec;
use ox_content_ast::{MdxJsxFlowElement, MdxJsxTextElement, Node, Span};

use super::Parser;
use crate::error::ParseResult;

mod braces;
mod cache;
mod children;
mod close;
mod expression;
mod lex;
mod lex_memo;
mod scan;
mod tag_path;

pub(super) use cache::ScanScope;

pub(super) fn looks_like_jsx_open(bytes: &[u8], at: usize) -> bool {
    scan::looks_like_jsx_open(bytes, at)
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
        let Some((open, close)) =
            self.scan_jsx_element(self.source, trimmed_start, 0, &mut attributes)
        else {
            return Ok(None);
        };

        let (children, element_end) = if let Some((close_start, close_end)) = close {
            (self.parse_jsx_flow_children(open.end, close_start)?, close_end)
        } else {
            if !scan::only_ws_until_eol(self.source.as_bytes(), open.end) {
                return Ok(None);
            }
            (self.allocator.new_vec(), open.end)
        };

        self.position = scan::after_trailing_line_ws(self.source.as_bytes(), element_end);
        Ok(Some(Node::MdxJsxFlowElement(self.allocator.boxed(MdxJsxFlowElement {
            name: open.name,
            attributes,
            children,
            self_closing: open.self_closing,
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
        let Some((open, close)) = self.scan_jsx_element(content, pos, offset, &mut attributes)
        else {
            return Ok(None);
        };

        let (children, end) = if let Some((close_start, close_end)) = close {
            let inner = &content[open.end..close_start];
            (self.parse_jsx_phrasing(inner, offset + open.end)?, close_end)
        } else {
            (self.allocator.new_vec(), open.end)
        };

        Ok(Some((
            Node::MdxJsxTextElement(self.allocator.boxed(MdxJsxTextElement {
                name: open.name,
                attributes,
                children,
                self_closing: open.self_closing,
                span: Span::new((offset + pos) as u32, (offset + end) as u32),
            })),
            end,
        )))
    }

    fn parse_jsx_phrasing(
        &self,
        content: &'a str,
        offset: usize,
    ) -> ParseResult<Vec<'a, Node<'a>>> {
        self.parse_inline(content, offset)
    }

    fn parse_jsx_flow_children(
        &self,
        inner_start: usize,
        inner_end: usize,
    ) -> ParseResult<Vec<'a, Node<'a>>> {
        if inner_start >= inner_end || self.source[inner_start..inner_end].trim().is_empty() {
            return Ok(self.allocator.new_vec());
        }
        let inner = &self.source[inner_start..inner_end];
        let child_source = children::normalize_indentation(self.allocator, inner);
        let mut sub =
            self.sub_parser_with_lazy_lines(child_source.source, rustc_hash::FxHashSet::default());
        if child_source.offsets.is_none() {
            // Children that kept their indentation are a slice of this
            // source, so the walk that found their closing tag already
            // paired the tags inside them. Without this every level of an
            // element nest walked its own children again.
            sub.mdx_scan = self.mdx_scan.share();
        }
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
