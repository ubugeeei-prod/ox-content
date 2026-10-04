use super::cjk::segment_cjk_run;
use super::dictionary::is_known_token;
use super::latin::assign_latin_languages;
use super::utils::*;
use super::*;

pub(super) fn collect_tokens(
    masked_line: &str,
    options: &InternalMarkdownLintOptions,
    dictionary: &DictionaryBundle,
    tokens: &mut Vec<Token>,
) {
    tokens.clear();
    collect_latin_tokens(masked_line, &options.latin_languages, dictionary, tokens);
    if !options.languages.iter().any(|language| language == "ja" || language == "zh") {
        return;
    }
    let latin_count = tokens.len();

    if let Some(cjk_run_pattern) = CJK_RUN_PATTERN.as_ref() {
        let mut cursor = CharIndexCursor::new(masked_line);
        for value in cjk_run_pattern.find_iter(masked_line) {
            let start = cursor.char_index(value.start());
            tokens.extend(collect_cjk_tokens(
                value.as_str(),
                start,
                &options.languages,
                dictionary,
            ));
        }
    }

    if latin_count > 0 && tokens.len() > latin_count {
        tokens.sort_unstable_by_key(|token| token.start);
    }
}

fn collect_latin_tokens(
    masked_line: &str,
    languages: &[String],
    dictionary: &DictionaryBundle,
    tokens: &mut Vec<Token>,
) {
    let Some(language) = languages.first() else {
        return;
    };
    let fallback_language = CompactString::from(language.as_str());

    if let Some(latin_word_pattern) = LATIN_WORD_PATTERN.as_ref() {
        let mut cursor = CharIndexCursor::new(masked_line);
        for value in latin_word_pattern.find_iter(masked_line) {
            let text = CompactString::from(value.as_str());
            let start = cursor.char_index(value.start());
            let end = start + count_code_points(value.as_str());
            tokens.push(Token { end, language: fallback_language.clone(), start, text });
        }
    }

    assign_latin_languages(tokens, languages, dictionary, &fallback_language);
}

fn collect_cjk_tokens(
    run: &str,
    start_offset: usize,
    languages: &[String],
    dictionary: &DictionaryBundle,
) -> Vec<Token> {
    let has_kana =
        run.chars().any(|value| matches!(value, '\u{3040}'..='\u{309F}' | '\u{30A0}'..='\u{30FF}'));

    let japanese = languages.iter().any(|language| language == "ja");
    let chinese = languages.iter().any(|language| language == "zh");
    let candidates = if has_kana && japanese {
        [Some("ja"), None]
    } else {
        [chinese.then_some("zh"), japanese.then_some("ja")]
    };

    let mut best_candidate: Option<(usize, Vec<Token>)> = None;

    for language in candidates.into_iter().flatten() {
        let tokens = segment_cjk_run(run, start_offset, language, dictionary);
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
