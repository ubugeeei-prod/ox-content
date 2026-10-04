use crate::Result;
use ox_content_markdown_lint::{
    MarkdownLintDictionaryOptions, MarkdownLintLanguageWords, MarkdownLintOptions,
    MarkdownLintRuleOptions,
};
use serde::Deserialize;

#[derive(Default, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Config {
    #[serde(default)]
    pub include: Vec<String>,
    #[serde(default)]
    pub ignore: Vec<String>,
    languages: Option<Vec<String>>,
    rules: Option<Rules>,
    dictionary: Option<Dictionary>,
}

#[derive(Default, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct Rules {
    duplicate_headings: Option<bool>,
    heading_increment: Option<bool>,
    max_consecutive_blank_lines: Option<u32>,
    repeated_punctuation: Option<bool>,
    repeated_words: Option<bool>,
    spellcheck: Option<bool>,
    trailing_spaces: Option<bool>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct Dictionary {
    words: Option<Vec<String>>,
    ignored_words: Option<Vec<String>>,
    by_language: Option<Vec<LanguageWords>>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct LanguageWords {
    language: String,
    words: Vec<String>,
}

impl Config {
    pub fn read(path: &str) -> Result<Self> {
        let source = std::fs::read_to_string(path)?;
        serde_json::from_str(&source)
            .map_err(|error| format!("Invalid lint rule or configuration: {error}").into())
    }

    pub fn native(&self, spellcheck: bool, mdx: bool) -> MarkdownLintOptions {
        let defaults = Rules::default();
        let rules = self.rules.as_ref().unwrap_or(&defaults);
        MarkdownLintOptions {
            languages: self.languages.clone(),
            mdx: Some(mdx),
            rules: Some(MarkdownLintRuleOptions {
                duplicate_headings: rules.duplicate_headings,
                heading_increment: rules.heading_increment,
                max_consecutive_blank_lines: rules.max_consecutive_blank_lines,
                repeated_punctuation: rules.repeated_punctuation,
                repeated_words: rules.repeated_words,
                spellcheck: Some(spellcheck || rules.spellcheck.unwrap_or(false)),
                trailing_spaces: rules.trailing_spaces,
            }),
            dictionary: self.dictionary.as_ref().map(|dictionary| MarkdownLintDictionaryOptions {
                words: dictionary.words.clone(),
                ignored_words: dictionary.ignored_words.clone(),
                by_language: dictionary.by_language.as_ref().map(|entries| {
                    entries
                        .iter()
                        .map(|entry| MarkdownLintLanguageWords {
                            language: entry.language.clone(),
                            words: entry.words.clone(),
                        })
                        .collect()
                }),
            }),
        }
    }
}
