use serde::{Deserialize, Serialize};

/// UTF-8 byte offsets into the original source. End is exclusive.
#[derive(Clone, Debug, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct MarkdownLintFix {
    pub start: u32,
    pub end: u32,
    pub text: String,
}

/// Native prose rules inspired by textlint. All are off by default.
#[derive(Default, Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct MarkdownLintTextRules {
    /// Maximum Unicode scalar values per sentence, excluding Markdown syntax.
    pub sentence_length: Option<u32>,
    /// Maximum Japanese commas (、) per sentence.
    pub max_ten: Option<u32>,
    pub no_exclamation_question_mark: Option<bool>,
    pub no_todo: Option<bool>,
    /// Exact, case-sensitive replacements, never regular expressions.
    pub terminology: Option<Vec<MarkdownLintTerm>>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct MarkdownLintTerm {
    pub term: String,
    pub replacement: String,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct MarkdownLintRuleSeverity {
    pub rule_id: String,
    pub severity: MarkdownLintSeverity,
}

#[derive(Clone, Copy, Debug, Deserialize, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum MarkdownLintSeverity {
    Off,
    Info,
    Warning,
    Error,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MarkdownLintFixResult {
    pub output: String,
    pub applied_fixes: u32,
    pub result: super::MarkdownLintResult,
}

impl super::MarkdownLintRuleOptions {
    /// Enables document style rules without enabling optional prose rules.
    pub fn strict() -> Self {
        Self {
            empty_headings: Some(true),
            first_heading_h1: Some(true),
            single_h1: Some(true),
            code_fence_language: Some(true),
            code_fence_closed: Some(true),
            empty_links: Some(true),
            image_alt: Some(true),
            final_newline: Some(true),
            ..Self::default()
        }
    }
}

pub(super) fn deserialize_severities<'de, D: serde::Deserializer<'de>>(
    deserializer: D,
) -> Result<Option<Vec<MarkdownLintRuleSeverity>>, D::Error> {
    #[derive(Deserialize)]
    #[serde(untagged)]
    enum Config {
        Map(std::collections::BTreeMap<String, MarkdownLintSeverity>),
        Entries(Vec<MarkdownLintRuleSeverity>),
    }
    Ok(Option::<Config>::deserialize(deserializer)?.map(|value| match value {
        Config::Map(values) => values
            .into_iter()
            .map(|(rule_id, severity)| MarkdownLintRuleSeverity { rule_id, severity })
            .collect(),
        Config::Entries(values) => values,
    }))
}
