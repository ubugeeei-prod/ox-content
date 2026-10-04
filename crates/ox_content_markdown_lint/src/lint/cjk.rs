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
            language: CompactString::from(language),
            start: start_offset,
            text: CompactString::from(run),
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
                language: CompactString::from(language),
                start: start_offset + char_index,
                text: CompactString::from(word.text.as_str()),
            });
            char_index += word.char_len;
            continue;
        }

        let end_char = char_index + 1;
        let end_byte = char_boundaries[end_char];
        tokens.push(Token {
            end: start_offset + end_char,
            language: CompactString::from(language),
            start: start_offset + char_index,
            text: CompactString::from(&run[start_byte..end_byte]),
        });
        char_index = end_char;
    }

    if tokens.iter().all(|token| count_code_points(&token.text) == 1) {
        return vec![Token {
            end: start_offset + count_code_points(run),
            language: CompactString::from(language),
            start: start_offset,
            text: CompactString::from(run),
        }];
    }

    tokens
}
