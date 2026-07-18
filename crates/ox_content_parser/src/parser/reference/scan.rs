//! Pure byte/line scanners for the reference-definition parser.
//!
//! These helpers carry no parser state; they inspect raw source bytes to
//! locate line boundaries, strip block quote markers, and recognize the
//! fence and paragraph-closing markers the collection pre-pass needs.

pub(super) fn skip_ws_one_newline(bytes: &[u8], mut i: usize) -> Option<usize> {
    let mut newlines = 0;
    while let Some(&byte) = bytes.get(i) {
        match byte {
            b' ' | b'\t' => i += 1,
            b'\n' => {
                newlines += 1;
                if newlines > 1 {
                    return None;
                }
                i += 1;
            }
            _ => break,
        }
    }
    Some(i)
}

/// Returns the offset just past the line's newline (or EOF) when only
/// spaces/tabs remain between `i` and the end of the line.
pub(super) fn line_end_if_blank_after(bytes: &[u8], mut i: usize) -> Option<usize> {
    while let Some(&byte) = bytes.get(i) {
        match byte {
            b' ' | b'\t' => i += 1,
            b'\n' => return Some(i + 1),
            _ => return None,
        }
    }
    Some(bytes.len())
}

pub(super) fn next_blank_line(bytes: &[u8], mut pos: usize) -> usize {
    while pos < bytes.len() {
        let line_end = memchr::memchr(b'\n', &bytes[pos..]).map_or(bytes.len(), |o| pos + o);
        if bytes[pos..line_end].iter().all(|byte| matches!(byte, b' ' | b'\t')) {
            return pos;
        }
        pos = line_end + 1;
    }
    bytes.len()
}

/// Strips leading `>` block quote markers (each with up to three spaces of
/// indent and one optional following space).
pub(super) fn strip_quote_markers(mut line: &str) -> &str {
    loop {
        let bytes = line.as_bytes();
        let mut i = 0;
        while i < bytes.len() && i < 3 && bytes[i] == b' ' {
            i += 1;
        }
        if bytes.get(i) != Some(&b'>') {
            return line;
        }
        i += 1;
        if bytes.get(i) == Some(&b' ') {
            i += 1;
        }
        line = &line[i..];
    }
}

/// Lines that close an open paragraph without themselves opening one:
/// ATX headings and setext/thematic marker runs. Everything else that is
/// non-blank keeps (or opens) paragraph-like context for the pre-pass.
pub(in crate::parser) fn closes_paragraph_context(trimmed: &str) -> bool {
    if trimmed.starts_with('#') {
        return true;
    }
    let bytes = trimmed.trim_end().as_bytes();
    !bytes.is_empty()
        && (bytes.iter().all(|&byte| byte == b'-')
            || bytes.iter().all(|&byte| byte == b'=')
            || bytes.iter().all(|&byte| byte == b'*' || byte == b' '))
}

pub(in crate::parser) fn fence_open(trimmed: &str) -> Option<(u8, usize)> {
    let bytes = trimmed.as_bytes();
    let first = *bytes.first()?;
    if first != b'`' && first != b'~' {
        return None;
    }
    let len = bytes.iter().take_while(|&&byte| byte == first).count();
    if len < 3 {
        return None;
    }
    // An opening backtick fence cannot contain backticks in its info string.
    if first == b'`' && trimmed[len..].contains('`') {
        return None;
    }
    Some((first, len))
}

pub(in crate::parser) fn is_fence_close(trimmed: &str, fence_byte: u8, fence_len: usize) -> bool {
    let bytes = trimmed.as_bytes();
    let len = bytes.iter().take_while(|&&byte| byte == fence_byte).count();
    len >= fence_len && trimmed[len..].trim().is_empty()
}
