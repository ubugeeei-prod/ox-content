use ox_content_allocator::Allocator;
use ox_content_parser::{ParseError, Parser, ParserOptions};
use tower_lsp::lsp_types::{Diagnostic, DiagnosticSeverity};

use crate::document::TextDocumentState;
use crate::frontmatter::FrontmatterBlock;

pub(super) fn markdown_parse_diagnostics(
    document: &TextDocumentState,
    block: Option<&FrontmatterBlock>,
) -> Vec<Diagnostic> {
    let (source, offset) = block.map_or_else(
        || (document.text(), 0),
        |block| {
            (
                &document.text()[block.content_start_offset..block.content_end_offset],
                block.content_start_offset,
            )
        },
    );

    let allocator = Allocator::new();
    let parser = Parser::with_options(&allocator, source, ParserOptions::gfm());
    let diagnostics = match parser.parse() {
        Ok(_) => Vec::new(),
        Err(error) => vec![parse_error_to_diagnostic(document, offset, error)],
    };

    diagnostics
}

fn parse_error_to_diagnostic(
    document: &TextDocumentState,
    base_offset: usize,
    error: ParseError,
) -> Diagnostic {
    let span = error.span();
    let start = base_offset + span.start as usize;
    let end = (base_offset + span.end as usize).max(start + 1).min(document.text().len());

    Diagnostic {
        range: document.range_from_offsets(start, end),
        severity: Some(DiagnosticSeverity::ERROR),
        source: Some("ox-content".to_string()),
        message: error.to_string(),
        ..Default::default()
    }
}
