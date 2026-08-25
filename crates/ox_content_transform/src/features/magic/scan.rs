use super::{OPEN, ResolvedMagicLinks, emit};
use crate::features::segments::{is_closing_fence, is_indented_code_line, parse_opening_fence};
use crate::html_scan::find_ci;

pub(super) fn transform(source: &str, options: &ResolvedMagicLinks) -> Option<String> {
    let bytes = source.as_bytes();
    let mut out = String::with_capacity(source.len());
    let mut changed = false;
    let mut cursor = 0usize;
    let mut fence: Option<(u8, usize)> = None;
    let mut html_close: Option<&'static str> = None;

    while cursor < bytes.len() {
        if let Some(close) = html_close {
            if let Some(end) = skip_html_close(source, cursor, close) {
                out.push_str(&source[cursor..end]);
                cursor = end;
                html_close = None;
            } else {
                out.push_str(&source[cursor..]);
                return changed.then_some(out);
            }
            continue;
        }

        if let Some((fence_char, fence_len)) = fence {
            let line_end = line_end_index(source, cursor);
            let line = &source[cursor..line_end];
            out.push_str(line);
            if is_closing_fence(line.trim_end_matches('\n'), fence_char, fence_len) {
                fence = None;
            }
            cursor = line_end;
            continue;
        }

        if is_line_start(bytes, cursor) {
            let line_end = line_end_index(source, cursor);
            let line = source[cursor..line_end].trim_end_matches('\n');
            if let Some(open) = parse_opening_fence(line) {
                fence = Some((open.fence_char, open.fence_len));
                out.push_str(&source[cursor..line_end]);
                cursor = line_end;
                continue;
            }
            if is_indented_code_line(line) {
                out.push_str(&source[cursor..line_end]);
                cursor = line_end;
                continue;
            }
        }

        let Some(rel) = next_marker(&bytes[cursor..]) else {
            out.push_str(&source[cursor..]);
            break;
        };
        let pos = cursor + rel;
        out.push_str(&source[cursor..pos]);
        if pos > 0 && bytes[pos - 1] == b'\\' {
            out.push(source[pos..].chars().next().unwrap_or('\\'));
            cursor = pos + 1;
            continue;
        }

        match bytes[pos] {
            b'{' => {
                if try_rewrite_magic(source, pos, options, &mut out) {
                    changed = true;
                    cursor = matching_close(source, pos).map_or(pos + 1, |close| close + 1);
                } else {
                    out.push('{');
                    cursor = pos + 1;
                }
            }
            b'`' => cursor = copy_inline_code(source, pos, &mut out),
            b'[' => cursor = copy_markdown_link_or_bracket(source, pos, &mut out),
            b'<' => {
                if let Some((end, close)) = take_html(source, pos) {
                    out.push_str(&source[pos..end]);
                    cursor = end;
                    html_close = close;
                } else {
                    out.push('<');
                    cursor = pos + 1;
                }
            }
            _ => cursor = pos + 1,
        }
    }

    changed.then_some(out)
}

fn try_rewrite_magic(
    source: &str,
    pos: usize,
    options: &ResolvedMagicLinks,
    out: &mut String,
) -> bool {
    if !source[pos..].starts_with(OPEN) {
        return false;
    }
    let Some(close) = matching_close(source, pos) else {
        return false;
    };
    let Some(html) = emit::render_body(&source[pos + OPEN.len()..close], options) else {
        return false;
    };
    out.push_str(&html);
    true
}

fn matching_close(source: &str, pos: usize) -> Option<usize> {
    let inner_start = pos + OPEN.len();
    let relative = source[inner_start..].find('}')?;
    let close = inner_start + relative;
    if source[inner_start..close].bytes().any(|byte| byte == b'\n' || byte == b'\r') {
        return None;
    }
    Some(close)
}

fn copy_inline_code(source: &str, pos: usize, out: &mut String) -> usize {
    let bytes = source.as_bytes();
    let ticks = count_repeated(bytes, pos, b'`');
    let close = find_closing_ticks(bytes, pos + ticks, ticks).unwrap_or(bytes.len());
    let end = if close == bytes.len() { bytes.len() } else { close + ticks };
    out.push_str(&source[pos..end]);
    end
}

fn copy_markdown_link_or_bracket(source: &str, pos: usize, out: &mut String) -> usize {
    let bytes = source.as_bytes();
    let Some(text_end) = find_unescaped(bytes, pos + 1, b']') else {
        out.push('[');
        return pos + 1;
    };
    let after = text_end + 1;
    let dest_end = match bytes.get(after).copied() {
        Some(b'(') => find_unescaped(bytes, after + 1, b')').map(|end| end + 1),
        Some(b'[') => find_unescaped(bytes, after + 1, b']').map(|end| end + 1),
        _ => None,
    };
    if let Some(end) = dest_end {
        out.push_str(&source[pos..end]);
        end
    } else {
        out.push('[');
        pos + 1
    }
}

fn take_html(source: &str, pos: usize) -> Option<(usize, Option<&'static str>)> {
    let after = pos + 1;
    if source[after..].starts_with("!--") {
        let close = source[after + 3..].find("-->").map(|rel| after + 3 + rel + 3)?;
        return Some((close, None));
    }
    let end = scan_tag_end(source, pos)?;
    if source.as_bytes().get(after) == Some(&b'/') {
        return Some((end, None));
    }
    let close = html_tag_name(&source[after..]).and_then(protected_close);
    let self_closing = source[..end].trim_end_matches('>').ends_with('/');
    Some((end, close.filter(|_| !self_closing)))
}

fn skip_html_close(source: &str, from: usize, close: &str) -> Option<usize> {
    let start = find_ci(source, from, close)?;
    source[start + close.len()..].find('>').map(|rel| start + close.len() + rel + 1)
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

fn protected_close(name: &str) -> Option<&'static str> {
    match name.to_ascii_lowercase().as_str() {
        "a" => Some("</a"),
        "code" => Some("</code"),
        "pre" => Some("</pre"),
        "script" => Some("</script"),
        "style" => Some("</style"),
        "textarea" => Some("</textarea"),
        _ => None,
    }
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

fn find_closing_ticks(bytes: &[u8], from: usize, count: usize) -> Option<usize> {
    let mut cursor = from;
    while cursor < bytes.len() {
        let relative = memchr::memchr(b'`', &bytes[cursor..])?;
        let start = cursor + relative;
        if count_repeated(bytes, start, b'`') >= count {
            return Some(start);
        }
        cursor = start + 1;
    }
    None
}

fn count_repeated(bytes: &[u8], start: usize, byte: u8) -> usize {
    bytes[start..].iter().take_while(|value| **value == byte).count()
}

fn line_end_index(source: &str, from: usize) -> usize {
    source[from..].find('\n').map_or(source.len(), |rel| from + rel + 1)
}

fn is_line_start(bytes: &[u8], cursor: usize) -> bool {
    cursor == 0 || bytes[cursor - 1] == b'\n'
}

fn next_marker(bytes: &[u8]) -> Option<usize> {
    match (memchr::memchr2(b'{', b'`', bytes), memchr::memchr2(b'[', b'<', bytes)) {
        (Some(left), Some(right)) => Some(left.min(right)),
        (Some(index), None) | (None, Some(index)) => Some(index),
        (None, None) => None,
    }
}
