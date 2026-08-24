//! Opt-in `::: card`, `::: link-card`, and `::: card-grid` blocks.
//!
//! Disabled by default. Unclosed blocks stay literal so they cannot swallow the
//! rest of a file. Hostile titles are escaped; `javascript:`, `data:`,
//! `vbscript:`, and protocol-relative `//` hrefs are never emitted on an
//! anchor.

use crate::CardOptions;

mod parse;
#[cfg(test)]
mod tests;

use super::segments::{is_closing_fence, parse_opening_fence};
use super::{escape_html_attr, escape_html_text};
use parse::{
    CardKind, ParsedCardOpener, is_safe_href, parse_any_opener, parse_closer, parse_opener,
};

const CARD_TYPES: &[&str] = &["card", "link-card", "card-grid"];

#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) struct ResolvedCardOptions;

pub(super) fn reserved_type_names() -> &'static [&'static str] {
    CARD_TYPES
}

pub(super) fn resolve(options: Option<&CardOptions>) -> Option<ResolvedCardOptions> {
    let options = options?;
    if options.enabled == Some(false) {
        return None;
    }
    Some(ResolvedCardOptions)
}

pub(super) fn transform(source: &str) -> String {
    let lines: Vec<(&str, &str)> = source.split_inclusive('\n').map(split_ending).collect();
    let mut out = String::with_capacity(source.len() + 64);
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
            fence_char = open.fence_char;
            fence_len = open.fence_len;
            out.push_str(line);
            out.push_str(ending);
            index += 1;
            continue;
        }

        let trimmed = trim_container_indent(line);
        if let Some(opener) = parse_opener(trimmed)
            && let Some(end) = find_closer(&lines, index, opener.colon_count)
        {
            let inner = collect_inner(&lines, index + 1, end);
            emit_block(&mut out, &opener, &inner);
            index = end + 1;
            continue;
        }

        out.push_str(line);
        out.push_str(ending);
        index += 1;
    }

    out
}

fn find_closer(lines: &[(&str, &str)], start: usize, colon_count: usize) -> Option<usize> {
    let mut in_fence = false;
    let mut fence_char = b'\0';
    let mut fence_len = 0usize;
    let mut stack = vec![colon_count];

    for (offset, (line, _)) in lines.iter().enumerate().skip(start + 1) {
        if in_fence {
            if is_closing_fence(line, fence_char, fence_len) {
                in_fence = false;
            }
            continue;
        }

        if let Some(open) = parse_opening_fence(line) {
            in_fence = true;
            fence_char = open.fence_char;
            fence_len = open.fence_len;
            continue;
        }

        let trimmed = trim_container_indent(line);
        if let Some(opener) = parse_any_opener(trimmed) {
            stack.push(opener);
            continue;
        }

        if let Some(close) = parse_closer(trimmed)
            && let Some(index) = stack.iter().rposition(|open| *open <= close)
        {
            stack.truncate(index);
            if stack.is_empty() {
                return Some(offset);
            }
        }
    }

    None
}

fn collect_inner(lines: &[(&str, &str)], start: usize, end: usize) -> String {
    let mut inner = String::new();
    for (line, ending) in &lines[start..end] {
        inner.push_str(line);
        inner.push_str(ending);
    }
    inner
}

fn emit_block(out: &mut String, opener: &ParsedCardOpener, inner: &str) {
    match opener.kind {
        CardKind::Card => emit_card(out, opener.title.as_deref(), inner),
        CardKind::LinkCard => emit_link_card(out, opener, inner),
        CardKind::CardGrid => {
            let nested = transform(inner);
            out.push_str("<div class=\"ox-card-grid\">\n\n");
            out.push_str(&nested);
            if !nested.ends_with('\n') && !nested.is_empty() {
                out.push('\n');
            }
            out.push_str("</div>\n");
        }
    }
}

fn emit_card(out: &mut String, title: Option<&str>, inner: &str) {
    out.push_str("<article class=\"ox-card\">\n");
    if let Some(title) = title.filter(|value| !value.is_empty()) {
        out.push_str("<p class=\"ox-card-title\">");
        escape_html_text(title, out);
        out.push_str("</p>\n");
    }
    out.push('\n');
    out.push_str(inner);
    if !inner.ends_with('\n') && !inner.is_empty() {
        out.push('\n');
    }
    out.push_str("</article>\n");
}

fn emit_link_card(out: &mut String, opener: &ParsedCardOpener, inner: &str) {
    let title = opener.title.as_deref().unwrap_or("");
    let description = trim_description(inner);
    let href = opener.href.as_deref().map(str::trim).filter(|value| !value.is_empty());

    match href {
        Some(href) if is_safe_href(href) => {
            out.push_str("<a class=\"ox-link-card\" href=\"");
            escape_html_attr(href, out);
            out.push_str("\">");
            emit_link_card_body(out, title, description);
            out.push_str("</a>\n");
        }
        _ => {
            out.push_str("<span class=\"ox-link-card\">");
            emit_link_card_body(out, title, description);
            out.push_str("</span>\n");
        }
    }
}

fn emit_link_card_body(out: &mut String, title: &str, description: &str) {
    out.push_str("<span class=\"ox-link-card-title\">");
    escape_html_text(title, out);
    out.push_str("</span>");
    if !description.is_empty() {
        out.push_str("<span class=\"ox-link-card-description\">");
        escape_html_text(description, out);
        out.push_str("</span>");
    }
}

fn trim_description(inner: &str) -> &str {
    inner.trim_matches(|ch| ch == '\n' || ch == '\r').trim()
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
