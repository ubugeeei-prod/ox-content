use napi_derive::napi;
mod extensions;
pub use extensions::*;
use ox_content_markdown_lint::{
    MarkdownLintDiagnostic, MarkdownLintDictionaryOptions, MarkdownLintLanguageWords,
    MarkdownLintOptions, MarkdownLintResult, MarkdownLintRuleOptions,
    lint_markdown as lint_markdown_core, lint_markdown_documents as lint_markdown_documents_core,
};
use std::collections::HashMap;

#[napi(object)]
#[derive(Clone)]
pub struct JsMarkdownLintLanguageWords {
    pub language: String,
    pub words: Vec<String>,
}

impl From<JsMarkdownLintLanguageWords> for MarkdownLintLanguageWords {
    fn from(value: JsMarkdownLintLanguageWords) -> Self {
        Self { language: value.language, words: value.words }
    }
}

#[napi(object)]
#[derive(Default, Clone)]
pub struct JsMarkdownLintDictionaryOptions {
    pub words: Option<Vec<String>>,
    pub by_language: Option<Vec<JsMarkdownLintLanguageWords>>,
    pub ignored_words: Option<Vec<String>>,
}

impl From<JsMarkdownLintDictionaryOptions> for MarkdownLintDictionaryOptions {
    fn from(value: JsMarkdownLintDictionaryOptions) -> Self {
        Self {
            words: value.words,
            by_language: value
                .by_language
                .map(|entries| entries.into_iter().map(Into::into).collect()),
            ignored_words: value.ignored_words,
        }
    }
}

#[napi(object)]
#[derive(Default, Clone)]
pub struct JsMarkdownLintRuleOptions {
    pub duplicate_headings: Option<bool>,
    pub heading_increment: Option<bool>,
    pub max_consecutive_blank_lines: Option<u32>,
    pub repeated_punctuation: Option<bool>,
    pub repeated_words: Option<bool>,
    pub spellcheck: Option<bool>,
    pub trailing_spaces: Option<bool>,
    pub empty_headings: Option<bool>,
    pub first_heading_h1: Option<bool>,
    pub single_h1: Option<bool>,
    pub code_fence_language: Option<bool>,
    pub code_fence_closed: Option<bool>,
    pub empty_links: Option<bool>,
    pub image_alt: Option<bool>,
    pub final_newline: Option<bool>,
}

impl From<JsMarkdownLintRuleOptions> for MarkdownLintRuleOptions {
    fn from(value: JsMarkdownLintRuleOptions) -> Self {
        Self {
            duplicate_headings: value.duplicate_headings,
            heading_increment: value.heading_increment,
            max_consecutive_blank_lines: value.max_consecutive_blank_lines,
            repeated_punctuation: value.repeated_punctuation,
            repeated_words: value.repeated_words,
            spellcheck: value.spellcheck,
            trailing_spaces: value.trailing_spaces,
            empty_headings: value.empty_headings,
            first_heading_h1: value.first_heading_h1,
            single_h1: value.single_h1,
            code_fence_language: value.code_fence_language,
            code_fence_closed: value.code_fence_closed,
            empty_links: value.empty_links,
            image_alt: value.image_alt,
            final_newline: value.final_newline,
        }
    }
}

#[napi(object)]
#[derive(Default, Clone)]
pub struct JsMarkdownLintOptions {
    #[napi(ts_type = "boolean | Record<string, unknown>")]
    pub markdownlint: Option<serde_json::Value>,
    pub no_inline_config: Option<bool>,
    pub languages: Option<Vec<String>>,
    pub rules: Option<JsMarkdownLintRuleOptions>,
    pub dictionary: Option<JsMarkdownLintDictionaryOptions>,
    /// Enable MDX-aware syntax masking while linting visible prose.
    pub mdx: Option<bool>,
    pub text_rules: Option<JsMarkdownLintTextRules>,
    pub severities: Option<HashMap<String, String>>,
}

impl TryFrom<JsMarkdownLintOptions> for MarkdownLintOptions {
    type Error = napi::Error;
    fn try_from(value: JsMarkdownLintOptions) -> napi::Result<Self> {
        Ok(Self {
            no_inline_config: value.no_inline_config,
            markdownlint: value.markdownlint.map(serde_json::from_value).transpose().map_err(
                |error| {
                    napi::Error::from_reason(format!("Invalid markdownlint configuration: {error}"))
                },
            )?,
            languages: value.languages,
            rules: value.rules.map(Into::into),
            dictionary: value.dictionary.map(Into::into),
            mdx: value.mdx,
            text_rules: value.text_rules.map(Into::into),
            severities: value.severities.map(extensions::severities).transpose()?,
        })
    }
}

#[napi(object)]
#[derive(Clone)]
pub struct JsMarkdownLintDiagnostic {
    pub rule_id: String,
    pub severity: String,
    pub message: String,
    pub line: u32,
    pub column: u32,
    pub end_line: u32,
    pub end_column: u32,
    pub language: Option<String>,
    pub suggestions: Option<Vec<String>>,
    pub fix: Option<JsMarkdownLintFix>,
}

impl From<MarkdownLintDiagnostic> for JsMarkdownLintDiagnostic {
    fn from(value: MarkdownLintDiagnostic) -> Self {
        Self {
            rule_id: value.rule_id,
            severity: value.severity,
            message: value.message,
            line: value.line,
            column: value.column,
            end_line: value.end_line,
            end_column: value.end_column,
            language: value.language,
            suggestions: value.suggestions,
            fix: value.fix.map(Into::into),
        }
    }
}

#[napi(object)]
pub struct JsMarkdownLintResult {
    pub diagnostics: Vec<JsMarkdownLintDiagnostic>,
    pub error_count: u32,
    pub warning_count: u32,
    pub info_count: u32,
    pub masked_document: String,
}

impl From<MarkdownLintResult> for JsMarkdownLintResult {
    fn from(value: MarkdownLintResult) -> Self {
        Self {
            diagnostics: value.diagnostics.into_iter().map(Into::into).collect(),
            error_count: value.error_count,
            warning_count: value.warning_count,
            info_count: value.info_count,
            masked_document: value.masked_document,
        }
    }
}

#[napi(js_name = "lintMarkdown")]
pub fn lint_markdown(
    source: String,
    options: Option<JsMarkdownLintOptions>,
) -> napi::Result<JsMarkdownLintResult> {
    Ok(lint_markdown_core(&source, options.map(TryInto::try_into).transpose()?).into())
}

#[napi(js_name = "lintMarkdownDocuments")]
pub fn lint_markdown_documents(
    sources: Vec<String>,
    options: Option<JsMarkdownLintOptions>,
) -> napi::Result<Vec<JsMarkdownLintResult>> {
    Ok(lint_markdown_documents_core(&sources, options.map(TryInto::try_into).transpose()?)
        .into_iter()
        .map(Into::into)
        .collect())
}

#[napi(js_name = "fixMarkdown")]
pub fn fix_markdown(
    source: String,
    options: Option<JsMarkdownLintOptions>,
) -> napi::Result<JsMarkdownLintFixResult> {
    Ok(ox_content_markdown_lint::fix_markdown(&source, options.map(TryInto::try_into).transpose()?)
        .into())
}
