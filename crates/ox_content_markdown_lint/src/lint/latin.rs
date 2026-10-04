use super::dictionary::language_contains_word;
use super::utils::*;
use super::*;

pub(super) fn assign_latin_languages(
    tokens: &mut [Token],
    languages: &[String],
    dictionary: &DictionaryBundle,
    fallback_language: &str,
) {
    if languages.len() <= 1 {
        return;
    }
    let mut scores =
        languages.iter().map(|language| (language.as_str(), 0_usize)).collect::<FxHashMap<_, _>>();
    for token in tokens.iter_mut() {
        let normalized = normalize_word_for_lookup(&token.text);
        let mut matching = languages
            .iter()
            .map(String::as_str)
            .filter(|language| language_contains_word(language, &normalized, dictionary));
        if let Some(first) = matching.next() {
            if matching.next().is_none() {
                *scores.entry(first).or_default() += 1;
            }
            token.language = CompactString::from(first);
        } else {
            token.language = CompactString::from(
                infer_latin_language_from_characters(&token.text, languages).unwrap_or(""),
            );
        }
    }

    let dominant_language = scores
        .into_iter()
        .max_by(|left, right| left.1.cmp(&right.1))
        .map_or(fallback_language, |(language, _)| language);
    for token in tokens.iter_mut().filter(|token| token.language.is_empty()) {
        token.language = CompactString::from(dominant_language);
    }
}

fn infer_latin_language_from_characters(word: &str, languages: &[String]) -> Option<&'static str> {
    if languages.iter().any(|language| language == "pl")
        && word.chars().any(|value| {
            matches!(
                value,
                'ą' | 'ć'
                    | 'ę'
                    | 'ł'
                    | 'ń'
                    | 'ó'
                    | 'ś'
                    | 'ź'
                    | 'ż'
                    | 'Ą'
                    | 'Ć'
                    | 'Ę'
                    | 'Ł'
                    | 'Ń'
                    | 'Ó'
                    | 'Ś'
                    | 'Ź'
                    | 'Ż'
            )
        })
    {
        return Some("pl");
    }

    if languages.iter().any(|language| language == "de")
        && word.chars().any(|value| matches!(value, 'ä' | 'ö' | 'ü' | 'ß' | 'Ä' | 'Ö' | 'Ü'))
    {
        return Some("de");
    }

    if languages.iter().any(|language| language == "fr")
        && word.chars().any(|value| {
            matches!(
                value,
                'à' | 'â'
                    | 'æ'
                    | 'ç'
                    | 'é'
                    | 'è'
                    | 'ê'
                    | 'ë'
                    | 'î'
                    | 'ï'
                    | 'ô'
                    | 'œ'
                    | 'ù'
                    | 'û'
                    | 'ü'
                    | 'ÿ'
                    | 'À'
                    | 'Â'
                    | 'Æ'
                    | 'Ç'
                    | 'É'
                    | 'È'
                    | 'Ê'
                    | 'Ë'
                    | 'Î'
                    | 'Ï'
                    | 'Ô'
                    | 'Œ'
                    | 'Ù'
                    | 'Û'
                    | 'Ü'
                    | 'Ÿ'
            )
        })
    {
        return Some("fr");
    }

    None
}
