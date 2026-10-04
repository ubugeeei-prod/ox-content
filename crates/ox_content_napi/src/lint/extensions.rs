use super::JsMarkdownLintResult;
use napi_derive::napi;
use ox_content_markdown_lint::{
    MarkdownLintFix, MarkdownLintFixResult, MarkdownLintRuleSeverity, MarkdownLintSeverity,
    MarkdownLintTerm, MarkdownLintTextRules,
};

#[napi(object)]
#[derive(Clone)]
pub struct JsMarkdownLintFix {
    /// UTF-8 byte offsets into the original document, end exclusive.
    pub start: u32,
    pub end: u32,
    pub text: String,
}

impl From<MarkdownLintFix> for JsMarkdownLintFix {
    fn from(value: MarkdownLintFix) -> Self {
        Self { start: value.start, end: value.end, text: value.text }
    }
}

#[napi(object)]
#[derive(Clone)]
pub struct JsMarkdownLintTerm {
    pub term: String,
    pub replacement: String,
}

#[napi(object)]
#[derive(Default, Clone)]
pub struct JsMarkdownLintTextRules {
    pub sentence_length: Option<u32>,
    pub max_ten: Option<u32>,
    pub no_exclamation_question_mark: Option<bool>,
    pub no_todo: Option<bool>,
    pub terminology: Option<Vec<JsMarkdownLintTerm>>,
}

impl From<JsMarkdownLintTextRules> for MarkdownLintTextRules {
    fn from(value: JsMarkdownLintTextRules) -> Self {
        Self {
            sentence_length: value.sentence_length,
            max_ten: value.max_ten,
            no_exclamation_question_mark: value.no_exclamation_question_mark,
            no_todo: value.no_todo,
            terminology: value.terminology.map(|terms| {
                terms
                    .into_iter()
                    .map(|v| MarkdownLintTerm { term: v.term, replacement: v.replacement })
                    .collect()
            }),
        }
    }
}

#[napi(object)]
pub struct JsMarkdownLintFixResult {
    pub output: String,
    pub applied_fixes: u32,
    pub result: JsMarkdownLintResult,
}

impl From<MarkdownLintFixResult> for JsMarkdownLintFixResult {
    fn from(value: MarkdownLintFixResult) -> Self {
        Self {
            output: value.output,
            applied_fixes: value.applied_fixes,
            result: value.result.into(),
        }
    }
}

pub(super) fn severities(
    values: std::collections::HashMap<String, String>,
) -> napi::Result<Vec<MarkdownLintRuleSeverity>> {
    values
        .into_iter()
        .map(|(rule_id, value)| {
            let severity = match value.as_str() {
                "off" => MarkdownLintSeverity::Off,
                "info" => MarkdownLintSeverity::Info,
                "warning" => MarkdownLintSeverity::Warning,
                "error" => MarkdownLintSeverity::Error,
                _ => {
                    return Err(napi::Error::from_reason(format!(
                        "Invalid severity for {rule_id}: {value}"
                    )));
                }
            };
            Ok(MarkdownLintRuleSeverity { rule_id, severity })
        })
        .collect()
}
