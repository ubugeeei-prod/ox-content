use super::utils::*;
use super::*;

pub(super) fn create_dictionary_bundle(options: &InternalMarkdownLintOptions) -> DictionaryBundle {
    let extra_global_words = options
        .dictionary
        .words
        .iter()
        .map(|word| normalize_word_for_set(word))
        .filter(|word| !word.is_empty())
        .collect::<FxHashSet<_>>();

    let extra_by_language = options
        .dictionary
        .by_language
        .iter()
        .map(|(language, words)| {
            (
                language.clone(),
                words
                    .iter()
                    .map(|word| normalize_word_for_set(word))
                    .filter(|word| !word.is_empty())
                    .collect::<FxHashSet<_>>(),
            )
        })
        .filter(|(_, words)| !words.is_empty())
        .collect::<FxHashMap<_, _>>();

    let mut latin_words = FxHashSet::default();
    let mut latin_suggestion_words = FxHashSet::default();

    for language in &options.languages {
        if language == "ja" || language == "zh" {
            continue;
        }

        if let Some(words) = PREPARED_LINT_DICTIONARY_DATA.by_language.get(language.as_str()) {
            for word in &words.words {
                latin_words.insert(word.clone());
                latin_suggestion_words.insert(word.clone());
            }
        }

        for word in &PREPARED_LINT_DICTIONARY_DATA.global_words {
            latin_words.insert(word.clone());
            latin_suggestion_words.insert(word.clone());
        }

        if let Some(words) = extra_by_language.get(language.as_str()) {
            for word in words {
                latin_words.insert(word.clone());
                latin_suggestion_words.insert(word.clone());
            }
        }
    }

    for word in &extra_global_words {
        latin_words.insert(word.clone());
        latin_suggestion_words.insert(word.clone());
    }

    let ignored_words =
        options.dictionary.ignored_words.iter().map(|word| normalize_word_for_set(word)).collect();

    let active_languages = options
        .languages
        .iter()
        .filter(|language| {
            PREPARED_LINT_DICTIONARY_DATA
                .by_language
                .get((*language).as_str())
                .is_some_and(|entry| entry.has_base_words)
                || extra_by_language
                    .get((*language).as_str())
                    .is_some_and(|words| !words.is_empty())
        })
        .cloned()
        .collect();

    let mut cjk_segment_words = FxHashMap::default();

    for language in
        options.languages.iter().filter(|language| *language == "ja" || *language == "zh")
    {
        let mut words = PREPARED_LINT_DICTIONARY_DATA
            .by_language
            .get(language.as_str())
            .map_or_else(Vec::new, |entry| entry.cjk_segment_words.clone());

        words.extend(
            extra_global_words
                .iter()
                .filter(|word| word.chars().any(is_cjk_char))
                .map(|word| SegmentWord { char_len: count_code_points(word), text: word.clone() }),
        );

        if let Some(extra_words) = extra_by_language.get(language.as_str()) {
            words.extend(
                extra_words.iter().filter(|word| word.chars().any(is_cjk_char)).map(|word| {
                    SegmentWord { char_len: count_code_points(word), text: word.clone() }
                }),
            );
        }

        sort_and_dedupe_segment_words(&mut words);
        if !words.is_empty() {
            cjk_segment_words.insert(language.clone(), words);
        }
    }

    DictionaryBundle {
        active_languages,
        cjk_segment_words,
        extra_by_language,
        extra_global_words,
        ignored_words,
        latin_words,
        latin_suggestion_words: {
            let mut values = latin_suggestion_words.into_iter().collect::<Vec<_>>();
            values.sort();
            values
        },
    }
}

pub(super) fn language_contains_word(
    language: &str,
    normalized_word: &str,
    dictionary: &DictionaryBundle,
) -> bool {
    dictionary.extra_global_words.contains(normalized_word)
        || PREPARED_LINT_DICTIONARY_DATA.global_words.contains(normalized_word)
        || dictionary
            .extra_by_language
            .get(language)
            .is_some_and(|words| words.contains(normalized_word))
        || PREPARED_LINT_DICTIONARY_DATA
            .by_language
            .get(language)
            .is_some_and(|words| words.words.contains(normalized_word))
}

pub(super) fn should_spellcheck_token(token: &Token, dictionary: &DictionaryBundle) -> bool {
    let normalized = normalize_word_for_set(&token.text);

    if normalized.is_empty() || dictionary.ignored_words.contains(&normalized) {
        return false;
    }

    if !dictionary.active_languages.contains(&token.language) {
        return false;
    }

    if token.text.contains('_')
        || token.text.contains('/')
        || token.text.contains('\\')
        || token.text.chars().any(char::is_numeric)
    {
        return false;
    }

    if is_uppercase_token(&token.text) {
        return false;
    }

    if token.language == "ja" || token.language == "zh" {
        if token.language == "ja" && token.text.chars().all(is_hiragana) {
            return count_code_points(&token.text) > 2;
        }

        return count_code_points(&token.text) > 1;
    }

    normalize_comparable_word(&token.text).chars().count() > 2
}

pub(super) fn is_known_token(token: &Token, dictionary: &DictionaryBundle) -> bool {
    let normalized = normalize_word_for_set(&token.text);
    if normalized.is_empty() || dictionary.ignored_words.contains(&normalized) {
        return true;
    }

    if token.language == "ja" || token.language == "zh" {
        return language_contains_word(token.language.as_str(), &normalized, dictionary);
    }

    dictionary.latin_words.contains(&normalized)
}

pub(super) fn suggest_latin_words(word: &str, candidates: &[String]) -> Vec<String> {
    let normalized_word = normalize_latin_word(word);
    let first_char = normalized_word.chars().next();
    let mut suggestions = candidates
        .iter()
        .filter(|candidate| {
            let candidate_length = candidate.chars().count();
            let word_length = normalized_word.chars().count();
            candidate_length.abs_diff(word_length) <= 2
        })
        .filter(|candidate| candidate.chars().next() == first_char)
        .map(|candidate| (candidate.clone(), levenshtein(&normalized_word, candidate)))
        .filter(|(_, distance)| *distance <= 2)
        .collect::<Vec<_>>();

    suggestions.sort_by(|left, right| left.1.cmp(&right.1).then_with(|| left.0.cmp(&right.0)));

    dedupe_strings(suggestions.into_iter().take(3).map(|(candidate, _)| candidate).collect())
}
