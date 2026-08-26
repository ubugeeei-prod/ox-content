//! Opt-in VitePress-style `::: code-group` fence groups.
//!
//! Disabled by default. A closed group of labeled fences is rewritten to
//! `<tabs><tab label="…">` markup so the existing no-JS `transform_tabs`
//! widget can expand it after Markdown render. Unclosed openers stay
//! literal. Unknown or malformed group metadata degrades to ordinary
//! fences and records a warning.

use crate::CodeGroupOptions;

use super::escape::escape_html_attr;
use super::segments::{is_closing_fence, parse_opening_fence};

mod parse;
#[cfg(test)]
mod tests;

use parse::{
    GroupInner, parse_any_opener, parse_closer, parse_opener, split_ending, trim_container_indent,
};

#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) struct ResolvedCodeGroupOptions;

pub(super) fn reserved_type_names() -> &'static [&'static str] {
    &["code-group"]
}

pub(super) fn resolve(options: Option<&CodeGroupOptions>) -> Option<ResolvedCodeGroupOptions> {
    let options = options?;
    if options.enabled == Some(false) {
        return None;
    }
    Some(ResolvedCodeGroupOptions)
}

pub(super) fn transform(source: &str, errors: &mut Vec<String>) -> String {
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
        if let Some(colon_count) = parse_opener(trimmed)
            && let Some(end) = find_closer(&lines, index, colon_count)
        {
            let inner = collect_inner(&lines, index + 1, end);
            emit_group(&mut out, &inner, errors);
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

fn emit_group(out: &mut String, inner: &str, errors: &mut Vec<String>) {
    match parse::analyze_inner(inner) {
        GroupInner::Fences(fences) => {
            out.push_str("<tabs>\n");
            for fence in fences {
                if let Some(warning) = fence.warning {
                    errors.push(warning);
                }
                out.push_str("<tab label=\"");
                escape_html_attr(&fence.label, out);
                out.push_str("\">\n\n");
                out.push_str(&fence.source);
                if !fence.source.ends_with('\n') {
                    out.push('\n');
                }
                out.push_str("\n</tab>\n");
            }
            out.push_str("</tabs>\n");
        }
        GroupInner::Degrade(warning) => {
            errors.push(warning);
            out.push_str(inner);
        }
    }
}
