use super::utils::*;
use super::*;

pub(super) fn collect_repeated_punctuation_diagnostics(
    line_number: usize,
    masked_line: &str,
) -> Vec<MarkdownLintDiagnostic> {
    let chars = masked_line.chars().collect::<Vec<_>>();
    let mut diagnostics = Vec::new();
    let mut index = 0;

    while index + 1 < chars.len() {
        let value = chars[index];
        if !is_repeated_punctuation_char(value) || chars[index + 1] != value {
            index += 1;
            continue;
        }

        let start = index;
        let mut end = index + 1;
        while end < chars.len() && chars[end] == value {
            end += 1;
        }

        diagnostics.push(create_diagnostic(
            "repeated-punctuation",
            format!(
                "Repeated punctuation \"{}\" looks accidental.",
                chars[start..end].iter().collect::<String>()
            ),
            line_number,
            start + 1,
            end + 1,
            None,
            None,
        ));
        index = end;
    }

    diagnostics
}

pub(super) fn should_ignore_repeated_word_token(token: &Token) -> bool {
    if token.language == "ja" || token.language == "zh" {
        return count_code_points(&token.text) <= 1;
    }

    normalize_comparable_word(&token.text).chars().count() <= 1
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
    }
}
