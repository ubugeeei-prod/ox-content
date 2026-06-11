use super::utils::*;
use super::*;

pub(super) fn segment_cjk_run(
    run: &str,
    start_offset: usize,
    language: &str,
    dictionary: &DictionaryBundle,
) -> Vec<Token> {
    let Some(words) = dictionary.cjk_segment_words.get(language) else {
        return vec![Token {
            end: start_offset + count_code_points(run),
            language: language.to_string(),
            start: start_offset,
            text: run.to_string(),
        }];
    };

    let char_boundaries = collect_char_boundaries(run);
    let total_chars = char_boundaries.len().saturating_sub(1);
    let mut tokens = Vec::new();
    let mut char_index = 0;

    while char_index < total_chars {
        let start_byte = char_boundaries[char_index];
        let best_match = words.iter().find(|word| {
            let end_char = char_index + word.char_len;
            if end_char > total_chars {
                return false;
            }

            let end_byte = char_boundaries[end_char];
            run[start_byte..end_byte] == word.text
        });

        if let Some(word) = best_match {
            tokens.push(Token {
                end: start_offset + char_index + word.char_len,
                language: language.to_string(),
                start: start_offset + char_index,
                text: word.text.clone(),
            });
            char_index += word.char_len;
            continue;
        }

        let end_char = char_index + 1;
        let end_byte = char_boundaries[end_char];
        tokens.push(Token {
            end: start_offset + end_char,
            language: language.to_string(),
            start: start_offset + char_index,
            text: run[start_byte..end_byte].to_string(),
        });
        char_index = end_char;
    }

    if tokens.iter().all(|token| count_code_points(&token.text) == 1) {
        return vec![Token {
            end: start_offset + count_code_points(run),
            language: language.to_string(),
            start: start_offset,
            text: run.to_string(),
        }];
    }

    tokens
}
