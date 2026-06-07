//! Smart selection ranges for Markdown / MDC documents.
//!
//! Powers the editor's "expand selection" command: from the word under
//! the cursor outward through the inline node, the block, and finally the
//! whole document. Each level is a strictly nested range, as the LSP
//! `textDocument/selectionRange` response requires.
//!
//! Ranges come straight from the AST (every node whose span covers the
//! cursor), plus a word range for the innermost step and the document
//! range for the outermost. They're rebased through the same
//! frontmatter-offset trick the outline and folding code use.

use ox_content_allocator::Allocator;
use ox_content_ast::{
    walk_list_item, walk_node, walk_table_cell, walk_table_row, ListItem, Node, Span, TableCell,
    TableRow, Visit,
};
use ox_content_parser::{Parser, ParserOptions};
use tower_lsp::lsp_types::{Position, SelectionRange};

use crate::document::TextDocumentState;
use crate::frontmatter::parse_frontmatter;

/// Computes a nested selection range for each requested position.
#[must_use]
pub fn selection_ranges(
    document: &TextDocumentState,
    positions: &[Position],
) -> Vec<SelectionRange> {
    let source = document.text();
    let (content, base_offset) = match parse_frontmatter(document).block {
        Some(block) => (&source[block.block_end_offset..], block.block_end_offset),
        None => (source, 0),
    };

    // Parse once and reuse the AST across every requested position.
    let allocator = Allocator::for_source_len(content.len());
    let parser = Parser::with_options(&allocator, content, ParserOptions::gfm());
    let parsed = parser.parse();
    let children: &[Node] = match &parsed {
        Ok(document) => &document.children,
        Err(_) => &[],
    };

    positions
        .iter()
        .map(|&position| {
            let target = document.position_to_offset(position);
            let mut ranges: Vec<(usize, usize)> = Vec::new();

            // Innermost step: the identifier-like word under the cursor.
            let word = document.word_range_at(position, |ch| ch.is_alphanumeric() || ch == '_');
            let word_start = document.position_to_offset(word.start);
            let word_end = document.position_to_offset(word.end);
            if word_end > word_start {
                ranges.push((word_start, word_end));
            }

            // Every AST node whose span covers the cursor, rebased to
            // document coordinates.
            if target >= base_offset {
                let mut collector = SpanCollector::new((target - base_offset) as u32);
                for child in children {
                    collector.visit_node(child);
                }
                for span in collector.spans {
                    ranges
                        .push((base_offset + span.start as usize, base_offset + span.end as usize));
                }
            }

            // Outermost step: the whole document.
            ranges.push((0, source.len()));

            build_chain(document, target, ranges)
        })
        .collect()
}

/// Collects the span of every node whose range covers `offset`. The
/// `visit_node` override catches block and inline nodes; list items and
/// table rows/cells are dispatched through their own visit methods, so
/// they're recorded explicitly to keep those selection steps available.
struct SpanCollector {
    offset: u32,
    spans: Vec<Span>,
}

impl SpanCollector {
    fn new(offset: u32) -> Self {
        Self { offset, spans: Vec::new() }
    }

    fn record(&mut self, span: Span) {
        if span.start <= self.offset && self.offset < span.end {
            self.spans.push(span);
        }
    }
}

impl<'a> Visit<'a> for SpanCollector {
    fn visit_node(&mut self, node: &Node<'a>) {
        self.record(node.span());
        walk_node(self, node);
    }

    fn visit_list_item(&mut self, item: &ListItem<'a>) {
        self.record(item.span);
        walk_list_item(self, item);
    }

    fn visit_table_row(&mut self, row: &TableRow<'a>) {
        self.record(row.span);
        walk_table_row(self, row);
    }

    fn visit_table_cell(&mut self, cell: &TableCell<'a>) {
        self.record(cell.span);
        walk_table_cell(self, cell);
    }
}

/// Builds the nested `SelectionRange` chain (innermost returned, parents
/// pointing outward) from the collected candidate ranges. Ranges that
/// don't cover the cursor are dropped, and each step must be strictly
/// contained in its parent so the chain is always well-formed.
fn build_chain(
    document: &TextDocumentState,
    target: usize,
    mut ranges: Vec<(usize, usize)>,
) -> SelectionRange {
    ranges.retain(|&(start, end)| start <= target && target <= end);
    ranges.sort_by_key(|&(start, end)| end - start);
    ranges.dedup();

    let mut selection: Option<SelectionRange> = None;
    let mut parent_bounds: Option<(usize, usize)> = None;
    for &(start, end) in ranges.iter().rev() {
        if let Some((parent_start, parent_end)) = parent_bounds {
            let strictly_inside = start >= parent_start
                && end <= parent_end
                && (start > parent_start || end < parent_end);
            if !strictly_inside {
                continue;
            }
        }
        selection = Some(SelectionRange {
            range: document.range_from_offsets(start, end),
            parent: selection.take().map(Box::new),
        });
        parent_bounds = Some((start, end));
    }

    selection.unwrap_or_else(|| SelectionRange {
        range: document.range_from_offsets(target, target),
        parent: None,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fmt::Write as _;

    /// Returns the source substring covered by each step of the chain,
    /// innermost first — a readable proxy for the nested ranges.
    fn chain_texts(source: &str, position: Position) -> Vec<String> {
        let document = TextDocumentState::new(source.to_string());
        let ranges = selection_ranges(&document, &[position]);
        assert_eq!(ranges.len(), 1, "one selection range per position");

        let mut texts = Vec::new();
        let mut current = Some(&ranges[0]);
        while let Some(selection) = current {
            let start = document.position_to_offset(selection.range.start);
            let end = document.position_to_offset(selection.range.end);
            texts.push(source[start..end].to_string());
            current = selection.parent.as_deref();
        }
        texts
    }

    fn position(line: u32, character: u32) -> Position {
        Position { line, character }
    }

    #[test]
    fn expands_word_to_paragraph_to_document() {
        let source = "Hello world\n";
        // Cursor inside "world".
        let texts = chain_texts(source, position(0, 8));
        assert_eq!(texts, vec!["world", "Hello world", "Hello world\n"]);
    }

    #[test]
    fn expands_through_inline_emphasis() {
        let source = "a **bold** word\n";
        // Cursor inside "bold": word -> the strong inline node -> the
        // surrounding block. (Here the paragraph span already covers the
        // whole single-line document, so there's no separate block step.)
        let texts = chain_texts(source, position(0, 5));
        assert_eq!(texts, vec!["bold", "**bold**", "a **bold** word\n"]);
    }

    #[test]
    fn steps_are_strictly_nested() {
        let source = "# Heading\n\nFirst paragraph with words.\n";
        let document = TextDocumentState::new(source.to_string());
        let ranges = selection_ranges(&document, &[position(2, 6)]);
        let selection = &ranges[0];

        // Walk the chain and assert each parent strictly contains its child.
        let mut current = Some(selection);
        let mut child_bounds: Option<(usize, usize)> = None;
        while let Some(node) = current {
            let start = document.position_to_offset(node.range.start);
            let end = document.position_to_offset(node.range.end);
            if let Some((cs, ce)) = child_bounds {
                assert!(start <= cs && ce <= end, "parent must contain child");
                assert!(start < cs || ce < end, "parent must be strictly larger");
            }
            child_bounds = Some((start, end));
            current = node.parent.as_deref();
        }
    }

    #[test]
    fn returns_one_range_per_position() {
        let source = "one two three\n";
        let document = TextDocumentState::new(source.to_string());
        let positions = [position(0, 1), position(0, 5), position(0, 9)];
        let ranges = selection_ranges(&document, &positions);
        assert_eq!(ranges.len(), 3);

        // Sanity: each innermost range is the word at that cursor.
        let mut innermost = String::new();
        for (selection, expected) in ranges.iter().zip(["one", "two", "three"]) {
            let start = document.position_to_offset(selection.range.start);
            let end = document.position_to_offset(selection.range.end);
            let _ = write!(innermost, "{} ", &source[start..end]);
            assert_eq!(&source[start..end], expected);
        }
    }

    #[test]
    fn position_inside_frontmatter_still_yields_a_range() {
        let source = "---\ntitle: Doc\n---\n\n# Body\n";
        let document = TextDocumentState::new(source.to_string());
        // Cursor on the frontmatter `title` key (before the body the
        // parser sees) should not panic and should return the document
        // range at minimum.
        let ranges = selection_ranges(&document, &[position(1, 2)]);
        assert_eq!(ranges.len(), 1);
        // Outermost step covers the whole document.
        let mut outer = &ranges[0];
        while let Some(parent) = outer.parent.as_deref() {
            outer = parent;
        }
        let start = document.position_to_offset(outer.range.start);
        let end = document.position_to_offset(outer.range.end);
        assert_eq!((start, end), (0, source.len()));
    }
}
