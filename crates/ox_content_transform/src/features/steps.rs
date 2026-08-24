//! Opt-in `::: steps` ordered lists.
//!
//! Disabled by default. A closed `::: steps` / `:::` wrapper around an ordered
//! list is rewritten to `<div class="ox-steps">` markup. Unclosed openers stay
//! literal and do not consume the rest of the file.

use crate::StepsOptions;

use super::escape_html_text;

#[cfg(test)]
mod tests;

#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) struct ResolvedStepsOptions;

pub(super) fn resolve(options: Option<&StepsOptions>) -> Option<ResolvedStepsOptions> {
    let options = options?;
    if options.enabled == Some(false) {
        return None;
    }
    Some(ResolvedStepsOptions)
}

pub(super) fn transform(source: &str) -> String {
    let lines: Vec<(&str, &str)> = source.split_inclusive('\n').map(split_ending).collect();
    let mut out = String::with_capacity(source.len() + 128);
    let mut index = 0usize;
    let mut in_fence = false;
    let mut fence_char = b'\0';
    let mut fence_len = 0usize;

    while index < lines.len() {
        let (line, ending) = lines[index];

        if in_fence {
            out.push_str(line);
            out.push_str(ending);
            if is_closing_fence(line, fence_char, fence_len) {
                in_fence = false;
            }
            index += 1;
            continue;
        }

        if let Some(open) = parse_opening_fence(line) {
            in_fence = true;
            fence_char = open.0;
            fence_len = open.1;
            out.push_str(line);
            out.push_str(ending);
            index += 1;
            continue;
        }

        if let Some(colon_count) = parse_steps_opener(trim_container_indent(line))
            && let Some(close_at) = find_closer(&lines, index + 1, colon_count)
        {
            let body = collect_body(&lines[index + 1..close_at]);
            emit_steps(&mut out, &body);
            index = close_at + 1;
            continue;
        }

        out.push_str(line);
        out.push_str(ending);
        index += 1;
    }

    out
}

fn find_closer(lines: &[(&str, &str)], start: usize, opener_colons: usize) -> Option<usize> {
    let mut in_fence = false;
    let mut fence_char = b'\0';
    let mut fence_len = 0usize;

    for (offset, (line, _)) in lines[start..].iter().enumerate() {
        if in_fence {
            if is_closing_fence(line, fence_char, fence_len) {
                in_fence = false;
            }
            continue;
        }
        if let Some(open) = parse_opening_fence(line) {
            in_fence = true;
            fence_char = open.0;
            fence_len = open.1;
            continue;
        }
        let trimmed = trim_container_indent(line);
        if parse_steps_opener(trimmed).is_some() {
            return None;
        }
        if let Some(close) = parse_closer(trimmed)
            && close >= opener_colons
        {
            return Some(start + offset);
        }
    }
    None
}

fn collect_body(lines: &[(&str, &str)]) -> String {
    let mut body = String::new();
    for (line, ending) in lines {
        body.push_str(line);
        body.push_str(ending);
    }
    body
}

fn emit_steps(out: &mut String, body: &str) {
    let (preamble, items) = split_ordered_items(body);
    out.push_str("<div class=\"ox-steps\">\n");
    if !items.is_empty() && !preamble.trim().is_empty() {
        let escaped_preamble = escape_item_markdown(&preamble);
        out.push_str(escaped_preamble.trim_end());
        out.push_str("\n\n");
    }
    out.push_str("<ol class=\"ox-steps__list\">\n");
    if items.is_empty() {
        let escaped = escape_item_markdown(body);
        if !escaped.trim().is_empty() {
            out.push_str("<li class=\"ox-steps__item\">\n\n");
            out.push_str(escaped.trim_end());
            out.push_str("\n\n</li>\n");
        }
    } else {
        for item in items {
            out.push_str("<li class=\"ox-steps__item\">\n\n");
            out.push_str(item.trim_end());
            out.push_str("\n\n</li>\n");
        }
    }
    out.push_str("</ol>\n</div>\n");
}

fn split_ordered_items(body: &str) -> (String, Vec<String>) {
    let mut preamble = String::new();
    let mut items = Vec::new();
    let mut current: Option<String> = None;
    let mut item_indent: Option<usize> = None;
    let mut in_fence = false;
    let mut fence_char = b'\0';
    let mut fence_len = 0usize;

    for line_with_end in body.split_inclusive('\n') {
        let (line, ending) = split_ending(line_with_end);

        if in_fence {
            push_item_line(&mut current, &mut preamble, line, ending);
            if is_closing_fence(line, fence_char, fence_len) {
                in_fence = false;
            }
            continue;
        }

        if let Some(open) = parse_opening_fence(line) {
            in_fence = true;
            fence_char = open.0;
            fence_len = open.1;
            push_item_line(&mut current, &mut preamble, line, ending);
            continue;
        }

        if let Some(marker) = parse_ordered_marker(line)
            && item_indent.is_none_or(|indent| marker.indent == indent)
        {
            item_indent = Some(marker.indent);
            if let Some(item) = current.take() {
                items.push(escape_item_markdown(&item));
            }
            current = Some(String::new());
            if let Some(buf) = current.as_mut() {
                buf.push_str(marker.rest);
                buf.push_str(ending);
            }
            continue;
        }

        if let Some(buf) = current.as_mut() {
            let stripped = strip_item_indent(line, item_indent.unwrap_or(0));
            buf.push_str(stripped);
            buf.push_str(ending);
        } else {
            preamble.push_str(line);
            preamble.push_str(ending);
        }
    }

    if let Some(item) = current.take() {
        items.push(escape_item_markdown(&item));
    }
    (preamble, items)
}

fn push_item_line(current: &mut Option<String>, preamble: &mut String, line: &str, ending: &str) {
    if let Some(buf) = current.as_mut() {
        buf.push_str(line);
        buf.push_str(ending);
    } else {
        preamble.push_str(line);
        preamble.push_str(ending);
    }
}

struct OrderedMarker<'a> {
    indent: usize,
    rest: &'a str,
}

fn parse_ordered_marker(line: &str) -> Option<OrderedMarker<'_>> {
    let bytes = line.as_bytes();
    let mut indent = 0usize;
    while indent < bytes.len() && indent < 3 && bytes[indent] == b' ' {
        indent += 1;
    }
    let rest = &line[indent..];
    let digits = rest.bytes().take_while(u8::is_ascii_digit).count();
    if digits == 0 || digits > 9 {
        return None;
    }
    let after_digits = rest.as_bytes().get(digits)?;
    if *after_digits != b'.' && *after_digits != b')' {
        return None;
    }
    let after_marker = &rest[digits + 1..];
    if after_marker.is_empty() {
        return Some(OrderedMarker { indent, rest: "" });
    }
    if !after_marker.as_bytes().first().is_some_and(|byte| *byte == b' ' || *byte == b'\t') {
        return None;
    }
    Some(OrderedMarker { indent, rest: after_marker.trim_start() })
}

fn strip_item_indent(line: &str, item_indent: usize) -> &str {
    let bytes = line.as_bytes();
    let mut stripped = 0usize;
    let max = item_indent + 3;
    while stripped < bytes.len() && stripped < max && bytes[stripped] == b' ' {
        stripped += 1;
    }
    &line[stripped..]
}

fn escape_item_markdown(source: &str) -> String {
    let mut out = String::with_capacity(source.len());
    let mut in_fence = false;
    let mut fence_char = b'\0';
    let mut fence_len = 0usize;

    for line_with_end in source.split_inclusive('\n') {
        let (line, ending) = split_ending(line_with_end);
        if in_fence {
            out.push_str(line);
            out.push_str(ending);
            if is_closing_fence(line, fence_char, fence_len) {
                in_fence = false;
            }
            continue;
        }
        if let Some(open) = parse_opening_fence(line) {
            in_fence = true;
            fence_char = open.0;
            fence_len = open.1;
            out.push_str(line);
            out.push_str(ending);
            continue;
        }
        escape_html_text(line, &mut out);
        out.push_str(ending);
    }
    out
}

fn parse_steps_opener(line: &str) -> Option<usize> {
    let colon_count = line.bytes().take_while(|byte| *byte == b':').count();
    if colon_count < 3 {
        return None;
    }
    let rest = line[colon_count..].trim_start();
    let end = rest
        .find(|ch: char| ch.is_ascii_whitespace() || ch == '[' || ch == '{')
        .unwrap_or(rest.len());
    let name = &rest[..end];
    name.eq_ignore_ascii_case("steps").then_some(colon_count)
}

fn parse_closer(line: &str) -> Option<usize> {
    let colon_count = line.bytes().take_while(|byte| *byte == b':').count();
    if colon_count < 3 {
        return None;
    }
    line[colon_count..].bytes().all(|byte| byte.is_ascii_whitespace()).then_some(colon_count)
}

fn split_ending(line_with_end: &str) -> (&str, &str) {
    if let Some(line) = line_with_end.strip_suffix("\r\n") {
        (line, "\n")
    } else if let Some(line) = line_with_end.strip_suffix('\n') {
        (line, "\n")
    } else {
        (line_with_end.strip_suffix('\r').unwrap_or(line_with_end), "")
    }
}

fn trim_container_indent(line: &str) -> &str {
    let bytes = line.as_bytes();
    let mut indent = 0usize;
    while indent < bytes.len() && indent < 3 && bytes[indent] == b' ' {
        indent += 1;
    }
    &line[indent..]
}

fn parse_opening_fence(line: &str) -> Option<(u8, usize)> {
    let trimmed = line.trim_start();
    let bytes = trimmed.as_bytes();
    let fence_char = *bytes.first()?;
    if fence_char != b'`' && fence_char != b'~' {
        return None;
    }
    let fence_len = bytes.iter().take_while(|byte| **byte == fence_char).count();
    (fence_len >= 3).then_some((fence_char, fence_len))
}

fn is_closing_fence(line: &str, fence_char: u8, fence_len: usize) -> bool {
    let trimmed = line.trim_start();
    let bytes = trimmed.as_bytes();
    let close_len = bytes.iter().take_while(|byte| **byte == fence_char).count();
    close_len >= fence_len
        && bytes.get(close_len..).is_none_or(|rest| rest.iter().all(u8::is_ascii_whitespace))
}
