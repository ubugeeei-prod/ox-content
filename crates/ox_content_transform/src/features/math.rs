//! Opt-in `$…$` inline and `$$…$$` block math.

#[cfg(test)]
mod tests;

use std::borrow::Cow;

use super::escape_html_attr;
use super::segments::transform_markdown_prose_segments;
use crate::MathOptions;

pub(super) fn resolve(options: Option<&MathOptions>) -> bool {
    options.is_some_and(|options| options.enabled != Some(false))
}

pub(super) fn apply(current: &mut Cow<'_, str>, enabled: bool) {
    if enabled
        && current.contains('$')
        && let Some(replaced) = transform_markdown_prose_segments(current, replace_math)
    {
        *current = Cow::Owned(replaced);
    }
}

pub(super) fn replace_math(segment: &str, out: &mut String) {
    if is_indented_code_line(segment) {
        out.push_str(segment);
        return;
    }

    let bytes = segment.as_bytes();
    let mut cursor = 0usize;
    while cursor < bytes.len() {
        let Some(relative) = memchr::memchr(b'$', &bytes[cursor..]) else {
            out.push_str(&segment[cursor..]);
            return;
        };
        let dollar = cursor + relative;
        if dollar > 0 && bytes[dollar - 1] == b'\\' {
            out.push_str(&segment[cursor..dollar - 1]);
            out.push('$');
            cursor = dollar + 1;
            continue;
        }

        out.push_str(&segment[cursor..dollar]);
        if bytes.get(dollar + 1) == Some(&b'$') {
            let inner_start = dollar + 2;
            if let Some(close) = find_block_close(bytes, inner_start) {
                let block = is_block_math_context(bytes, dollar, close);
                emit_math(&segment[inner_start..close], block, out);
                cursor = close + 2;
                continue;
            }
            out.push_str(&segment[dollar..]);
            return;
        }

        if !can_open_inline(bytes, dollar) {
            out.push('$');
            cursor = dollar + 1;
            continue;
        }

        let inner_start = dollar + 1;
        if let Some(close) = find_inline_close(bytes, inner_start) {
            emit_math(&segment[inner_start..close], false, out);
            cursor = close + 1;
            continue;
        }

        out.push_str(&segment[dollar..]);
        return;
    }
}

fn emit_math(tex: &str, block: bool, out: &mut String) {
    if block {
        out.push_str("<div class=\"ox-math ox-math-block\" data-ox-tex=\"");
    } else {
        out.push_str("<span class=\"ox-math ox-math-inline\" data-ox-tex=\"");
    }
    escape_html_attr(tex, out);
    if block {
        out.push_str("\"><math display=\"block\"><mtext>");
    } else {
        out.push_str("\"><math><mtext>");
    }
    escape_html_attr(tex, out);
    if block {
        out.push_str("</mtext></math></div>");
    } else {
        out.push_str("</mtext></math></span>");
    }
}

fn find_block_close(bytes: &[u8], from: usize) -> Option<usize> {
    let mut cursor = from;
    while cursor < bytes.len() {
        if bytes[cursor] == b'\\' && bytes.get(cursor + 1) == Some(&b'$') {
            cursor += 2;
            continue;
        }
        if bytes[cursor] == b'$' && bytes.get(cursor + 1) == Some(&b'$') {
            return Some(cursor);
        }
        cursor += 1;
    }
    None
}

fn find_inline_close(bytes: &[u8], from: usize) -> Option<usize> {
    let mut cursor = from;
    while cursor < bytes.len() {
        if bytes[cursor] == b'\n' {
            return None;
        }
        if bytes[cursor] == b'\\' && bytes.get(cursor + 1) == Some(&b'$') {
            cursor += 2;
            continue;
        }
        if bytes[cursor] == b'$' && can_close_inline(bytes, cursor) {
            return Some(cursor);
        }
        cursor += 1;
    }
    None
}

fn is_block_math_context(bytes: &[u8], open: usize, close: usize) -> bool {
    is_line_start(bytes, open) && is_line_end(bytes, close + 2)
}

fn is_line_start(bytes: &[u8], index: usize) -> bool {
    index == 0
        || bytes[..index]
            .iter()
            .rev()
            .take_while(|byte| **byte != b'\n')
            .all(u8::is_ascii_whitespace)
}

fn is_line_end(bytes: &[u8], index: usize) -> bool {
    index >= bytes.len()
        || bytes[index..].iter().take_while(|byte| **byte != b'\n').all(u8::is_ascii_whitespace)
}

fn can_open_inline(bytes: &[u8], index: usize) -> bool {
    let next = bytes.get(index + 1).copied();
    let prev = index.checked_sub(1).and_then(|prev| bytes.get(prev).copied());
    !matches!(next, None | Some(b' ' | b'\t' | b'\n' | b'0'..=b'9'))
        && !matches!(prev, Some(b'0'..=b'9'))
}

fn can_close_inline(bytes: &[u8], index: usize) -> bool {
    let prev = index.checked_sub(1).and_then(|prev| bytes.get(prev).copied());
    let next = bytes.get(index + 1).copied();
    !matches!(prev, None | Some(b' ' | b'\t' | b'\n')) && !matches!(next, Some(b'0'..=b'9'))
}

fn is_indented_code_line(segment: &str) -> bool {
    let line = segment.split_once('\n').map_or(segment, |(line, _)| line);
    line.starts_with("    ") || line.starts_with('\t')
}
