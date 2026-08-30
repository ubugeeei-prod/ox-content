use super::super::segments::{is_indented_code_line, parse_opening_fence};
use super::Item;

pub(super) fn parse_list(lines: &[(&str, &str)], start: usize) -> Option<(Vec<Item>, usize)> {
    let mut items = Vec::new();
    let mut index = start;
    while let Some((item, next)) = parse_item(lines, index) {
        items.push(item);
        index = next;
        if index < lines.len() && is_blank(lines[index].0) && parse_item(lines, index + 1).is_some()
        {
            index += 1;
            continue;
        }
        break;
    }
    (!items.is_empty()).then_some((items, index))
}

fn parse_item(lines: &[(&str, &str)], start: usize) -> Option<(Item, usize)> {
    if start >= lines.len() || is_blank(lines[start].0) {
        return None;
    }

    let mut terms = Vec::new();
    let mut index = start;
    while index < lines.len() {
        let line = lines[index].0;
        if is_blank(line) {
            break;
        }
        if definition_body(line).is_some() {
            if terms.is_empty() {
                return None;
            }
            break;
        }
        if !is_term_line(line) {
            return None;
        }
        terms.push(line.trim().to_string());
        index += 1;
    }
    if terms.is_empty() {
        return None;
    }
    if index < lines.len() && is_blank(lines[index].0) {
        index += 1;
    }

    let mut defs = Vec::new();
    while index < lines.len() {
        let Some(body) = definition_body(lines[index].0) else {
            break;
        };
        defs.push(body.trim().to_string());
        index += 1;
    }
    if defs.is_empty() {
        return None;
    }
    Some((Item { terms, defs }, index))
}

fn is_term_line(line: &str) -> bool {
    let trimmed = trim_at_most_three_spaces(line);
    !trimmed.is_empty()
        && definition_body(line).is_none()
        && !is_heading(trimmed)
        && !is_list_marker(trimmed)
        && !trimmed.starts_with('>')
        && !trimmed.starts_with(":::")
        && parse_opening_fence(line).is_none()
        && !is_indented_code_line(line)
        && !is_thematic_or_setext(trimmed)
        && !trimmed.starts_with('<')
        && !has_unclosed_inline_code(line)
}

fn has_unclosed_inline_code(line: &str) -> bool {
    let bytes = line.as_bytes();
    let mut cursor = 0usize;
    while cursor < bytes.len() {
        let Some(rel) = bytes[cursor..].iter().position(|byte| *byte == b'`') else {
            return false;
        };
        let start = cursor + rel;
        let ticks = bytes[start..].iter().take_while(|byte| **byte == b'`').count();
        let mut search = start + ticks;
        let mut closed = false;
        while search < bytes.len() {
            if bytes[search] != b'`' {
                search += 1;
                continue;
            }
            let close = bytes[search..].iter().take_while(|byte| **byte == b'`').count();
            if close >= ticks {
                cursor = search + close;
                closed = true;
                break;
            }
            search += close;
        }
        if !closed {
            return true;
        }
    }
    false
}

fn definition_body(line: &str) -> Option<&str> {
    let rest = trim_at_most_three_spaces(line);
    let bytes = rest.as_bytes();
    if bytes.first() != Some(&b':') {
        return None;
    }
    let after = bytes.get(1).copied()?;
    if after != b' ' && after != b'\t' {
        return None;
    }
    Some(rest[2..].trim_start())
}

fn is_heading(trimmed: &str) -> bool {
    let bytes = trimmed.as_bytes();
    let mut hashes = 0usize;
    while hashes < bytes.len() && hashes < 6 && bytes[hashes] == b'#' {
        hashes += 1;
    }
    hashes > 0 && bytes.get(hashes).is_none_or(|byte| *byte == b' ' || *byte == b'\t')
}

fn is_list_marker(trimmed: &str) -> bool {
    let bytes = trimmed.as_bytes();
    if matches!(bytes, [b'-' | b'*' | b'+'] | [b'-' | b'*' | b'+', b' ' | b'\t', ..]) {
        return true;
    }
    let digits = bytes.iter().take_while(|byte| byte.is_ascii_digit()).count();
    if digits == 0 || digits > 9 {
        return false;
    }
    matches!(bytes.get(digits), Some(b'.' | b')'))
        && bytes.get(digits + 1).is_none_or(|byte| *byte == b' ' || *byte == b'\t')
}

fn is_thematic_or_setext(trimmed: &str) -> bool {
    let first = trimmed.as_bytes().first().copied();
    let Some(marker) = first.filter(|byte| matches!(byte, b'-' | b'=' | b'*' | b'_')) else {
        return false;
    };
    let count = trimmed.bytes().filter(|byte| *byte == marker).count();
    count >= 3 && trimmed.bytes().all(|byte| byte == marker || byte == b' ')
}

pub(super) fn raw_html_open(line: &str) -> Option<&'static str> {
    let rest = line.trim_start();
    if !rest.starts_with('<') {
        return None;
    }
    let name_start = rest[1..].trim_start_matches('/');
    for (name, close) in
        [("pre", "</pre"), ("code", "</code"), ("script", "</script"), ("style", "</style")]
    {
        if html_tag_starts(name_start, name) {
            return Some(close);
        }
    }
    None
}

fn html_tag_starts(rest: &str, name: &str) -> bool {
    // Compare bytes: `rest` is arbitrary line text, so slicing it at the
    // name's byte length lands inside a character whenever the line opens
    // with something like `<x\u{3042}`. Every tag name here is ASCII, so a
    // byte-wise match implies the prefix was ASCII too.
    let bytes = rest.as_bytes();
    bytes.len() >= name.len()
        && bytes[..name.len()].eq_ignore_ascii_case(name.as_bytes())
        && bytes
            .get(name.len())
            .is_none_or(|byte| *byte == b'>' || *byte == b'/' || byte.is_ascii_whitespace())
}

pub(super) fn split_ending(line_with_end: &str) -> (&str, &str) {
    if let Some(line) = line_with_end.strip_suffix("\r\n") {
        (line, "\n")
    } else if let Some(line) = line_with_end.strip_suffix('\n') {
        (line, "\n")
    } else {
        (line_with_end.strip_suffix('\r').unwrap_or(line_with_end), "")
    }
}

fn trim_at_most_three_spaces(line: &str) -> &str {
    let bytes = line.as_bytes();
    let mut indent = 0usize;
    while indent < bytes.len() && indent < 3 && bytes[indent] == b' ' {
        indent += 1;
    }
    &line[indent..]
}

pub(super) fn is_blank(line: &str) -> bool {
    line.trim().is_empty()
}

pub(super) fn contains_ignore_ascii_case(haystack: &str, needle: &str) -> bool {
    haystack.to_ascii_lowercase().contains(needle)
}
