use super::dictionary::language_contains_word;
use super::utils::*;
use super::*;

pub(super) fn assign_latin_languages(
    tokens: Vec<Token>,
    languages: &[String],
    dictionary: &DictionaryBundle,
    fallback_language: &str,
) -> Vec<Token> {
    let mut scores =
        languages.iter().map(|language| (language.clone(), 0_usize)).collect::<FxHashMap<_, _>>();

    let matching_languages = tokens
        .iter()
        .map(|token| get_matching_latin_languages(&token.text, languages, dictionary))
        .collect::<Vec<_>>();

    for matches in &matching_languages {
        if matches.len() == 1 {
            let language = &matches[0];
            *scores.entry(language.clone()).or_default() += 1;
        }
    }

    let dominant_language = scores
        .into_iter()
        .max_by(|left, right| left.1.cmp(&right.1))
        .map_or_else(|| fallback_language.to_string(), |(language, _)| language);

    tokens
        .into_iter()
        .zip(matching_languages)
        .map(|(token, matching_languages)| {
            let inferred_language = matching_languages
                .first()
                .cloned()
                .or_else(|| infer_latin_language_from_characters(&token.text, languages));

            Token {
                language: inferred_language.unwrap_or_else(|| dominant_language.clone()),
                ..token
            }
        })
        .collect()
}

fn get_matching_latin_languages(
    word: &str,
    languages: &[String],
    dictionary: &DictionaryBundle,
) -> Vec<String> {
    let normalized_word = normalize_word_for_set(word);

    languages
        .iter()
        .filter(|language| {
            language_contains_word((*language).as_str(), &normalized_word, dictionary)
        })
        .cloned()
        .collect()
}

fn infer_latin_language_from_characters(word: &str, languages: &[String]) -> Option<String> {
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
        return Some("pl".to_string());
    }

    if languages.iter().any(|language| language == "de")
        && word.chars().any(|value| matches!(value, 'ä' | 'ö' | 'ü' | 'ß' | 'Ä' | 'Ö' | 'Ü'))
    {
        return Some("de".to_string());
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
        return Some("fr".to_string());
    }

    None
}
