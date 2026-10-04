#[derive(Clone, Debug, serde::Deserialize, serde::Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct MarkdownLintLanguageWords {
    pub language: String,
    pub words: Vec<String>,
}

#[derive(Default, Clone, Debug, serde::Deserialize, serde::Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct MarkdownLintDictionaryOptions {
    pub words: Option<Vec<String>>,
    pub by_language: Option<Vec<MarkdownLintLanguageWords>>,
    pub ignored_words: Option<Vec<String>>,
}

#[derive(Default, Clone, Debug, serde::Deserialize, serde::Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct MarkdownLintRuleOptions {
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

#[derive(Default, Clone, Debug, serde::Deserialize, serde::Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct MarkdownLintOptions {
    pub languages: Option<Vec<String>>,
    pub rules: Option<MarkdownLintRuleOptions>,
    pub dictionary: Option<MarkdownLintDictionaryOptions>,
    pub mdx: Option<bool>,
    pub text_rules: Option<super::MarkdownLintTextRules>,
    #[serde(default, deserialize_with = "super::extensions::deserialize_severities")]
    pub severities: Option<Vec<super::MarkdownLintRuleSeverity>>,
}

#[derive(Clone, Debug, serde::Deserialize, serde::Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct MarkdownLintDiagnostic {
    pub rule_id: String,
    pub severity: String,
    pub message: String,
    pub line: u32,
    pub column: u32,
    pub end_line: u32,
    pub end_column: u32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub language: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub suggestions: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub fix: Option<super::MarkdownLintFix>,
}

#[derive(Debug, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MarkdownLintResult {
    pub diagnostics: Vec<MarkdownLintDiagnostic>,
    pub error_count: u32,
    pub warning_count: u32,
    pub info_count: u32,
    pub masked_document: String,
}
