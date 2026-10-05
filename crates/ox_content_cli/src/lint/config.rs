use crate::Result;
use ox_content_markdown_lint::{
    MarkdownLintDictionaryOptions, MarkdownLintOptions, MarkdownLintRuleOptions,
    MarkdownLintRuleSeverity, MarkdownLintSeverity, MarkdownLintTextRules, MarkdownlintConfig,
};
use serde::Deserialize;
use serde_json::Value;
use std::{
    collections::BTreeMap,
    path::{Path, PathBuf},
};

#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Config {
    #[serde(default)]
    pub include: Vec<String>,
    #[serde(default)]
    pub ignore: Vec<String>,
    pub markdownlint: Option<MarkdownlintConfig>,
    #[serde(default)]
    #[serde(rename = "noInlineConfig")]
    pub inline_disabled: bool,
    languages: Option<Vec<String>>,
    rules: Option<MarkdownLintRuleOptions>,
    dictionary: Option<MarkdownLintDictionaryOptions>,
    text_rules: Option<MarkdownLintTextRules>,
    severities: Option<BTreeMap<String, MarkdownLintSeverity>>,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            markdownlint: Some(MarkdownlintConfig::default()),
            inline_disabled: false,
            include: Vec::new(),
            ignore: Vec::new(),
            languages: None,
            rules: None,
            dictionary: None,
            text_rules: None,
            severities: None,
        }
    }
}

impl Config {
    pub fn read(path: &str) -> Result<Self> {
        let value = read_value(Path::new(path), &mut Vec::new())?;
        let ox_config = value.as_object().is_some_and(|map| {
            map.keys().any(|key| {
                matches!(
                    key.as_str(),
                    "noInlineConfig"
                        | "include"
                        | "ignore"
                        | "rules"
                        | "textRules"
                        | "dictionary"
                        | "languages"
                        | "severities"
                        | "markdownlint"
                )
            })
        });
        if ox_config
            || Path::new(path)
                .file_name()
                .is_some_and(|name| name == ".oxlint.json" || name == "oxlint.json")
        {
            serde_json::from_value(value)
                .map_err(|error| format!("Invalid lint rule or configuration: {error}").into())
        } else {
            Ok(Self {
                markdownlint: Some(
                    serde_json::from_value(value)
                        .map_err(|error| format!("Invalid markdownlint configuration: {error}"))?,
                ),
                ..Self::default()
            })
        }
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
            markdownlint: self.markdownlint.clone(),
            no_inline_config: Some(self.inline_disabled),
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

fn read_value(path: &Path, ancestors: &mut Vec<PathBuf>) -> Result<Value> {
    let path = path.canonicalize()?;
    if ancestors.contains(&path) {
        return Err(format!("Circular markdownlint extends: {}", path.display()).into());
    }
    if ancestors.len() >= 32 {
        return Err("Markdownlint extends exceeds 32 configuration files".into());
    }
    ancestors.push(path.clone());
    let source = std::fs::read_to_string(&path)?;
    let mut value: Value = match path.extension().and_then(|extension| extension.to_str()) {
        Some("yaml" | "yml") => serde_yaml::from_str(&source)?,
        _ => jsonc_parser::parse_to_serde_value::<Value>(
            &source,
            &jsonc_parser::ParseOptions::default(),
        )?,
    };
    if let Some(extends) = value.as_object_mut().and_then(|map| map.remove("extends")) {
        let bases = match extends {
            Value::String(path) => vec![path],
            Value::Array(paths) => paths
                .into_iter()
                .map(|value| {
                    value
                        .as_str()
                        .map(str::to_string)
                        .ok_or("markdownlint extends must contain paths")
                })
                .collect::<std::result::Result<Vec<_>, _>>()?,
            _ => return Err("markdownlint extends must be a path or path array".into()),
        };
        let mut inherited = serde_json::Map::new();
        for base in bases {
            let base = path.parent().unwrap_or_else(|| Path::new(".")).join(base);
            let base = read_value(&base, ancestors)?;
            inherited.extend(
                base.as_object().ok_or("Extended configuration must be an object")?.clone(),
            );
        }
        inherited.extend(value.as_object().ok_or("Lint configuration must be an object")?.clone());
        value = inherited.into();
    }
    ancestors.pop();
    Ok(value)
}
