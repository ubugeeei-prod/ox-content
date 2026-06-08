//! Document highlights for Markdown / MDC documents.
//!
//! When the cursor sits inside a Markdown link or image, every other link
//! or image in the document that points at the *same* target is
//! highlighted — a quick way to spot every place a URL or file is
//! referenced. The highlight hugs the URL, matching the document-link
//! ranges.
//!
//! The feature deliberately stays quiet everywhere else: a cursor on
//! plain prose returns nothing, so there's none of the noise a
//! highlight-every-word implementation would produce.

use ox_content_allocator::Allocator;
use ox_content_ast::{walk_link, Image, Link, Span, Visit};
use ox_content_parser::{Parser, ParserOptions};
use tower_lsp::lsp_types::{DocumentHighlight, DocumentHighlightKind, Position};

use crate::document::TextDocumentState;
use crate::frontmatter::parse_frontmatter;

/// Computes the highlights for the link/image target under `position`, or
/// `None` when the cursor isn't on a link or image.
#[must_use]
pub fn document_highlights(
    document: &TextDocumentState,
    position: Position,
) -> Option<Vec<DocumentHighlight>> {
    let source = document.text();
    let (content, base_offset) = match parse_frontmatter(document).block {
        Some(block) => (&source[block.block_end_offset..], block.block_end_offset),
        None => (source, 0),
    };

    let target = document.position_to_offset(position);
    if target < base_offset {
        return None; // inside the frontmatter, where there are no links
    }
    let content_offset = (target - base_offset) as u32;

    let allocator = Allocator::for_source_len(content.len());
    let parser = Parser::with_options(&allocator, content, ParserOptions::gfm());
    let ast = parser.parse().ok()?;

    let mut collector = LinkCollector { found: Vec::new() };
    collector.visit_document(&ast);

    // The link under the cursor decides which target we highlight.
    let url = collector
        .found
        .iter()
        .find(|(span, _)| span.start <= content_offset && content_offset < span.end)
        .map(|(_, url)| *url)?;
    if url.is_empty() {
        return None;
    }

    let highlights: Vec<DocumentHighlight> = collector
        .found
        .iter()
        .filter(|(_, candidate)| *candidate == url)
        .map(|(span, candidate)| {
            let (start, len) = url_subspan(content, *span, candidate);
            DocumentHighlight {
                range: document.range_from_offsets(base_offset + start, base_offset + start + len),
                kind: Some(DocumentHighlightKind::TEXT),
            }
        })
        .collect();

    (!highlights.is_empty()).then_some(highlights)
}

/// Collects `(span, url)` for every link and image, reaching them
/// wherever they nest via the AST `Visit` walker.
struct LinkCollector<'a> {
    found: Vec<(Span, &'a str)>,
}

impl<'a> Visit<'a> for LinkCollector<'a> {
    fn visit_link(&mut self, link: &Link<'a>) {
        self.found.push((link.span, link.url));
        walk_link(self, link);
    }

    fn visit_image(&mut self, image: &Image<'a>) {
        self.found.push((image.span, image.url));
    }
}

/// Locates the URL inside a link's source text so the highlight hugs the
/// target rather than the whole `[label](url)`. Falls back to the entire
/// span for reference-style links whose URL lives in a definition.
fn url_subspan(content: &str, span: Span, url: &str) -> (usize, usize) {
    let span_start = span.start as usize;
    let text = &content[span_start..span.end as usize];
    if let Some(rel) = text.rfind(url) {
        return (span_start + rel, url.len());
    }
    (span_start, text.len())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn highlights(source: &str, position: Position) -> Vec<(u32, u32, u32, u32)> {
        let document = TextDocumentState::new(source.to_string());
        document_highlights(&document, position)
            .unwrap_or_default()
            .into_iter()
            .map(|highlight| {
                (
                    highlight.range.start.line,
                    highlight.range.start.character,
                    highlight.range.end.line,
                    highlight.range.end.character,
                )
            })
            .collect()
    }

    fn position(line: u32, character: u32) -> Position {
        Position { line, character }
    }

    #[test]
    fn highlights_every_link_with_the_same_target() {
        // Two links to `./a.md` and one to `./b.md`; the cursor on the
        // first should highlight both `./a.md` links and not the `./b.md`.
        let source = "[one](./a.md) [two](./b.md)\n\n[three](./a.md)\n";
        let result = highlights(source, position(0, 2));
        assert_eq!(result.len(), 2, "expected both ./a.md links, got {result:?}");
        // Ranges hug the `./a.md` URL: `[one](` is 6 chars, URL is 6 long.
        assert!(result.contains(&(0, 6, 0, 12)), "first link URL range missing: {result:?}");
        assert!(result.contains(&(2, 8, 2, 14)), "third link URL range missing: {result:?}");
    }

    #[test]
    fn matches_images_and_links_sharing_a_target() {
        let source = "![pic](./shared.png) and [link](./shared.png)\n";
        let result = highlights(source, position(0, 3));
        assert_eq!(result.len(), 2, "image and link to the same target, got {result:?}");
    }

    #[test]
    fn returns_none_off_a_link() {
        let document = TextDocumentState::new("just [a](./x.md) here\n".to_string());
        // Cursor on the plain word "just".
        assert!(document_highlights(&document, position(0, 1)).is_none());
    }

    #[test]
    fn single_link_highlights_only_itself() {
        let source = "[solo](./only.md)\n";
        let result = highlights(source, position(0, 3));
        assert_eq!(result.len(), 1);
        assert!(result.contains(&(0, 7, 0, 16)), "URL range missing: {result:?}");
    }

    #[test]
    fn accounts_for_frontmatter_offset() {
        let source = "---\ntitle: Doc\n---\n\n[x](./p.md) [y](./p.md)\n";
        // Both links sit on line 4; cursor on the first.
        let result = highlights(source, position(4, 1));
        assert_eq!(result.len(), 2, "got {result:?}");
        assert!(result.iter().all(|(start_line, _, _, _)| *start_line == 4));
    }
}
