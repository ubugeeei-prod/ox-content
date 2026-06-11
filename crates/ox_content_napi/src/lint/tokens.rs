use super::cjk::segment_cjk_run;
use super::dictionary::is_known_token;
use super::latin::assign_latin_languages;
use super::utils::*;
use super::*;

pub(super) fn collect_tokens(
    masked_line: &str,
    languages: &[String],
    dictionary: &DictionaryBundle,
) -> Vec<Token> {
    let latin_tokens = collect_latin_tokens(masked_line, languages, dictionary);
    let mut cjk_tokens = Vec::new();

    if let Some(cjk_run_pattern) = CJK_RUN_PATTERN.as_ref() {
        for value in cjk_run_pattern.find_iter(masked_line) {
            let start = byte_to_char_index(masked_line, value.start());
            cjk_tokens.extend(collect_cjk_tokens(value.as_str(), start, languages, dictionary));
        }
    }

    merge_tokens(latin_tokens, cjk_tokens)
}

fn merge_tokens(left: Vec<Token>, right: Vec<Token>) -> Vec<Token> {
    let mut merged = Vec::with_capacity(left.len() + right.len());
    let mut left_iter = left.into_iter().peekable();
    let mut right_iter = right.into_iter().peekable();

    while left_iter.peek().is_some() || right_iter.peek().is_some() {
        let take_left = match (left_iter.peek(), right_iter.peek()) {
            (Some(left_token), Some(right_token)) => left_token.start <= right_token.start,
            (Some(_), None) => true,
            _ => false,
        };

        if take_left {
            if let Some(token) = left_iter.next() {
                merged.push(token);
            }
        } else if let Some(token) = right_iter.next() {
            merged.push(token);
        }
    }

    merged
}

fn collect_latin_tokens(
    masked_line: &str,
    languages: &[String],
    dictionary: &DictionaryBundle,
) -> Vec<Token> {
    let latin_languages = languages
        .iter()
        .filter(|language| language.as_str() != "ja" && language.as_str() != "zh")
        .cloned()
        .collect::<Vec<_>>();

    if latin_languages.is_empty() {
        return Vec::new();
    }

    let fallback_language = latin_languages[0].clone();
    let mut tokens = Vec::new();

    if let Some(latin_word_pattern) = LATIN_WORD_PATTERN.as_ref() {
        for value in latin_word_pattern.find_iter(masked_line) {
            let text = value.as_str().to_string();
            let start = byte_to_char_index(masked_line, value.start());
            let end = start + count_code_points(value.as_str());
            tokens.push(Token { end, language: fallback_language.clone(), start, text });
        }
    }

    assign_latin_languages(tokens, &latin_languages, dictionary, &fallback_language)
}

fn collect_cjk_tokens(
    run: &str,
    start_offset: usize,
    languages: &[String],
    dictionary: &DictionaryBundle,
) -> Vec<Token> {
    let has_kana =
        run.chars().any(|value| matches!(value, '\u{3040}'..='\u{309F}' | '\u{30A0}'..='\u{30FF}'));

    let mut candidates = Vec::new();

    if has_kana && languages.iter().any(|language| language == "ja") {
        candidates.push("ja".to_string());
    } else {
        if languages.iter().any(|language| language == "zh") {
            candidates.push("zh".to_string());
        }
        if languages.iter().any(|language| language == "ja") {
            candidates.push("ja".to_string());
        }
    }

    if candidates.is_empty() {
        return Vec::new();
    }

    let mut best_candidate: Option<(usize, Vec<Token>)> = None;

    for language in candidates {
        let tokens = segment_cjk_run(run, start_offset, &language, dictionary);
        let known_count = tokens.iter().filter(|token| is_known_token(token, dictionary)).count();

        match &best_candidate {
            Some((best_known_count, best_tokens))
                if known_count < *best_known_count
                    || (known_count == *best_known_count && tokens.len() >= best_tokens.len()) => {}
            _ => best_candidate = Some((known_count, tokens)),
        }
    }

    best_candidate.map_or_else(Vec::new, |(_, tokens)| tokens)
}
