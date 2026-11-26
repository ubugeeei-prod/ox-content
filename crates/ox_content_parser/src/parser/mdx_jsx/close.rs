//! The closing-tag search: a walk over brace groups, code spans and tags.

use super::lex::Lex;
use super::scan::{is_attr_name_start, scan_attr_name, scan_member_name, skip_ws};

/// Any tag the search steps over, in the looser grammar it uses.
pub(super) struct TagSkip<'a> {
    pub name: Option<&'a str>,
    pub start: usize,
    pub closing: bool,
    pub self_closing: bool,
    pub end: usize,
}

pub(super) enum Search {
    /// Start and end of the closing tag.
    Found(usize, usize),
    /// The input ended first.
    Missing,
    /// The search reached its `stop` offset first.
    Unfinished,
}

impl Search {
    pub(super) fn found(self) -> Option<(usize, usize)> {
        match self {
            Self::Found(start, end) => Some((start, end)),
            Self::Missing | Self::Unfinished => None,
        }
    }
}

/// Finds the tag that closes an element named `name` whose opening tag ends
/// at `from`. `lex` must scan `source`.
///
/// The search gives up once it has stepped to `stop` or beyond; pass
/// `usize::MAX` to search to the end.
pub(super) fn find_matching_close(
    lex: &mut Lex<'_>,
    source: &str,
    from: usize,
    name: Option<&str>,
    stop: usize,
) -> Search {
    let bytes = source.as_bytes();
    let mut cursor = from;
    let mut depth = 1u32;
    while cursor < bytes.len() {
        if cursor >= stop {
            return Search::Unfinished;
        }
        match bytes[cursor] {
            b'{' => cursor = lex.skip_braces(cursor).unwrap_or(cursor + 1),
            b'`' => cursor = lex.skip_backticks(cursor).unwrap_or(cursor + 1),
            b'<' => {
                let Some(tag) = scan_tag_skip(lex, source, cursor) else {
                    cursor += 1;
                    continue;
                };
                if tag.name == name {
                    if tag.closing {
                        depth = depth.saturating_sub(1);
                        if depth == 0 {
                            return Search::Found(tag.start, tag.end);
                        }
                    } else if !tag.self_closing {
                        depth = depth.saturating_add(1);
                    }
                }
                cursor = tag.end;
            }
            _ => cursor += 1,
        }
    }
    Search::Missing
}

pub(super) fn scan_tag_skip<'a>(
    lex: &mut Lex<'_>,
    source: &'a str,
    start: usize,
) -> Option<TagSkip<'a>> {
    let bytes = source.as_bytes();
    if bytes.get(start)? != &b'<' {
        return None;
    }
    let mut cursor = start + 1;
    let closing = bytes.get(cursor) == Some(&b'/');
    if closing {
        cursor += 1;
    }
    if bytes.get(cursor) == Some(&b'>') {
        return Some(TagSkip { name: None, start, closing, self_closing: false, end: cursor + 1 });
    }
    let name_end = scan_member_name(bytes, cursor)?;
    let name = &source[cursor..name_end];
    cursor = name_end;
    let self_closing = skip_tag_rest(lex, &mut cursor)?;
    if closing && self_closing {
        return None;
    }
    Some(TagSkip { name: Some(name), start, closing, self_closing, end: cursor })
}

fn skip_tag_rest(lex: &mut Lex<'_>, cursor: &mut usize) -> Option<bool> {
    let bytes = lex.bytes();
    loop {
        skip_ws(bytes, cursor);
        match bytes.get(*cursor)? {
            b'/' if bytes.get(*cursor + 1) == Some(&b'>') => {
                *cursor += 2;
                return Some(true);
            }
            b'>' => {
                *cursor += 1;
                return Some(false);
            }
            b'{' => *cursor = lex.skip_braces(*cursor)?,
            b'"' | b'\'' | b'`' => *cursor = lex.skip_quoted(*cursor)?,
            byte if is_attr_name_start(*byte) => {
                *cursor = scan_attr_name(bytes, *cursor)?;
                skip_ws(bytes, cursor);
                if bytes.get(*cursor) == Some(&b'=') {
                    *cursor += 1;
                    skip_ws(bytes, cursor);
                    match bytes.get(*cursor)? {
                        b'"' | b'\'' => *cursor = lex.skip_quoted(*cursor)?,
                        b'{' => *cursor = lex.skip_braces(*cursor)?,
                        _ => skip_unquoted_value(bytes, cursor),
                    }
                }
            }
            _ => return None,
        }
    }
}

fn skip_unquoted_value(bytes: &[u8], cursor: &mut usize) {
    while *cursor < bytes.len()
        && !matches!(
            bytes[*cursor],
            b' ' | b'\t' | b'\n' | b'\r' | b'"' | b'\'' | b'=' | b'<' | b'>' | b'`' | b'/'
        )
    {
        *cursor += 1;
    }
}
