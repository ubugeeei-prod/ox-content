use crate::document::TextDocumentState;
use ox_content_markdown_lint::{MarkdownLintOptions, MarkdownLinter};
use std::sync::Arc;
use tower_lsp::lsp_types::{
    Diagnostic, DiagnosticSeverity, NumberOrString, Position, Range, TextEdit,
};

pub const SOURCE: &str = "ox-content-lint";

#[derive(Clone)]
pub struct Config {
    markdown: Arc<MarkdownLinter>,
    mdx: Arc<MarkdownLinter>,
}

impl std::fmt::Debug for Config {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.debug_struct("MarkdownLintConfig").finish_non_exhaustive()
    }
}

impl Config {
    pub fn new(mut options: MarkdownLintOptions) -> Self {
        options.rules.get_or_insert_default().spellcheck.get_or_insert(false);
        let markdown = Arc::new(MarkdownLinter::new(Some(options.clone())));
        options.mdx = Some(true);
        let mdx = Arc::new(MarkdownLinter::new(Some(options)));
        Self { markdown, mdx }
    }
}

pub fn diagnostics(document: &TextDocumentState, config: &Config, mdx: bool) -> Vec<Diagnostic> {
    let linter = if mdx { &config.mdx } else { &config.markdown };
    linter
        .lint_without_mask(document.text())
        .diagnostics
        .into_iter()
        .map(|value| {
            let range = Range::new(
                Position::new(value.line - 1, value.column - 1),
                Position::new(value.end_line - 1, value.end_column - 1),
            );
            let data = value.fix.and_then(|fix| {
                serde_json::to_value(TextEdit {
                    range: document.range_from_offsets(fix.start as usize, fix.end as usize),
                    new_text: fix.text,
                })
                .ok()
            });
            Diagnostic {
                range,
                severity: Some(match value.severity.as_str() {
                    "error" => DiagnosticSeverity::ERROR,
                    "info" => DiagnosticSeverity::INFORMATION,
                    _ => DiagnosticSeverity::WARNING,
                }),
                code: Some(NumberOrString::String(value.rule_id)),
                source: Some(SOURCE.into()),
                message: value.message,
                data,
                ..Default::default()
            }
        })
        .collect()
}

pub fn fix_edit_from_diagnostic(diagnostic: &Diagnostic) -> Option<TextEdit> {
    if diagnostic.source.as_deref() != Some(SOURCE) {
        return None;
    }
    serde_json::from_value(diagnostic.data.clone()?).ok()
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn unicode_quickfix_uses_utf16_and_the_exact_edit_range() {
        let document = TextDocumentState::new("😀 with with\n".into());
        let values = diagnostics(&document, &Config::new(MarkdownLintOptions::default()), false);
        assert_eq!(values.len(), 1);
        assert_eq!(values[0].range.start.character, 8);
        let edit = fix_edit_from_diagnostic(&values[0]).unwrap();
        assert_eq!(edit.range.start.character, 7);
        assert_eq!(edit.range.end.character, 12);
        assert!(edit.new_text.is_empty());
    }
}
