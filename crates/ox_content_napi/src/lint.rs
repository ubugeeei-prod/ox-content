#![cfg_attr(test, allow(dead_code))]

use rustc_hash::{FxHashMap, FxHashSet};
use std::sync::LazyLock;

use compact_str::CompactString;
use napi_derive::napi;
use regex::Regex;
use serde::Deserialize;
use unicode_normalization::UnicodeNormalization;

mod cjk;
mod diagnostics;
mod dictionary;
mod latin;
mod mask;
mod tokens;
mod utils;

use diagnostics::*;
use dictionary::*;
use mask::*;
use tokens::*;
use utils::*;

const SUPPORTED_MARKDOWN_LINT_LANGUAGES: [&str; 6] = ["en", "ja", "zh", "fr", "de", "pl"];
const DEFAULT_LANGUAGES: [&str; 1] = ["en"];

static HEADING_PATTERN: LazyLock<Option<Regex>> =
    LazyLock::new(|| Regex::new(r"^\s{0,3}(#{1,6})[ \t]+(.+?)\s*#*\s*$").ok());
static FENCE_PATTERN: LazyLock<Option<Regex>> =
    LazyLock::new(|| Regex::new(r"^\s{0,3}(`{3,}|~{3,})").ok());
static LIST_PREFIX_PATTERN: LazyLock<Option<Regex>> =
    LazyLock::new(|| Regex::new(r"^\s{0,3}(?:#{1,6}[ \t]+|>\s?|(?:[-*+]|(?:\d+[.)]))[ \t]+)").ok());
static REFERENCE_DEFINITION_PATTERN: LazyLock<Option<Regex>> =
    LazyLock::new(|| Regex::new(r"^\s{0,3}\[[^\]]+\]:").ok());
static URL_PATTERN: LazyLock<Option<Regex>> =
    LazyLock::new(|| Regex::new(r"(?iu)\b(?:https?://|mailto:|www\.)\S+").ok());
static HTML_TAG_PATTERN: LazyLock<Option<Regex>> =
    LazyLock::new(|| Regex::new(r"(?u)</?[\p{L}!][^>]*>").ok());
static FOOTNOTE_PATTERN: LazyLock<Option<Regex>> =
    LazyLock::new(|| Regex::new(r"\[\^[^\]]+\]").ok());
static LATIN_WORD_PATTERN: LazyLock<Option<Regex>> =
    LazyLock::new(|| Regex::new(r"(?u)\p{Latin}+(?:['’-]\p{Latin}+)*").ok());
static CJK_RUN_PATTERN: LazyLock<Option<Regex>> =
    LazyLock::new(|| Regex::new(r"(?u)[\p{Han}\p{Hiragana}\p{Katakana}ー]+").ok());

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

struct MarkdownLintState {
    diagnostics: Vec<JsMarkdownLintDiagnostic>,
    masked_lines: Vec<String>,
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

fn normalize_lint_options(options: Option<JsMarkdownLintOptions>) -> InternalMarkdownLintOptions {
    let options = options.unwrap_or_default();
    let languages = options
        .languages
        .unwrap_or_else(|| DEFAULT_LANGUAGES.iter().map(ToString::to_string).collect())
        .into_iter()
        .filter(|language| is_supported_language(language))
        .collect::<Vec<_>>();
    let dictionary = options.dictionary.unwrap_or_default();
    let rules = options.rules.unwrap_or_default();

    InternalMarkdownLintOptions {
        dictionary: InternalMarkdownLintDictionary {
            words: dictionary.words.unwrap_or_default(),
            by_language: dictionary
                .by_language
                .unwrap_or_default()
                .into_iter()
                .filter(|entry| is_supported_language(&entry.language))
                .map(|entry| (entry.language, entry.words))
                .collect(),
            ignored_words: dictionary.ignored_words.unwrap_or_default(),
        },
        languages: if languages.is_empty() {
            DEFAULT_LANGUAGES.iter().map(ToString::to_string).collect()
        } else {
            dedupe_strings(languages)
        },
        rules: InternalMarkdownLintRules {
            duplicate_headings: rules.duplicate_headings.unwrap_or(true),
            heading_increment: rules.heading_increment.unwrap_or(true),
            max_consecutive_blank_lines: rules.max_consecutive_blank_lines.unwrap_or(1),
            repeated_punctuation: rules.repeated_punctuation.unwrap_or(true),
            repeated_words: rules.repeated_words.unwrap_or(true),
            spellcheck: rules.spellcheck.unwrap_or(true),
            trailing_spaces: rules.trailing_spaces.unwrap_or(true),
        },
    }
}

fn collect_markdown_lint_state(
    source: &str,
    normalized_options: &InternalMarkdownLintOptions,
    dictionary: &DictionaryBundle,
) -> MarkdownLintState {
    let mut diagnostics = Vec::new();
    let mut masked_lines = Vec::new();
    let mut seen_headings = FxHashMap::default();

    let mut blank_line_streak = 0_u32;
    let mut html_comment_open = false;
    let mut in_fence = false;
    let mut fence_char = '\0';
    let mut fence_length = 0_usize;
    let mut frontmatter_open = false;
    let mut frontmatter_checked = false;
    let mut previous_heading_depth = 0_usize;

    for (index, raw_line) in source.split('\n').enumerate() {
        let line = raw_line.strip_suffix('\r').unwrap_or(raw_line);
        let line_number = index + 1;
        let trimmed = line.trim();

        if !frontmatter_checked {
            frontmatter_checked = true;
            if trimmed == "---" {
                frontmatter_open = true;
                masked_lines.push(create_skipped_line_mask(line));
                continue;
            }
        }

        if frontmatter_open {
            if trimmed == "---" || trimmed == "..." {
                frontmatter_open = false;
            }
            masked_lines.push(create_skipped_line_mask(line));
            continue;
        }

        if html_comment_open {
            if line.contains("-->") {
                html_comment_open = false;
            }
            masked_lines.push(create_skipped_line_mask(line));
            continue;
        }

        if !in_fence && trimmed.starts_with("<!--") {
            if !trimmed.contains("-->") {
                html_comment_open = true;
            }
            masked_lines.push(create_skipped_line_mask(line));
            continue;
        }

        if in_fence {
            if is_fence_close(line, fence_char, fence_length) {
                in_fence = false;
                fence_char = '\0';
                fence_length = 0;
            }
            masked_lines.push(create_skipped_line_mask(line));
            continue;
        }

        if let Some(fence_pattern) = FENCE_PATTERN.as_ref() {
            if let Some(fence_match) = fence_pattern.find(line) {
                let fence = &line[fence_match.start()..fence_match.end()];
                in_fence = true;
                fence_char = fence.chars().next().unwrap_or('\0');
                fence_length = fence.chars().count();
                masked_lines.push(create_skipped_line_mask(line));
                continue;
            }
        }

        if normalized_options.rules.trailing_spaces {
            let trailing_length = get_trailing_whitespace_length(line);
            if trailing_length > 0 {
                let line_length = count_code_points(line);
                let start_column = line_length.saturating_sub(trailing_length) + 1;
                diagnostics.push(create_diagnostic(
                    "trailing-spaces",
                    "Trailing whitespace is not allowed.".to_string(),
                    line_number,
                    start_column,
                    line_length + 1,
                    None,
                    None,
                ));
            }
        }

        if trimmed.is_empty() {
            blank_line_streak += 1;
            if blank_line_streak > normalized_options.rules.max_consecutive_blank_lines {
                let limit = normalized_options.rules.max_consecutive_blank_lines;
                diagnostics.push(create_diagnostic(
                    "max-consecutive-blank-lines",
                    format!(
                        "More than {limit} blank line{} in a row.",
                        if limit == 1 { "" } else { "s" }
                    ),
                    line_number,
                    1,
                    1,
                    None,
                    None,
                ));
            }
            masked_lines.push(create_skipped_line_mask(line));
            continue;
        }

        blank_line_streak = 0;

        if let Some(captures) = HEADING_PATTERN.as_ref().and_then(|pattern| pattern.captures(line))
        {
            let depth = captures.get(1).map_or(0, |value| value.as_str().chars().count());
            let heading_text = captures
                .get(2)
                .map(|value| collapse_whitespace(&get_visible_text(value.as_str())))
                .unwrap_or_default();
            let normalized_heading = normalize_latin_word(&heading_text);

            if normalized_options.rules.heading_increment
                && previous_heading_depth > 0
                && depth > previous_heading_depth + 1
            {
                diagnostics.push(create_diagnostic(
                    "heading-increment",
                    format!("Heading depth jumps from h{previous_heading_depth} to h{depth}."),
                    line_number,
                    1,
                    depth + 1,
                    None,
                    None,
                ));
            }

            previous_heading_depth = depth;

            if normalized_options.rules.duplicate_headings && !normalized_heading.is_empty() {
                if let Some(first_seen_line) = seen_headings.get(&normalized_heading) {
                    diagnostics.push(create_diagnostic(
                        "duplicate-heading",
                        format!("Heading text duplicates the heading on line {first_seen_line}."),
                        line_number,
                        1,
                        count_code_points(line) + 1,
                        None,
                        None,
                    ));
                } else {
                    seen_headings.insert(normalized_heading, line_number);
                }
            }
        }

        if REFERENCE_DEFINITION_PATTERN.as_ref().is_some_and(|pattern| pattern.is_match(line))
            || is_indented_code_block_line(line)
        {
            masked_lines.push(create_skipped_line_mask(line));
            continue;
        }

        let masked_line = mask_markdown_line(line);
        masked_lines.push(masked_line.clone());

        if normalized_options.rules.repeated_punctuation {
            diagnostics.extend(collect_repeated_punctuation_diagnostics(line_number, &masked_line));
        }

        let tokens = collect_tokens(&masked_line, &normalized_options.languages, dictionary);

        if normalized_options.rules.repeated_words {
            let mut previous_comparable_token: Option<&Token> = None;

            for token in &tokens {
                if should_ignore_repeated_word_token(token) {
                    continue;
                }

                if let Some(previous_token) = previous_comparable_token {
                    if normalize_comparable_word(&previous_token.text)
                        == normalize_comparable_word(&token.text)
                    {
                        diagnostics.push(create_diagnostic(
                            "repeated-word",
                            format!("Repeated word \"{}\" looks accidental.", token.text),
                            line_number,
                            token.start + 1,
                            token.end + 1,
                            Some(token.language.clone()),
                            None,
                        ));
                    }
                }

                previous_comparable_token = Some(token);
            }
        }

        if normalized_options.rules.spellcheck {
            for token in &tokens {
                if !should_spellcheck_token(token, dictionary) || is_known_token(token, dictionary)
                {
                    continue;
                }

                let suggestions = if token.language == "ja" || token.language == "zh" {
                    None
                } else {
                    let values =
                        suggest_latin_words(&token.text, &dictionary.latin_suggestion_words);
                    if values.is_empty() {
                        None
                    } else {
                        Some(values)
                    }
                };

                diagnostics.push(create_diagnostic(
                    "spellcheck",
                    format!("Unknown {} word \"{}\".", token.language, token.text),
                    line_number,
                    token.start + 1,
                    token.end + 1,
                    Some(token.language.clone()),
                    suggestions,
                ));
            }
        }
    }

    MarkdownLintState { diagnostics, masked_lines }
}
