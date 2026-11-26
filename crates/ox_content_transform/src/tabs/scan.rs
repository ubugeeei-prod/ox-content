use crate::html_scan::blocks::{BlockMatches, Match, TagCloses};
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

/// The start tag and closer searches of one walk over `html`, settled once
/// for the whole of it (see `html_scan/blocks.rs`). With `remember` off,
/// every search runs afresh, as the walk used to.
pub(super) struct Blocks {
    remember: bool,
    closes: TagCloses,
    matches: BlockMatches,
}

impl Blocks {
    pub(super) fn new(remember: bool) -> Self {
        Self { remember, closes: TagCloses::default(), matches: BlockMatches::default() }
    }

    pub(super) fn start_tag(&mut self, html: &str, pos: usize) -> Option<StartTag> {
        if self.remember {
            scan_start_tag_with(html, pos, |from| self.closes.close_from(html.as_bytes(), from))
        } else {
            scan_start_tag(html, pos)
        }
    }

    /// The `close` matching the `open` tag at `open_at`, whose start tag
    /// ends at `from`.
    pub(super) fn matching_close(
        &mut self,
        html: &str,
        (open_at, from): (usize, usize),
        open: &str,
        close: &str,
    ) -> Option<usize> {
        if self.remember {
            let closes = &mut self.closes;
            let found = self.matches.close_of(
                open_at,
                close.len(),
                |at| find_tag(html, at, open),
                |at| find_ci(html, at, close),
                |at| {
                    scan_start_tag_with(html, at, |from| closes.close_from(html.as_bytes(), from))
                        .map_or(at + 1, |tag| tag.end)
                },
            );
            match found {
                Match::Closes(close) => return Some(close),
                Match::Unclosed => return None,
                Match::Unknown => {}
            }
        }
        find_matching_close(html, from, open, close)
    }
}

/// Scan the start tag beginning at `pos` (which must point at `<`), respecting
/// quoted attribute values so a `>` inside a value doesn't end the tag early.
pub(super) fn scan_start_tag(html: &str, pos: usize) -> Option<StartTag> {
    let bytes = html.as_bytes();
    scan_start_tag_with(html, pos, |from| {
        let mut quote: Option<u8> = None;
        for (at, &byte) in bytes.iter().enumerate().skip(from) {
            match quote {
                Some(q) if byte == q => quote = None,
                Some(_) => {}
                None if byte == b'"' || byte == b'\'' => quote = Some(byte),
                None if byte == b'>' => return Some(at),
                None => {}
            }
        }
        None
    })
}

/// [`scan_start_tag`] with the search for the `>` that ends the tag, from
/// the end of its name, left to `close_from`.
fn scan_start_tag_with(
    html: &str,
    pos: usize,
    close_from: impl FnOnce(usize) -> Option<usize>,
) -> Option<StartTag> {
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
    let tag_end = close_from(name_end)?;
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
