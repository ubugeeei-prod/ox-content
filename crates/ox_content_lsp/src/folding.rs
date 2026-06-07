//! Folding range computation for Markdown / MDC documents.
//!
//! Surfaces the regions an author expects a Markdown editor to fold:
//!   * the YAML frontmatter block at the top of the file,
//!   * each fenced / indented code block,
//!   * every heading section — a heading folds down to the line before
//!     the next heading of equal-or-shallower depth (or the end of the
//!     document), with trailing blank lines trimmed.
//!
//! Ranges are line-based: `start_character` / `end_character` are left
//! unset so editors fold whole lines, matching the LSP default.

use ox_content_allocator::Allocator;
use ox_content_ast::Node;
use ox_content_parser::{Parser, ParserOptions};
use tower_lsp::lsp_types::{FoldingRange, FoldingRangeKind};

use crate::document::TextDocumentState;
use crate::frontmatter::parse_frontmatter;

/// Computes every foldable region in `document`.
#[must_use]
pub fn folding_ranges(document: &TextDocumentState) -> Vec<FoldingRange> {
    let source = document.text();
    let frontmatter = parse_frontmatter(document);

    let mut ranges = Vec::new();

    // Frontmatter folds as a single region spanning the opening `---`
    // through the closing `---`.
    if let Some(block) = &frontmatter.block {
        push_span_fold(&mut ranges, document, 0, block.block_end_offset);
    }

    // Heading sections and code blocks come from the parsed body. The
    // parser only sees the markdown after the frontmatter, so every
    // span is shifted back into document coordinates via `base_offset`
    // — the same trick the outline (`preview::document_symbols`) uses.
    let (content, base_offset) = match &frontmatter.block {
        Some(block) => (&source[block.block_end_offset..], block.block_end_offset),
        None => (source, 0),
    };

    let allocator = Allocator::for_source_len(content.len());
    let parser = Parser::with_options(&allocator, content, ParserOptions::gfm());
    let Ok(ast) = parser.parse() else {
        return ranges;
    };

    // One pass collects code-block folds (independent of each other)
    // and the heading skeleton; sections are resolved afterwards once
    // every heading's successor is known.
    let mut headings: Vec<(u8, u32)> = Vec::new();
    for node in &ast.children {
        match node {
            Node::CodeBlock(code) => push_span_fold(
                &mut ranges,
                document,
                base_offset + code.span.start as usize,
                base_offset + code.span.end as usize,
            ),
            Node::Heading(heading) => {
                let start = base_offset + heading.span.start as usize;
                headings.push((heading.depth, document.offset_to_position(start).line));
            }
            _ => {}
        }
    }

    append_heading_sections(&mut ranges, document, &headings);
    ranges
}

/// Resolves the flat heading list into section folds. Each heading owns
/// the lines down to (but not including) the next heading whose depth is
/// equal or shallower; trailing blank lines are trimmed so the fold hugs
/// the section's real content.
fn append_heading_sections(
    ranges: &mut Vec<FoldingRange>,
    document: &TextDocumentState,
    headings: &[(u8, u32)],
) {
    let last_line = (document.line_count().saturating_sub(1)) as u32;
    for (index, &(depth, start_line)) in headings.iter().enumerate() {
        let mut end_line = headings[index + 1..]
            .iter()
            .find(|&&(next_depth, _)| next_depth <= depth)
            .map_or(last_line, |&(_, next_line)| next_line.saturating_sub(1));

        while end_line > start_line && document.line_text(end_line).trim().is_empty() {
            end_line -= 1;
        }

        if end_line > start_line {
            ranges.push(region(start_line, end_line));
        }
    }
}

/// Pushes a fold covering the lines touched by `[start_offset,
/// end_offset)`, skipping single-line spans the editor cannot collapse.
fn push_span_fold(
    ranges: &mut Vec<FoldingRange>,
    document: &TextDocumentState,
    start_offset: usize,
    end_offset: usize,
) {
    let start_line = document.offset_to_position(start_offset).line;
    // `end_offset` is exclusive and often lands at the start of the
    // following line; step back one byte so the fold ends on the
    // block's last line rather than the line after it.
    let end_line = document.offset_to_position(end_offset.saturating_sub(1)).line;
    if end_line > start_line {
        ranges.push(region(start_line, end_line));
    }
}

fn region(start_line: u32, end_line: u32) -> FoldingRange {
    FoldingRange {
        start_line,
        end_line,
        kind: Some(FoldingRangeKind::Region),
        ..Default::default()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fmt::Write as _;

    fn fold(source: &str) -> String {
        let document = TextDocumentState::new(source.to_string());
        let mut ranges = folding_ranges(&document);
        ranges.sort_by_key(|range| (range.start_line, range.end_line));
        let mut out = String::new();
        for range in ranges {
            let _ = writeln!(&mut out, "{}..{}", range.start_line, range.end_line);
        }
        out
    }

    #[test]
    fn folds_heading_section_to_next_sibling() {
        // The h2 section ends on the line before the next h2; the file's
        // final h2 runs to the last content line.
        let source = "# Title\n\n## First\n\nbody\n\n## Second\n\nmore\n";
        assert_eq!(fold(source), "0..8\n2..4\n6..8\n");
    }

    #[test]
    fn deeper_heading_nests_inside_its_parent() {
        // The h3 is part of the h2 section, so the h2 still extends past
        // it; the h3 owns only its own lines.
        let source = "## Parent\n\ntext\n\n### Child\n\nleaf\n";
        assert_eq!(fold(source), "0..6\n4..6\n");
    }

    #[test]
    fn single_line_heading_is_not_foldable() {
        // A heading with no body underneath produces no range — there's
        // nothing to collapse.
        let source = "# Lonely\n";
        assert_eq!(fold(source), "");
    }

    #[test]
    fn folds_fenced_code_block() {
        let source = "para\n\n```rust\nfn main() {}\nlet x = 1;\n```\n";
        assert_eq!(fold(source), "2..5\n");
    }

    #[test]
    fn folds_frontmatter_block() {
        let source = "---\ntitle: Doc\ntags: [a, b]\n---\n\n# Heading\n\nbody\n";
        // Frontmatter (lines 0..3) plus the heading section (5..7).
        assert_eq!(fold(source), "0..3\n5..7\n");
    }

    #[test]
    fn trims_trailing_blank_lines_from_section() {
        // Three trailing blank lines should not be swallowed into the
        // fold; it stops at the last line with content.
        let source = "## Section\n\nbody\n\n\n\n";
        assert_eq!(fold(source), "0..2\n");
    }

    #[test]
    fn empty_document_has_no_folds() {
        assert_eq!(fold(""), "");
    }
}
