use super::*;

fn clamp_to_char_boundary(text: &str, byte_index: usize) -> usize {
    let mut index = byte_index.min(text.len());
    while !text.is_char_boundary(index) {
        index -= 1;
    }
    index
}

/// Converts byte offsets to character offsets in one forward pass.
///
/// Counting from the front of the line for every match of a scan is quadratic
/// in the line length — a single 1 MiB line took seconds. Regex matches arrive
/// in order, so a cursor only ever has to count the gap since its last answer.
pub(super) struct CharIndexCursor<'a> {
    ascii_only: bool,
    byte_index: usize,
    char_index: usize,
    text: &'a str,
}

impl<'a> CharIndexCursor<'a> {
    pub(super) fn new(text: &'a str) -> Self {
        Self { ascii_only: text.is_ascii(), byte_index: 0, char_index: 0, text }
    }

    /// Offsets are expected to arrive in non-decreasing order. One that goes
    /// backwards restarts the count from the front rather than answering with
    /// a stale character index, so an out-of-order caller stays correct and
    /// only loses the speedup.
    pub(super) fn char_index(&mut self, byte_index: usize) -> usize {
        let byte_index = clamp_to_char_boundary(self.text, byte_index);

        if self.ascii_only {
            return byte_index;
        }

        if byte_index < self.byte_index {
            self.byte_index = 0;
            self.char_index = 0;
        }

        self.char_index += self.text[self.byte_index..byte_index].chars().count();
        self.byte_index = byte_index;
        self.char_index
    }
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cursor_matches_a_full_recount_at_every_boundary() {
        for text in [
            "",
            "plain ascii text",
            "日本語のテキスト",
            "mixed 日本語 and ascii ではの",
            "combining a\u{0301}e\u{0301}o\u{0301} marks",
            "emoji \u{1F600} and \u{1F1EF}\u{1F1F5} flags",
        ] {
            let mut cursor = CharIndexCursor::new(text);
            for byte_index in 0..=text.len() {
                if !text.is_char_boundary(byte_index) {
                    continue;
                }
                let expected = text[..byte_index].chars().count();
                assert_eq!(cursor.char_index(byte_index), expected, "{text:?} at {byte_index}");
            }
        }
    }

    #[test]
    fn cursor_recovers_when_offsets_go_backwards() {
        let text = "日本語 mixed テキスト";
        let mut cursor = CharIndexCursor::new(text);

        let far = text.len();
        assert_eq!(cursor.char_index(far), text.chars().count());
        // Walking back has to recount rather than report a stale index.
        assert_eq!(cursor.char_index(0), 0);
        assert_eq!(cursor.char_index("日本語".len()), 3);
    }

    #[test]
    fn cursor_clamps_past_the_end_and_off_a_boundary() {
        let text = "日本語";
        let mut cursor = CharIndexCursor::new(text);

        // Past the end clamps to the end.
        assert_eq!(cursor.char_index(text.len() + 100), 3);

        // Inside a multi-byte character rounds down to its start.
        let mut cursor = CharIndexCursor::new(text);
        assert_eq!(cursor.char_index(4), 1);
        assert_eq!(cursor.char_index(5), 1);
        assert_eq!(cursor.char_index(6), 2);
    }

    #[test]
    fn cursor_takes_the_ascii_fast_path_without_changing_answers() {
        let text = "the quick brown fox";
        let mut cursor = CharIndexCursor::new(text);
        for byte_index in 0..=text.len() {
            assert_eq!(cursor.char_index(byte_index), byte_index);
        }
    }
}
