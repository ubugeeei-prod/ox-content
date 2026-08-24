//! Brace, string, and comment skip for JSX tags and MDX expressions.
//!
//! This is not a JavaScript parser. Unclosed input returns `None` so the
//! caller can refuse to emit a half-parsed node.

/// Inner source and byte offset after the matching `}`.
pub(super) fn scan_balanced_braces(source: &str, start: usize) -> Option<(&str, usize)> {
    let end = skip_braces(source.as_bytes(), start)?;
    Some((&source[start + 1..end - 1], end))
}

pub(super) fn skip_braces(bytes: &[u8], start: usize) -> Option<usize> {
    if bytes.get(start) != Some(&b'{') {
        return None;
    }
    let mut cursor = start;
    let mut depth = 0u32;
    while cursor < bytes.len() {
        match bytes[cursor] {
            b'"' | b'\'' | b'`' => cursor = skip_quoted(bytes, cursor)?,
            b'/' if bytes.get(cursor + 1) == Some(&b'/') => {
                cursor = skip_line_comment(bytes, cursor);
            }
            b'/' if bytes.get(cursor + 1) == Some(&b'*') => {
                cursor = skip_block_comment(bytes, cursor)?;
            }
            b'{' => {
                depth = depth.saturating_add(1);
                cursor += 1;
            }
            b'}' => {
                depth = depth.saturating_sub(1);
                cursor += 1;
                if depth == 0 {
                    return Some(cursor);
                }
            }
            _ => cursor += 1,
        }
    }
    None
}

pub(super) fn skip_quoted(bytes: &[u8], start: usize) -> Option<usize> {
    let quote = *bytes.get(start)?;
    let mut cursor = start + 1;
    while cursor < bytes.len() {
        if bytes[cursor] == b'\\' && cursor + 1 < bytes.len() {
            cursor += 2;
            continue;
        }
        if bytes[cursor] == quote {
            return Some(cursor + 1);
        }
        cursor += 1;
    }
    None
}

pub(super) fn skip_backticks(bytes: &[u8], start: usize) -> Option<usize> {
    let mut open = 0usize;
    while start + open < bytes.len() && bytes[start + open] == b'`' {
        open += 1;
    }
    if open == 0 {
        return None;
    }
    let mut cursor = start + open;
    while cursor + open <= bytes.len() {
        if bytes[cursor..cursor + open].iter().all(|byte| *byte == b'`')
            && bytes.get(cursor + open) != Some(&b'`')
        {
            return Some(cursor + open);
        }
        cursor += 1;
    }
    None
}

fn skip_line_comment(bytes: &[u8], start: usize) -> usize {
    let mut cursor = start + 2;
    while cursor < bytes.len() && bytes[cursor] != b'\n' {
        cursor += 1;
    }
    cursor
}

fn skip_block_comment(bytes: &[u8], start: usize) -> Option<usize> {
    let mut cursor = start + 2;
    while cursor + 1 < bytes.len() {
        if bytes[cursor] == b'*' && bytes[cursor + 1] == b'/' {
            return Some(cursor + 2);
        }
        cursor += 1;
    }
    None
}

#[cfg(test)]
mod tests;
