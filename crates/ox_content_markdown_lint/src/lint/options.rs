use super::*;

pub(super) fn normalize_lint_options(
    options: Option<MarkdownLintOptions>,
) -> InternalMarkdownLintOptions {
    let options = options.unwrap_or_default();
    let languages = options
        .languages
        .unwrap_or_else(|| DEFAULT_LANGUAGES.iter().map(ToString::to_string).collect())
        .into_iter()
        .filter(|language| is_supported_language(language))
        .collect::<Vec<_>>();
    let dictionary = options.dictionary.unwrap_or_default();
    let rules = options.rules.unwrap_or_default();

    let mut text_rules = options.text_rules.unwrap_or_default();
    if let Some(terms) = &mut text_rules.terminology {
        terms.retain(|v| !v.term.is_empty() && v.term != v.replacement);
    }
    let terminology = text_rules.terminology.as_ref().filter(|v| !v.is_empty()).and_then(|terms| {
        aho_corasick::AhoCorasick::builder()
            .match_kind(aho_corasick::MatchKind::LeftmostLongest)
            .build(terms.iter().map(|v| &v.term))
            .ok()
    });

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
        mdx: options.mdx.unwrap_or(false),
        text_rules,
        terminology,
        severities: options
            .severities
            .unwrap_or_default()
            .into_iter()
            .map(|v| (v.rule_id, v.severity))
            .collect(),
        rules: InternalMarkdownLintRules {
            duplicate_headings: rules.duplicate_headings.unwrap_or(true),
            heading_increment: rules.heading_increment.unwrap_or(true),
            max_consecutive_blank_lines: rules.max_consecutive_blank_lines.unwrap_or(1),
            repeated_punctuation: rules.repeated_punctuation.unwrap_or(true),
            repeated_words: rules.repeated_words.unwrap_or(true),
            spellcheck: rules.spellcheck.unwrap_or(true),
            trailing_spaces: rules.trailing_spaces.unwrap_or(true),
            structure: rules,
        },
    }
}
