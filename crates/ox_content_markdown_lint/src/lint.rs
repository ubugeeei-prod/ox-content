#![cfg_attr(test, allow(dead_code))]

use rustc_hash::{FxHashMap, FxHashSet};
use std::sync::LazyLock;

use compact_str::CompactString;
use serde::Deserialize;
use unicode_normalization::UnicodeNormalization;

mod cjk;
mod controls;
mod diagnostics;
mod dictionary;
mod extensions;
mod fixes;
mod latin;
mod mask;
mod options;
mod patterns;
mod prose;
mod source;
mod state;
mod structure;
mod syntax;
mod tokens;
mod types;
mod utils;

use diagnostics::*;
use dictionary::*;
use mask::*;
use options::normalize_lint_options;
use patterns::*;
use state::collect_markdown_lint_state;
use tokens::*;
use utils::*;

const SUPPORTED_MARKDOWN_LINT_LANGUAGES: [&str; 6] = ["en", "ja", "zh", "fr", "de", "pl"];
const DEFAULT_LANGUAGES: [&str; 1] = ["en"];

static LINT_DICTIONARY_DATA: LazyLock<Option<LintDictionaryData>> = LazyLock::new(|| {
    serde_json::from_str(include_str!(
        "../../../npm/vite-plugin-ox-content/src/lint-dictionaries.json"
    ))
    .ok()
});

#[derive(Deserialize)]
struct LintDictionaryData {
    global: Vec<String>,
    #[serde(rename = "byLanguage")]
    by_language: FxHashMap<String, Vec<String>>,
}

#[derive(Default)]
struct PreparedLintDictionaryData {
    global_words: FxHashSet<String>,
    by_language: FxHashMap<String, PreparedLanguageDictionary>,
}

struct PreparedLanguageDictionary {
    has_base_words: bool,
    cjk_segment_words: Vec<SegmentWord>,
    words: FxHashSet<String>,
}

#[derive(Clone)]
struct SegmentWord {
    char_len: usize,
    text: String,
}

static PREPARED_LINT_DICTIONARY_DATA: LazyLock<PreparedLintDictionaryData> = LazyLock::new(|| {
    let Some(dictionary_data) = LINT_DICTIONARY_DATA.as_ref() else {
        return PreparedLintDictionaryData::default();
    };

    let global_words = dictionary_data
        .global
        .iter()
        .map(|word| normalize_word_for_set(word))
        .filter(|word| !word.is_empty())
        .collect::<FxHashSet<_>>();

    let by_language = SUPPORTED_MARKDOWN_LINT_LANGUAGES
        .iter()
        .map(|language| {
            let words = dictionary_data
                .by_language
                .get(*language)
                .into_iter()
                .flatten()
                .map(|word| normalize_word_for_set(word))
                .filter(|word| !word.is_empty())
                .collect::<FxHashSet<_>>();

            let mut cjk_segment_words = words
                .iter()
                .chain(global_words.iter())
                .filter(|word| word.chars().any(is_cjk_char))
                .map(|word| SegmentWord {
                    char_len: count_code_points(word),
                    text: (*word).clone(),
                })
                .collect::<Vec<_>>();
            sort_and_dedupe_segment_words(&mut cjk_segment_words);

            (
                (*language).to_string(),
                PreparedLanguageDictionary {
                    has_base_words: !words.is_empty(),
                    cjk_segment_words,
                    words,
                },
            )
        })
        .collect();

    PreparedLintDictionaryData { global_words, by_language }
});

pub use extensions::*;
pub use types::*;

#[derive(Clone)]
struct InternalMarkdownLintOptions {
    dictionary: InternalMarkdownLintDictionary,
    languages: Vec<String>,
    mdx: bool,
    rules: InternalMarkdownLintRules,
    text_rules: MarkdownLintTextRules,
    severities: FxHashMap<String, MarkdownLintSeverity>,
    terminology: Option<aho_corasick::AhoCorasick>,
}

#[derive(Clone, Default)]
struct InternalMarkdownLintDictionary {
    words: Vec<String>,
    by_language: FxHashMap<String, Vec<String>>,
    ignored_words: Vec<String>,
}

#[derive(Clone)]
struct InternalMarkdownLintRules {
    duplicate_headings: bool,
    heading_increment: bool,
    max_consecutive_blank_lines: u32,
    repeated_punctuation: bool,
    repeated_words: bool,
    spellcheck: bool,
    trailing_spaces: bool,
    structure: MarkdownLintRuleOptions,
}

#[derive(Default)]
struct DictionaryBundle {
    active_languages: FxHashSet<String>,
    cjk_segment_words: FxHashMap<String, Vec<SegmentWord>>,
    extra_by_language: FxHashMap<String, FxHashSet<String>>,
    extra_global_words: FxHashSet<String>,
    ignored_words: FxHashSet<String>,
    latin_words: FxHashSet<String>,
    latin_suggestion_words: Vec<String>,
}

#[derive(Clone)]
struct Token {
    end: usize,
    language: CompactString,
    start: usize,
    text: CompactString,
}

/// Reusable options and dictionaries, shared without locks across worker threads.
pub struct MarkdownLinter {
    options: InternalMarkdownLintOptions,
    dictionary: DictionaryBundle,
}

impl MarkdownLinter {
    pub fn new(options: Option<MarkdownLintOptions>) -> Self {
        let options = normalize_lint_options(options);
        let dictionary = if options.rules.spellcheck
            || (options.rules.repeated_words
                && options.languages.iter().any(|v| v == "ja" || v == "zh"))
        {
            create_dictionary_bundle(&options)
        } else {
            DictionaryBundle::default()
        };
        Self { options, dictionary }
    }

    pub fn lint(&self, source: &str) -> MarkdownLintResult {
        self.run(source, true)
    }

    /// Don't allocate an external spellcheck mask for CLI/editor consumers.
    pub fn lint_without_mask(&self, source: &str) -> MarkdownLintResult {
        self.run(source, false)
    }

    fn run(&self, source: &str, include_mask: bool) -> MarkdownLintResult {
        let state =
            collect_markdown_lint_state(source, &self.options, &self.dictionary, include_mask);
        summarize_diagnostics(sort_diagnostics(state.diagnostics), state.masked_document)
    }

    pub fn fix(&self, source: &str) -> MarkdownLintFixResult {
        let result = self.lint_without_mask(source);
        let (output, applied_fixes) = fixes::apply(source, &result.diagnostics);
        let result = if applied_fixes == 0 { result } else { self.lint_without_mask(&output) };
        MarkdownLintFixResult { output, applied_fixes, result }
    }
}

pub fn lint_markdown(source: &str, options: Option<MarkdownLintOptions>) -> MarkdownLintResult {
    MarkdownLinter::new(options).lint(source)
}

pub fn fix_markdown(source: &str, options: Option<MarkdownLintOptions>) -> MarkdownLintFixResult {
    MarkdownLinter::new(options).fix(source)
}

/// Uses Rayon workers and preserves input order regardless of scheduling.
pub fn lint_markdown_documents(
    sources: &[String],
    options: Option<MarkdownLintOptions>,
) -> Vec<MarkdownLintResult> {
    use rayon::prelude::*;
    let linter = MarkdownLinter::new(options);
    if sources.len() < 8 {
        sources.iter().map(|source| linter.lint(source)).collect()
    } else {
        sources.par_iter().map(|source| linter.lint(source)).collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn lint_with_mdx(source: &str, mdx: bool) -> MarkdownLintResult {
        lint_markdown(
            source,
            Some(MarkdownLintOptions {
                dictionary: Some(MarkdownLintDictionaryOptions {
                    words: Some(
                        ["Guide", "Visible", "copy", "Hello"]
                            .into_iter()
                            .map(ToString::to_string)
                            .collect(),
                    ),
                    ..MarkdownLintDictionaryOptions::default()
                }),
                mdx: Some(mdx),
                ..MarkdownLintOptions::default()
            }),
        )
    }

    #[test]
    fn mdx_lint_masks_syntax_but_keeps_visible_copy() {
        let source = concat!(
            "import Card from './Card'\n\n",
            "export const meta = { title: 'Guide' }\n\n",
            "# Guide\n\n",
            "<Card tone={theme.primary}>Visible wrld {user.name}</Card>\n",
        );
        let result = lint_with_mdx(source, true);

        assert_eq!(result.diagnostics.len(), 1);
        assert_eq!(result.diagnostics[0].rule_id, "spellcheck");
        assert!(result.diagnostics[0].message.contains("wrld"));
        assert_eq!(result.diagnostics[0].line, 7);
    }

    #[test]
    fn explicit_mdx_opt_out_keeps_plain_markdown_linting() {
        let result = lint_with_mdx("import Card from './Card'\n", false);
        assert!(
            result.diagnostics.iter().any(|diagnostic| diagnostic.message.contains("Card")),
            "explicit mdx=false must preserve the Markdown path"
        );
    }
}
