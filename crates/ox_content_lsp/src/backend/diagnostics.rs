use ox_content_allocator::Allocator;
use ox_content_parser::{ParseError, Parser, ParserOptions};
use tower_lsp::lsp_types::{Diagnostic, DiagnosticSeverity};

use crate::document::TextDocumentState;
use crate::frontmatter::FrontmatterBlock;

pub(super) fn markdown_parse_diagnostics(
    document: &TextDocumentState,
    block: Option<&FrontmatterBlock>,
) -> Vec<Diagnostic> {
    // Diagnostics apply to the markdown body, not the YAML frontmatter.
    // `block_end_offset` is the byte just past the closing `---` line.
    let (source, offset) = block.map_or_else(
        || (document.text(), 0),
        |block| (&document.text()[block.block_end_offset..], block.block_end_offset),
    );

    let allocator = Allocator::for_source_len(source.len());
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
    // MDC diagnostics apply to the markdown body. Use the post-frontmatter
    // slice so column/line numbers from the checker line up with the source
    // the editor opened.
    let (source, line_offset) = block.map_or_else(
        || (document.text(), 0),
        |block| {
            let before = &document.text()[..block.block_end_offset];
            (
                &document.text()[block.block_end_offset..],
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::frontmatter::parse_frontmatter;

    #[test]
    fn markdown_parse_diagnostics_skip_valid_body_after_frontmatter() {
        let source = "---\ntitle: Doc\n---\n\n# Valid heading\n\nA paragraph.\n";
        let document = TextDocumentState::new(source.to_string());
        let frontmatter = parse_frontmatter(&document);
        let diagnostics = markdown_parse_diagnostics(&document, frontmatter.block.as_ref());
        assert!(diagnostics.is_empty(), "expected clean parse for valid body, got {diagnostics:?}");
    }

    #[test]
    fn markdown_parse_diagnostics_skip_yaml_inside_frontmatter() {
        // The YAML body would look like a paragraph to the markdown parser
        // and could produce spurious diagnostics. This regression test pins
        // the contract that the YAML is fed to the YAML parser only.
        let source = "---\ntitle: A title with : colon\n---\n\nbody\n";
        let document = TextDocumentState::new(source.to_string());
        let frontmatter = parse_frontmatter(&document);
        let diagnostics = markdown_parse_diagnostics(&document, frontmatter.block.as_ref());
        assert!(diagnostics.is_empty(), "yaml leaked into markdown diagnostics: {diagnostics:?}");
    }
}
