use crate::html_scan::blocks::{BlockMatches, Match, TagCloses};
use crate::html_scan::find_ci;

// Shared shape with `tabs.rs`.
pub(super) struct StartTag {
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
        if !self.remember {
            return scan_start_tag(html, pos);
        }
        let bytes = html.as_bytes();
        if bytes.get(pos) != Some(&b'<') {
            return None;
        }
        let tag_end = self.closes.close_from(bytes, pos + 1)?;
        let self_closing = tag_end > pos && bytes[tag_end - 1] == b'/';
        Some(StartTag { end: tag_end + 1, self_closing })
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
                    if html.as_bytes().get(at) == Some(&b'<') {
                        closes.close_from(html.as_bytes(), at + 1).map_or(at + 1, |end| end + 1)
                    } else {
                        at + 1
                    }
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

pub(super) fn scan_start_tag(html: &str, pos: usize) -> Option<StartTag> {
    let bytes = html.as_bytes();
    if bytes.get(pos) != Some(&b'<') {
        return None;
    }
    let mut i = pos + 1;
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
    Some(StartTag { end: tag_end + 1, self_closing })
}

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
