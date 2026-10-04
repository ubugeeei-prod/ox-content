use crate::Result;
use ox_content_markdown_lint::{
    MarkdownLintDictionaryOptions, MarkdownLintOptions, MarkdownLintRuleOptions,
    MarkdownLintRuleSeverity, MarkdownLintSeverity, MarkdownLintTextRules,
};
use serde::Deserialize;
use std::collections::BTreeMap;

#[derive(Default, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Config {
    #[serde(default)]
    pub include: Vec<String>,
    #[serde(default)]
    pub ignore: Vec<String>,
    languages: Option<Vec<String>>,
    rules: Option<MarkdownLintRuleOptions>,
    dictionary: Option<MarkdownLintDictionaryOptions>,
    text_rules: Option<MarkdownLintTextRules>,
    severities: Option<BTreeMap<String, MarkdownLintSeverity>>,
}

impl Config {
    pub fn read(path: &str) -> Result<Self> {
        let source = std::fs::read_to_string(path)?;
        serde_json::from_str(&source)
            .map_err(|error| format!("Invalid lint rule or configuration: {error}").into())
    }

    pub fn native(&self, spellcheck: bool, mdx: bool, strict: bool) -> MarkdownLintOptions {
        let mut rules = self.rules.clone().unwrap_or_default();
        rules.spellcheck = Some(spellcheck || rules.spellcheck.unwrap_or(false));
        if strict {
            rules.empty_headings.get_or_insert(true);
            rules.first_heading_h1.get_or_insert(true);
            rules.single_h1.get_or_insert(true);
            rules.code_fence_language.get_or_insert(true);
            rules.code_fence_closed.get_or_insert(true);
            rules.empty_links.get_or_insert(true);
            rules.image_alt.get_or_insert(true);
            rules.final_newline.get_or_insert(true);
        }
        MarkdownLintOptions {
            languages: self.languages.clone(),
            mdx: Some(mdx),
            rules: Some(rules),
            dictionary: self.dictionary.clone(),
            text_rules: self.text_rules.clone(),
            severities: self.severities.as_ref().map(|entries| {
                entries
                    .iter()
                    .map(|(rule, severity)| MarkdownLintRuleSeverity {
                        rule_id: rule.clone(),
                        severity: *severity,
                    })
                    .collect()
            }),
        }
    }
}
