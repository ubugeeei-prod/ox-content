use crate::html_scan::find_ci;

/// A scanned start tag.
pub(super) struct StartTag {
    /// Byte index just past the element name (start of the attribute region).
    pub(super) name_end: usize,
    /// Byte index of the attribute region end (before `/` of `/>` or before `>`).
    pub(super) inner_end: usize,
    /// Byte index just past the closing `>`.
    pub(super) end: usize,
    pub(super) self_closing: bool,
}

/// Scan the start tag beginning at `pos` (which must point at `<`), respecting
/// quoted attribute values so a `>` inside a value doesn't end the tag early.
pub(super) fn scan_start_tag(html: &str, pos: usize) -> Option<StartTag> {
    let bytes = html.as_bytes();
    if bytes.get(pos) != Some(&b'<') {
        return None;
    }
    let mut i = pos + 1;
    while i < bytes.len() && !bytes[i].is_ascii_whitespace() && bytes[i] != b'>' && bytes[i] != b'/'
    {
        i += 1;
    }
    let name_end = i;
    let mut quote: Option<u8> = None;
    let mut tag_end = None;
    while i < bytes.len() {
        let b = bytes[i];
        match quote {
            Some(q) => {
                if b == q {
                    quote = None;
                }
            }
            None => {
                if b == b'"' || b == b'\'' {
                    quote = Some(b);
                } else if b == b'>' {
                    tag_end = Some(i);
                    break;
                }
            }
        }
        i += 1;
    }
    let tag_end = tag_end?;
    let self_closing = tag_end > pos && bytes[tag_end - 1] == b'/';
    let inner_end = if self_closing { tag_end - 1 } else { tag_end };
    Some(StartTag { name_end, inner_end, end: tag_end + 1, self_closing })
}

/// Find the next start tag at or after `from` with a proper element boundary.
/// `open` is the `<name` literal (e.g. `"<tabs"`); the boundary check means
/// `<tab` never matches `<tabs`/`<table` and `<tabs` never matches `<tabset>`.
pub(super) fn find_tag(html: &str, from: usize, open: &str) -> Option<usize> {
    let bytes = html.as_bytes();
    let mut search = from;
    loop {
        let at = find_ci(html, search, open)?;
        let after = at + open.len();
        let boundary = bytes.get(after).copied();
        if matches!(boundary, Some(b) if b == b'>' || b == b'/' || b.is_ascii_whitespace()) {
            return Some(at);
        }
        search = after;
    }
}

/// Given the position just past a `<name ...>` start tag, find the byte offset
/// of its matching `close`, accounting for nested `open` tags.
pub(super) fn find_matching_close(
    html: &str,
    from: usize,
    open: &str,
    close: &str,
) -> Option<usize> {
    let mut depth = 1usize;
    let mut search = from;
    loop {
        let next_open = find_tag(html, search, open);
        let next_close = find_ci(html, search, close);
        match (next_open, next_close) {
            (Some(open_at), Some(close_at)) if open_at < close_at => {
                depth += 1;
                search = match scan_start_tag(html, open_at) {
                    Some(tag) => tag.end,
                    None => open_at + 1,
                };
            }
            (_, Some(close_at)) => {
                depth -= 1;
                if depth == 0 {
                    return Some(close_at);
                }
                search = close_at + close.len();
            }
            (_, None) => return None,
        }
    }
}

/// Read the value of `name` (case-insensitive) from a start tag's attribute
/// region. Supports quoted and unquoted values; missing value yields `""`.
pub(super) fn attribute_value(attrs: &str, name: &str) -> Option<String> {
    // The transform only needs one requested attribute from a start-tag slice.
    // A narrow byte scan is faster and simpler than constructing a generic
    // attribute map for every tab element.
    let bytes = attrs.as_bytes();
    let mut i = 0;
    while i < bytes.len() {
        while i < bytes.len() && bytes[i].is_ascii_whitespace() {
            i += 1;
        }
        if i >= bytes.len() || bytes[i] == b'/' {
            break;
        }
        let name_start = i;
        while i < bytes.len()
            && !bytes[i].is_ascii_whitespace()
            && bytes[i] != b'='
            && bytes[i] != b'/'
        {
            i += 1;
        }
        let attr_name = &attrs[name_start..i];
        while i < bytes.len() && bytes[i].is_ascii_whitespace() {
            i += 1;
        }
        let value = if i < bytes.len() && bytes[i] == b'=' {
            i += 1;
            while i < bytes.len() && bytes[i].is_ascii_whitespace() {
                i += 1;
            }
            if i >= bytes.len() {
                String::new()
            } else if bytes[i] == b'"' || bytes[i] == b'\'' {
                let q = bytes[i];
                i += 1;
                let vs = i;
                while i < bytes.len() && bytes[i] != q {
                    i += 1;
                }
                let v = attrs[vs..i].to_string();
                if i < bytes.len() {
                    i += 1;
                }
                v
            } else {
                let vs = i;
                while i < bytes.len() && !bytes[i].is_ascii_whitespace() && bytes[i] != b'/' {
                    i += 1;
                }
                attrs[vs..i].to_string()
            }
        } else {
            String::new()
        };
        if attr_name.eq_ignore_ascii_case(name) {
            return Some(value);
        }
    }
    None
}
