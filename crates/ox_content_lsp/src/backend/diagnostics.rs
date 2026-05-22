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

pub(super) fn mdc_diagnostics(
    document: &TextDocumentState,
    block: Option<&FrontmatterBlock>,
) -> Vec<Diagnostic> {
    let (source, line_offset) = block.map_or_else(
        || (document.text(), 0),
        |block| {
            let before = &document.text()[..block.content_start_offset];
            (
                &document.text()[block.content_start_offset..block.content_end_offset],
                before.chars().filter(|ch| *ch == '\n').count() as u32,
            )
        },
    );

    ox_content_mdc_checker::check(source)
        .into_iter()
        .map(|diagnostic| {
            let start_line = diagnostic.line.saturating_sub(1) + line_offset;
            let end_line = diagnostic.end_line.saturating_sub(1) + line_offset;
            Diagnostic {
                range: tower_lsp::lsp_types::Range {
                    start: tower_lsp::lsp_types::Position {
                        line: start_line,
                        character: diagnostic.column.saturating_sub(1),
                    },
                    end: tower_lsp::lsp_types::Position {
                        line: end_line,
                        character: diagnostic.end_column.saturating_sub(1),
                    },
                },
                severity: Some(DiagnosticSeverity::ERROR),
                source: Some("ox-content-mdc".to_string()),
                message: diagnostic.message,
                ..Default::default()
            }
        })
        .collect()
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
