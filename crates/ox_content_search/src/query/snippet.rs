const SNIPPET_CONTEXT_DIVISOR: usize = 3;
const SNIPPET_ELLIPSIS: &str = "...";

pub(super) fn generate_snippet(body: &str, matches: &[String], max_len: usize) -> String {
    if body.is_empty() || max_len == 0 {
        return String::new();
    }

    let body_lower = body.to_lowercase();

    let mut first_match_pos: Option<usize> = None;
    for term in matches {
        if let Some(pos) = body_lower.find(term) {
            first_match_pos = Some(first_match_pos.map_or(pos, |current| current.min(pos)));
        }
    }

    let start_pos = previous_char_boundary(body, first_match_pos.unwrap_or(0));
    let start_char = body[..start_pos.min(body.len())].chars().count();

    let context_before = max_len / SNIPPET_CONTEXT_DIVISOR;
    let mut snippet_start = byte_index_for_char(body, start_char.saturating_sub(context_before));
    snippet_start = previous_word_start_byte(body, snippet_start);

    let snippet_end = byte_index_after_chars(body, snippet_start, max_len);
    let needs_prefix = snippet_start > 0;
    let needs_suffix = snippet_end < body.len();
    let mut snippet = String::with_capacity(
        snippet_end.saturating_sub(snippet_start)
            + usize::from(needs_prefix) * SNIPPET_ELLIPSIS.len()
            + usize::from(needs_suffix) * SNIPPET_ELLIPSIS.len(),
    );

    if needs_prefix {
        snippet.push_str(SNIPPET_ELLIPSIS);
    }
    snippet.push_str(&body[snippet_start..snippet_end]);
    if needs_suffix {
        snippet.push_str(SNIPPET_ELLIPSIS);
    }

    snippet
}

fn byte_index_for_char(body: &str, target_char: usize) -> usize {
    if target_char == 0 {
        return 0;
    }

    body.char_indices().nth(target_char).map_or(body.len(), |(byte, _)| byte)
}

fn previous_char_boundary(body: &str, byte_index: usize) -> usize {
    let mut byte_index = byte_index.min(body.len());
    while !body.is_char_boundary(byte_index) {
        byte_index -= 1;
    }
    byte_index
}

fn previous_word_start_byte(body: &str, start_byte: usize) -> usize {
    let mut start_byte = previous_char_boundary(body, start_byte);

    while start_byte < body.len() {
        let Some(current_char) = body[start_byte..].chars().next() else {
            return start_byte;
        };

        if current_char.is_whitespace() {
            return start_byte + current_char.len_utf8();
        }

        if start_byte == 0 {
            return 0;
        }

        let Some((prev_byte, _)) = body[..start_byte].char_indices().next_back() else {
            return 0;
        };
        start_byte = prev_byte;
    }

    body.len()
}

fn byte_index_after_chars(body: &str, start_byte: usize, char_count: usize) -> usize {
    if char_count == 0 {
        return start_byte;
    }

    body[start_byte..]
        .char_indices()
        .nth(char_count)
        .map_or(body.len(), |(byte, _)| start_byte + byte)
}
