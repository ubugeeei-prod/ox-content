//! Inline math as it was scanned before closer searches were remembered:
//! every opening `$` walks forward on its own until something closes it or
//! the content ends.
//!
//! The functions under "the old scan" are kept as they were written.
//! `inline_math` drives them the way the inline parser does for text made
//! only of `$`, backticks, backslashes, emphasis markers and plain bytes, so
//! `math_scaling.rs` can hold the remembered searches to them.

use memchr::memchr2;

/// `(start, end, value)` of every inline math node in `content`.
pub fn inline_math(content: &str) -> Vec<(usize, usize, String)> {
    let bytes = content.as_bytes();
    let mut found = Vec::new();
    let mut pos = 0;
    while pos < bytes.len() {
        match bytes[pos] {
            // An escaped punctuation byte is literal, `$` and backtick
            // included.
            b'\\' if bytes.get(pos + 1).is_some_and(u8::is_ascii_punctuation) => pos += 2,
            // A code span when a run of the same length closes it, a
            // literal run otherwise.
            b'`' => {
                pos = closed_code_span_end(bytes, pos)
                    .unwrap_or_else(|| pos + marker_run_len(bytes, pos, b'`'));
            }
            b'$' => parse_inline_math(content, &mut found, &mut pos),
            _ => pos += 1,
        }
    }
    found
}

fn marker_run_len(bytes: &[u8], start: usize, marker: u8) -> usize {
    let mut count = 1;
    while start + count < bytes.len() && bytes[start + count] == marker {
        count += 1;
    }
    count
}

fn closed_code_span_end(bytes: &[u8], start: usize) -> Option<usize> {
    let open_len = marker_run_len(bytes, start, b'`');
    let mut cursor = start + open_len;
    while cursor < bytes.len() {
        let relative = memchr::memchr(b'`', &bytes[cursor..])?;
        cursor += relative;
        let close_len = marker_run_len(bytes, cursor, b'`');
        if close_len == open_len {
            return Some(cursor + close_len);
        }
        cursor += close_len;
    }
    None
}

// The old scan.

fn parse_inline_math(content: &str, found: &mut Vec<(usize, usize, String)>, pos: &mut usize) {
    let bytes = content.as_bytes();
    let open_len = if bytes.get(*pos + 1) == Some(&b'$') { 2 } else { 1 };
    if !can_open_inline(bytes, *pos, open_len)
        && !can_open_digit_prefixed_inline_math(bytes, *pos, open_len)
    {
        *pos += 1;
        return;
    }

    let inner_start = *pos + open_len;
    let mut cursor = inner_start;
    while let Some(relative) = memchr2(b'$', b'`', &bytes[cursor..]) {
        let candidate = cursor + relative;
        if bytes[candidate] == b'`' {
            if let Some(end) = closed_code_span_end(bytes, candidate) {
                cursor = end;
            } else {
                cursor = candidate + marker_run_len(bytes, candidate, b'`');
            }
            continue;
        }
        if is_escaped_marker(bytes, candidate) {
            cursor = candidate + 1;
            continue;
        }
        if marker_len_at(bytes, candidate) >= open_len
            && can_close_inline(bytes, candidate, open_len)
        {
            found.push((*pos, candidate + open_len, content[inner_start..candidate].to_string()));
            *pos = candidate + open_len;
            return;
        }
        cursor = candidate + 1;
    }

    *pos += open_len;
}

fn can_open_inline(bytes: &[u8], index: usize, open_len: usize) -> bool {
    let next = bytes.get(index + open_len).copied();
    let prev = index.checked_sub(1).and_then(|prev| bytes.get(prev).copied());
    !matches!(next, None | Some(b' ' | b'\t' | b'\n' | b'0'..=b'9'))
        && !matches!(prev, Some(b'0'..=b'9'))
}

fn can_open_digit_prefixed_inline_math(bytes: &[u8], index: usize, open_len: usize) -> bool {
    let next = bytes.get(index + open_len).copied();
    let prev = index.checked_sub(1).and_then(|prev| bytes.get(prev).copied());
    matches!(next, Some(b'0'..=b'9'))
        && !matches!(prev, Some(b'0'..=b'9'))
        && has_closing_inline_math(bytes, index, open_len)
}

fn has_closing_inline_math(bytes: &[u8], index: usize, open_len: usize) -> bool {
    let mut cursor = index + open_len;
    while let Some(relative) = memchr2(b'$', b'`', &bytes[cursor..]) {
        let candidate = cursor + relative;
        if bytes[candidate] == b'`' {
            if let Some(end) = closed_code_span_end(bytes, candidate) {
                cursor = end;
            } else {
                cursor = candidate + marker_run_len(bytes, candidate, b'`');
            }
            continue;
        }
        if !is_escaped_marker(bytes, candidate)
            && marker_len_at(bytes, candidate) >= open_len
            && can_close_inline(bytes, candidate, open_len)
        {
            return bytes[index + open_len..candidate]
                .iter()
                .any(|byte| matches!(byte, b'*' | b'_'));
        }
        cursor = candidate + 1;
    }
    false
}

fn can_close_inline(bytes: &[u8], index: usize, close_len: usize) -> bool {
    let prev = index.checked_sub(1).and_then(|prev| bytes.get(prev).copied());
    let next = bytes.get(index + close_len).copied();
    !matches!(prev, None | Some(b' ' | b'\t' | b'\n')) && !matches!(next, Some(b'0'..=b'9'))
}

fn marker_len_at(bytes: &[u8], start: usize) -> usize {
    let mut len = 0;
    while bytes.get(start + len) == Some(&b'$') {
        len += 1;
    }
    len
}

fn is_escaped_marker(bytes: &[u8], pos: usize) -> bool {
    let mut count = 0usize;
    let mut cursor = pos;
    while cursor > 0 && bytes[cursor - 1] == b'\\' {
        count += 1;
        cursor -= 1;
    }
    count % 2 == 1
}
