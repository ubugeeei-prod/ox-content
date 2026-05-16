use tower_lsp::lsp_types::{Diagnostic, DiagnosticSeverity, Position, Range};

use crate::document::TextDocumentState;
use crate::frontmatter::utils::{strip_line_breaks, yaml_error_range};
use crate::frontmatter::{FrontmatterBlock, FrontmatterDocument, TopLevelKey};

pub fn parse_frontmatter(document: &TextDocumentState) -> FrontmatterDocument {
    if strip_line_breaks(document.line_text(0)) != "---" {
        return FrontmatterDocument { block: None };
    }

    let Some(closing_line) = find_closing_line(document) else {
        return FrontmatterDocument { block: Some(unterminated_block(document)) };
    };

    let content_start_offset = document.line_end_offset(0).min(document.text().len());
    let content_end_offset = document.line_start_offset(closing_line);
    let raw = document.text()[content_start_offset..content_end_offset].to_string();
    let top_level_keys = collect_top_level_keys(document, content_start_offset, &raw);
    let (value, diagnostics) = parse_yaml(document, content_start_offset, &raw);

    FrontmatterDocument {
        block: Some(FrontmatterBlock {
            block_range: document.range_from_offsets(0, document.line_end_offset(closing_line)),
            content_range: document.range_from_offsets(content_start_offset, content_end_offset),
            content_start_offset,
            content_end_offset,
            value,
            diagnostics,
            top_level_keys,
        }),
    }
}

fn find_closing_line(document: &TextDocumentState) -> Option<usize> {
    (1..document.line_count())
        .find(|line| matches!(strip_line_breaks(document.line_text(*line as u32)), "---" | "..."))
}

fn unterminated_block(document: &TextDocumentState) -> FrontmatterBlock {
    let range = Range {
        start: Position { line: 0, character: 0 },
        end: Position { line: 0, character: 3 },
    };

    FrontmatterBlock {
        block_range: Range {
            start: range.start,
            end: document.offset_to_position(document.text().len()),
        },
        content_range: Range {
            start: Position { line: 1, character: 0 },
            end: document.offset_to_position(document.text().len()),
        },
        content_start_offset: document.line_start_offset(1.min(document.line_count())),
        content_end_offset: document.text().len(),
        value: None,
        diagnostics: vec![Diagnostic {
            range,
            severity: Some(DiagnosticSeverity::ERROR),
            source: Some("ox-content".to_string()),
            message: "Unterminated frontmatter block".to_string(),
            ..Default::default()
        }],
        top_level_keys: Vec::new(),
    }
}

fn parse_yaml(
    document: &TextDocumentState,
    content_start_offset: usize,
    raw: &str,
) -> (Option<serde_json::Value>, Vec<Diagnostic>) {
    match serde_yaml::from_str::<serde_yaml::Value>(raw) {
        Ok(parsed) => (serde_json::to_value(parsed).ok(), Vec::new()),
        Err(error) => (
            None,
            vec![Diagnostic {
                range: yaml_error_range(document, content_start_offset, raw, &error),
                severity: Some(DiagnosticSeverity::ERROR),
                source: Some("ox-content".to_string()),
                message: format!("Invalid YAML frontmatter: {error}"),
                ..Default::default()
            }],
        ),
    }
}

fn collect_top_level_keys(
    document: &TextDocumentState,
    content_start_offset: usize,
    raw: &str,
) -> Vec<TopLevelKey> {
    let mut entries = Vec::new();
    let mut line_offset = 0usize;

    for line in raw.lines() {
        let trimmed = strip_line_breaks(line);
        let indent = trimmed
            .chars()
            .take_while(|ch| *ch == ' ' || *ch == '\t')
            .map(char::len_utf8)
            .sum::<usize>();
        let candidate = trimmed.trim();

        if !candidate.is_empty() && !candidate.starts_with('#') && indent == 0 {
            if let Some(colon_index) = trimmed.find(':') {
                let raw_key = trimmed[indent..colon_index].trim();
                if !raw_key.is_empty() {
                    let name = raw_key.trim_matches('"').trim_matches('\'').to_string();
                    let start = content_start_offset + line_offset + indent;
                    let end = content_start_offset + line_offset + colon_index;
                    entries.push(TopLevelKey {
                        name,
                        key_range: document.range_from_offsets(start, end),
                    });
                }
            }
        }

        line_offset += line.len() + 1;
    }

    entries
}
