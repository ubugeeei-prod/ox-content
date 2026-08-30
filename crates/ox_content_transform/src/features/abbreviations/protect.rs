use crate::html_scan::find_ci;

pub(super) struct ProtectedSpan {
    pub start: usize,
    pub end: usize,
}

pub(super) fn next_protected(segment: &str, from: usize) -> Option<ProtectedSpan> {
    let bytes = segment.as_bytes();

    // A `[` with no `]` after it cannot open a link, and a `<` with no `>`
    // after it cannot open a tag — but the scans below only report that after
    // walking to the end of the segment, once per candidate. Bounding the
    // search at the last closer settles it for every candidate at once:
    // without this, 32 KiB of `*[` ` took 9.5 ms and grew x14 for every x4 of
    // input, on text with no abbreviations in it at all.
    let last_bracket = memchr::memrchr(b']', bytes);
    let last_angle = memchr::memrchr(b'>', bytes);

    let mut cursor = from;
    while cursor < bytes.len() {
        let bracket = last_bracket
            .filter(|last| cursor < *last)
            .and_then(|last| memchr::memchr(b'[', &bytes[cursor..last]).map(|rel| cursor + rel));
        let angle = last_angle
            .filter(|last| cursor < *last)
            .and_then(|last| memchr::memchr(b'<', &bytes[cursor..last]).map(|rel| cursor + rel));
        match (bracket, angle) {
            (None, None) => return None,
            (Some(start), Some(angle)) if angle < start => {
                if let Some(end) = take_html(segment, angle) {
                    return Some(ProtectedSpan { start: angle, end });
                }
                cursor = angle + 1;
            }
            (Some(start), _) => {
                if let Some(end) = take_markdown_link(bytes, start) {
                    return Some(ProtectedSpan { start, end });
                }
                cursor = start + 1;
            }
            (None, Some(angle)) => {
                if let Some(end) = take_html(segment, angle) {
                    return Some(ProtectedSpan { start: angle, end });
                }
                cursor = angle + 1;
            }
        }
    }
    None
}

fn take_markdown_link(bytes: &[u8], pos: usize) -> Option<usize> {
    let text_end = find_unescaped(bytes, pos + 1, b']')?;
    let after = text_end + 1;
    match bytes.get(after).copied() {
        Some(b'(') => find_unescaped(bytes, after + 1, b')').map(|end| end + 1),
        Some(b'[') => find_unescaped(bytes, after + 1, b']').map(|end| end + 1),
        _ => None,
    }
}

fn take_html(source: &str, pos: usize) -> Option<usize> {
    let after = pos + 1;
    if source.get(after..).is_some_and(|rest| rest.starts_with("!--")) {
        return source[after + 3..].find("-->").map(|rel| after + 3 + rel + 3);
    }
    let tag_end = scan_tag_end(source, pos)?;
    if source.as_bytes().get(after) == Some(&b'/') {
        return None;
    }
    let close = html_tag_name(&source[after..]).and_then(protected_close)?;
    if source[..tag_end].trim_end_matches('>').ends_with('/') {
        return None;
    }
    Some(skip_html_close(source, tag_end, close).unwrap_or(source.len()))
}

fn protected_close(name: &str) -> Option<&'static str> {
    match name.to_ascii_lowercase().as_str() {
        "a" => Some("</a"),
        "abbr" => Some("</abbr"),
        "code" => Some("</code"),
        "pre" => Some("</pre"),
        "script" => Some("</script"),
        "style" => Some("</style"),
        _ => None,
    }
}

fn html_tag_name(rest: &str) -> Option<&str> {
    let bytes = rest.as_bytes();
    let mut len = 0usize;
    while len < bytes.len() && bytes[len].is_ascii_alphabetic() {
        len += 1;
    }
    if len == 0 {
        return None;
    }
    let next = bytes.get(len).copied().unwrap_or(b'>');
    if next == b'>' || next == b'/' || next.is_ascii_whitespace() {
        Some(&rest[..len])
    } else {
        None
    }
}

fn skip_html_close(source: &str, from: usize, close: &str) -> Option<usize> {
    let start = find_ci(source, from, close)?;
    source[start + close.len()..].find('>').map(|rel| start + close.len() + rel + 1)
}

fn scan_tag_end(source: &str, start: usize) -> Option<usize> {
    let bytes = source.as_bytes();
    let mut cursor = start + 1;
    let mut quote = 0u8;
    while cursor < bytes.len() {
        let byte = bytes[cursor];
        if quote != 0 {
            if byte == quote {
                quote = 0;
            }
        } else if byte == b'"' || byte == b'\'' {
            quote = byte;
        } else if byte == b'>' {
            return Some(cursor + 1);
        }
        cursor += 1;
    }
    None
}

fn find_unescaped(bytes: &[u8], from: usize, needle: u8) -> Option<usize> {
    let mut cursor = from;
    while cursor < bytes.len() {
        let relative = memchr::memchr(needle, &bytes[cursor..])?;
        let pos = cursor + relative;
        if pos == 0 || bytes[pos - 1] != b'\\' {
            return Some(pos);
        }
        cursor = pos + 1;
    }
    None
}
