use super::utils::*;
use super::*;

pub(super) fn collect_repeated_punctuation_diagnostics(
    line: &str,
    masked: &str,
    offset: usize,
    source: &super::source::Source<'_>,
    output: &mut Vec<MarkdownLintDiagnostic>,
) {
    let mut chars = line.char_indices().zip(masked.chars()).peekable();
    while let Some(((start, _), c)) = chars.next() {
        if !is_repeated_punctuation_char(c) || chars.peek().is_none_or(|(_, next)| *next != c) {
            continue;
        }
        let mut end = start;
        while let Some(((byte, original), next)) = chars.peek() {
            if *next != c {
                break;
            }
            end = *byte + original.len_utf8();
            chars.next();
        }
        output.push(source.diagnostic(
            "repeated-punctuation",
            format!("Repeated punctuation \"{}\" looks accidental.", &line[start..end]),
            offset + start,
            offset + end,
        ));
    }
}

pub(super) fn should_ignore_repeated_word_token(token: &Token) -> bool {
    if token.language == "ja" || token.language == "zh" {
        return count_code_points(&token.text) <= 1;
    }

    if token.text.is_ascii() {
        token.text.len() <= 1
    } else {
        normalize_comparable_word(&token.text).chars().count() <= 1
    }
}

pub(super) fn summarize_diagnostics(
    diagnostics: Vec<MarkdownLintDiagnostic>,
    masked_document: String,
) -> MarkdownLintResult {
    let error_count =
        diagnostics.iter().filter(|diagnostic| diagnostic.severity == "error").count();
    let warning_count =
        diagnostics.iter().filter(|diagnostic| diagnostic.severity == "warning").count();
    let info_count = diagnostics.iter().filter(|diagnostic| diagnostic.severity == "info").count();

    MarkdownLintResult {
        diagnostics,
        error_count: error_count as u32,
        warning_count: warning_count as u32,
        info_count: info_count as u32,
        masked_document,
    }
}

pub(super) fn sort_diagnostics(
    mut diagnostics: Vec<MarkdownLintDiagnostic>,
) -> Vec<MarkdownLintDiagnostic> {
    diagnostics.sort_by(|left, right| {
        left.line
            .cmp(&right.line)
            .then_with(|| left.column.cmp(&right.column))
            .then_with(|| left.rule_id.cmp(&right.rule_id))
    });
    diagnostics
}

pub(super) fn create_diagnostic(
    rule_id: &str,
    message: String,
    line: usize,
    column: usize,
    end_column: usize,
    language: Option<String>,
    suggestions: Option<Vec<String>>,
) -> MarkdownLintDiagnostic {
    MarkdownLintDiagnostic {
        rule_id: rule_id.to_string(),
        severity: "warning".to_string(),
        message,
        line: line as u32,
        column: column as u32,
        end_line: line as u32,
        end_column: end_column as u32,
        language,
        suggestions,
        fix: None,
    }
}
