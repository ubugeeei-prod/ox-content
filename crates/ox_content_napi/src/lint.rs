#![cfg_attr(test, allow(dead_code))]

use rustc_hash::{FxHashMap, FxHashSet};
use std::sync::LazyLock;

use compact_str::CompactString;
use napi_derive::napi;
use serde::Deserialize;
use unicode_normalization::UnicodeNormalization;

mod cjk;
mod diagnostics;
mod dictionary;
mod latin;
mod mask;
mod options;
mod patterns;
mod state;
mod tokens;
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

#[napi(object)]
#[derive(Clone)]
pub struct JsMarkdownLintLanguageWords {
    pub language: String,
    pub words: Vec<String>,
}

#[napi(object)]
#[derive(Default, Clone)]
pub struct JsMarkdownLintDictionaryOptions {
    pub words: Option<Vec<String>>,
    pub by_language: Option<Vec<JsMarkdownLintLanguageWords>>,
    pub ignored_words: Option<Vec<String>>,
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
}

#[napi(object)]
#[derive(Default, Clone)]
pub struct JsMarkdownLintOptions {
    pub languages: Option<Vec<String>>,
    pub rules: Option<JsMarkdownLintRuleOptions>,
    pub dictionary: Option<JsMarkdownLintDictionaryOptions>,
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
}

#[napi(object)]
pub struct JsMarkdownLintResult {
    pub diagnostics: Vec<JsMarkdownLintDiagnostic>,
    pub error_count: u32,
    pub warning_count: u32,
    pub info_count: u32,
    pub masked_document: String,
}

#[derive(Clone)]
struct InternalMarkdownLintOptions {
    dictionary: InternalMarkdownLintDictionary,
    languages: Vec<String>,
    rules: InternalMarkdownLintRules,
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
}

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
    language: String,
    start: usize,
    text: String,
}

#[napi(js_name = "lintMarkdown")]
pub fn lint_markdown(
    source: String,
    options: Option<JsMarkdownLintOptions>,
) -> JsMarkdownLintResult {
    let normalized_options = normalize_lint_options(options);
    let dictionary = create_dictionary_bundle(&normalized_options);
    let state = collect_markdown_lint_state(&source, &normalized_options, &dictionary);
    summarize_diagnostics(sort_diagnostics(state.diagnostics), state.masked_lines.join("\n"))
}

#[napi(js_name = "lintMarkdownDocuments")]
pub fn lint_markdown_documents(
    sources: Vec<String>,
    options: Option<JsMarkdownLintOptions>,
) -> Vec<JsMarkdownLintResult> {
    let normalized_options = normalize_lint_options(options);
    let dictionary = create_dictionary_bundle(&normalized_options);

    sources
        .into_iter()
        .map(|source| {
            let state = collect_markdown_lint_state(&source, &normalized_options, &dictionary);
            summarize_diagnostics(
                sort_diagnostics(state.diagnostics),
                state.masked_lines.join("\n"),
            )
        })
        .collect()
}
