use super::*;

pub(super) fn byte_to_char_index(text: &str, byte_index: usize) -> usize {
    text[..byte_index.min(text.len())].chars().count()
}

pub(super) fn collect_char_boundaries(text: &str) -> Vec<usize> {
    let mut boundaries = text.char_indices().map(|(index, _)| index).collect::<Vec<_>>();
    boundaries.push(text.len());
    boundaries
}

pub(super) fn sort_and_dedupe_segment_words(words: &mut Vec<SegmentWord>) {
    words.sort_by(|left, right| {
        right.char_len.cmp(&left.char_len).then_with(|| left.text.cmp(&right.text))
    });
    words.dedup_by(|left, right| left.text == right.text);
}

pub(super) fn normalize_comparable_word(word: &str) -> String {
    normalize_latin_word(word).chars().filter(|value| !matches!(value, '\'' | '’' | '-')).collect()
}

pub(super) fn normalize_word_for_set(word: &str) -> String {
    if word.chars().any(is_cjk_char) {
        word.nfc().collect::<String>().trim().to_string()
    } else {
        normalize_latin_word(word)
    }
}

pub(super) fn normalize_latin_word(word: &str) -> String {
    word.nfc().flat_map(char::to_lowercase).collect()
}

pub(super) fn collapse_whitespace(text: &str) -> String {
    let mut collapsed = CompactString::default();
    let mut needs_space = false;

    for part in text.split_whitespace() {
        if needs_space {
            collapsed.push(' ');
        }
        collapsed.push_str(part);
        needs_space = true;
    }

    collapsed.into_string()
}

pub(super) fn count_code_points(text: &str) -> usize {
    text.chars().count()
}

pub(super) fn dedupe_strings(values: Vec<String>) -> Vec<String> {
    let mut seen = FxHashSet::default();
    let mut deduped = Vec::new();

    for value in values {
        if seen.insert(value.clone()) {
            deduped.push(value);
        }
    }

    deduped
}

pub(super) fn is_supported_language(language: &str) -> bool {
    SUPPORTED_MARKDOWN_LINT_LANGUAGES.contains(&language)
}

pub(super) fn is_repeated_punctuation_char(value: char) -> bool {
    matches!(value, '!' | '?' | '！' | '？' | '。' | '、' | '，')
}

pub(super) fn is_uppercase_token(value: &str) -> bool {
    let mut chars = value.chars();
    let Some(first_char) = chars.next() else {
        return false;
    };

    first_char.is_uppercase()
        && chars.all(|character| character.is_uppercase() || character.is_numeric())
}

pub(super) fn is_hiragana(value: char) -> bool {
    matches!(value, '\u{3040}'..='\u{309F}')
}

pub(super) fn is_cjk_char(value: char) -> bool {
    matches!(
        value,
        '\u{3040}'..='\u{309F}' | '\u{30A0}'..='\u{30FF}' | '\u{4E00}'..='\u{9FFF}'
    )
}

pub(super) fn levenshtein(left: &str, right: &str) -> usize {
    if left == right {
        return 0;
    }

    let left_chars = left.chars().collect::<Vec<_>>();
    let right_chars = right.chars().collect::<Vec<_>>();

    if left_chars.is_empty() {
        return right_chars.len();
    }
    if right_chars.is_empty() {
        return left_chars.len();
    }

    let mut previous_row = (0..=right_chars.len()).collect::<Vec<_>>();
    let mut current_row = vec![0; right_chars.len() + 1];

    for (left_index, left_value) in left_chars.iter().enumerate() {
        current_row[0] = left_index + 1;

        for (right_index, right_value) in right_chars.iter().enumerate() {
            let substitution_cost = usize::from(left_value != right_value);
            let insertion = current_row[right_index] + 1;
            let deletion = previous_row[right_index + 1] + 1;
            let substitution = previous_row[right_index] + substitution_cost;
            current_row[right_index + 1] = insertion.min(deletion).min(substitution);
        }

        previous_row.clone_from(&current_row);
    }

    previous_row[right_chars.len()]
}
