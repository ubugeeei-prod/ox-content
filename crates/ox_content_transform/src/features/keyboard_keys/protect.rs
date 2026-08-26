use crate::html_scan::find_ci;

pub(super) struct ProtectedSpan {
    pub start: usize,
    pub end: usize,
}

pub(super) fn next_protected(segment: &str, from: usize) -> Option<ProtectedSpan> {
    let bytes = segment.as_bytes();
    let mut cursor = from;
    while let Some(rel) = memchr::memchr(b'<', &bytes[cursor..]) {
        let start = cursor + rel;
        if let Some(end) = take_protected(segment, start) {
            return Some(ProtectedSpan { start, end });
        }
        cursor = start + 1;
    }
    None
}

fn take_protected(source: &str, pos: usize) -> Option<usize> {
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
