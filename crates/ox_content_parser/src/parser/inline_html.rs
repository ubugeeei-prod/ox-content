//! Inline raw HTML (CommonMark "Raw HTML").
//!
//! Only text matching the spec's tag grammar passes through as raw HTML;
//! anything else stays literal text. Open/closing tags may span line
//! endings, attribute names are restricted to the spec's character set,
//! attributes must be separated by whitespace, and comments, processing
//! instructions, declarations, and CDATA sections use their own
//! terminators.

use ox_content_ast::{Html, Span};

use super::Parser;
#[allow(unused_imports)]
use crate::profile_span_detail;

/// The bodies that end at a fixed terminator rather than at the first `>`:
/// opener, terminator, and the key the terminator's last position is kept
/// under in the parser's closer memo (`has_closer_from`). The keys are UTF-8
/// continuation bytes, which no markup rule asks about, and stay clear of
/// the ones `math/closer.rs` uses.
const TERMINATED: [(&str, &str, u8); 3] =
    [("<!--", "-->", 0x90), ("<?", "?>", 0x91), ("<![CDATA[", "]]>", 0x92)];

/// Content this short is searched directly. Same threshold as
/// `has_closer_from`.
const SHORT: usize = 256;

impl<'a> Parser<'a> {
    /// Raw HTML starting at the `<` at `pos`, and the index just past it.
    ///
    /// A comment, processing instruction or CDATA section with nothing to
    /// end it stays literal, and finding that out means searching to the end
    /// of the content. A run of them paid that search each — O(n²): 256 KiB
    /// of `<!-- ` took 3.8 s and of `<? ` 6.4 s. Where each terminator last
    /// occurs settles it for every opener in the content at once.
    pub(super) fn parse_inline_html(
        &self,
        content: &'a str,
        pos: usize,
        offset: usize,
    ) -> Option<(Html<'a>, usize)> {
        Self::scan_inline_html(content, pos, offset, |close, key, from| {
            if content.len() <= SHORT {
                return true;
            }
            let memo_key = (content.as_ptr() as usize, content.len(), key);
            let cached = self.last_closer.borrow().get(&memo_key).copied();
            let last = cached.unwrap_or_else(|| {
                let last = memchr::memmem::rfind(content.as_bytes(), close.as_bytes());
                self.last_closer.borrow_mut().insert(memo_key, last);
                last
            });
            last.is_some_and(|last| last >= from)
        })
    }

    /// [`Self::parse_inline_html`] for a single line, which is short enough
    /// to search directly.
    pub(super) fn parse_inline_html_line(content: &'a str) -> Option<(Html<'a>, usize)> {
        Self::scan_inline_html(content, 0, 0, |_, _, _| true)
    }

    /// `may_close(terminator, key, from)` reports whether `terminator` can
    /// occur at or after `from`; a `false` saves the search.
    fn scan_inline_html(
        content: &'a str,
        pos: usize,
        offset: usize,
        may_close: impl Fn(&str, u8, usize) -> bool,
    ) -> Option<(Html<'a>, usize)> {
        profile_span_detail!("parser::inline_html");
        let rest = &content[pos..];
        let bytes = content.as_bytes();

        // Comments, with the 0.31.2 empty forms `<!-->` and `<!--->`.
        for empty in ["<!-->", "<!--->"] {
            if rest.starts_with(empty) {
                let end = pos + empty.len();
                return Some((Self::inline_html_node(content, pos, end, offset), end));
            }
        }
        // Comments, processing instructions, CDATA, then other declarations.
        for (open, close, key) in TERMINATED {
            if let Some(body) = rest.strip_prefix(open) {
                let body_start = pos + open.len();
                if !may_close(close, key, body_start) {
                    return None;
                }
                let end = body.find(close).map(|found| body_start + found + close.len())?;
                return Some((Self::inline_html_node(content, pos, end, offset), end));
            }
        }
        if rest.starts_with("<!") && rest.as_bytes().get(2).is_some_and(u8::is_ascii_alphabetic) {
            let end = rest.find('>').map(|found| pos + found + 1)?;
            return Some((Self::inline_html_node(content, pos, end, offset), end));
        }

        let closing = bytes.get(pos + 1) == Some(&b'/');
        let tag_start = if closing { pos + 2 } else { pos + 1 };
        if !bytes.get(tag_start).is_some_and(u8::is_ascii_alphabetic) {
            return None;
        }

        let tag_end = Self::inline_tag_name_end(bytes, tag_start);
        let mut cursor = tag_end;

        if closing {
            while matches!(bytes.get(cursor), Some(b' ' | b'\t' | b'\n')) {
                cursor += 1;
            }
            return (bytes.get(cursor) == Some(&b'>')).then(|| {
                let end = cursor + 1;
                (Self::inline_html_node(content, pos, end, offset), end)
            });
        }

        let end = scan_open_tag_rest(bytes, cursor)?;
        Some((Self::inline_html_node(content, pos, end, offset), end))
    }

    fn inline_tag_name_end(bytes: &[u8], start: usize) -> usize {
        let mut end = start + 1;
        while end < bytes.len() && (bytes[end].is_ascii_alphanumeric() || bytes[end] == b'-') {
            end += 1;
        }
        end
    }

    fn inline_html_node(content: &'a str, start: usize, end: usize, offset: usize) -> Html<'a> {
        Html {
            value: &content[start..end],
            span: Span::new((offset + start) as u32, (offset + end) as u32),
        }
    }
}

/// Scans the remainder of an open tag after its name: whitespace-separated
/// attributes, optional `/`, and the closing `>`. Returns the index just
/// past `>`.
fn scan_open_tag_rest(bytes: &[u8], mut cursor: usize) -> Option<usize> {
    loop {
        let ws_start = cursor;
        while matches!(bytes.get(cursor), Some(b' ' | b'\t' | b'\n')) {
            cursor += 1;
        }
        match bytes.get(cursor)? {
            b'/' if bytes.get(cursor + 1) == Some(&b'>') => return Some(cursor + 2),
            b'>' => return Some(cursor + 1),
            _ => {
                // Attributes require separating whitespace.
                if cursor == ws_start {
                    return None;
                }
                cursor = scan_attribute(bytes, cursor)?;
            }
        }
    }
}

/// Scans one attribute: a spec-charset name plus an optional
/// `= value` (unquoted, single-, or double-quoted).
fn scan_attribute(bytes: &[u8], mut cursor: usize) -> Option<usize> {
    let first = *bytes.get(cursor)?;
    if !(first.is_ascii_alphabetic() || matches!(first, b'_' | b':')) {
        return None;
    }
    cursor += 1;
    while bytes.get(cursor).is_some_and(|byte| {
        byte.is_ascii_alphanumeric() || matches!(byte, b'_' | b'.' | b':' | b'-')
    }) {
        cursor += 1;
    }

    let mut look = cursor;
    while matches!(bytes.get(look), Some(b' ' | b'\t' | b'\n')) {
        look += 1;
    }
    if bytes.get(look) != Some(&b'=') {
        // Boolean attribute; the (skipped) whitespace still separates the
        // next attribute because the caller re-scans from `cursor`.
        return Some(cursor);
    }
    cursor = look + 1;
    while matches!(bytes.get(cursor), Some(b' ' | b'\t' | b'\n')) {
        cursor += 1;
    }

    if let quote @ (b'"' | b'\'') = *bytes.get(cursor)? {
        cursor += 1;
        while *bytes.get(cursor)? != quote {
            cursor += 1;
        }
        return Some(cursor + 1);
    }

    let value_start = cursor;
    while bytes.get(cursor).is_some_and(|byte| {
        !matches!(byte, b' ' | b'\t' | b'\n' | b'"' | b'\'' | b'=' | b'<' | b'>' | b'`')
    }) {
        cursor += 1;
    }
    (cursor > value_start).then_some(cursor)
}
