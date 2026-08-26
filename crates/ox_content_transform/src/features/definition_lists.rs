//! Opt-in PHP Markdown Extra / mdBook-style definition lists.
//!
//! Disabled by default. When enabled:
//!
//! ```md
//! Term
//! : Definition text
//! : Another definition
//! ```
//!
//! becomes `<dl class="ox-definition-list">` with `<dt>` / `<dd>` children.
//! Inner Markdown is left for the native parser. Fenced, indented, and inline
//! code stay untouched. Invalid or ambiguous forms remain paragraphs or lists.

use crate::DefinitionListOptions;

use super::segments::{is_closing_fence, is_indented_code_line, parse_opening_fence};

mod parse;
#[cfg(test)]
mod tests;

use parse::{contains_ignore_ascii_case, is_blank, parse_list, raw_html_open, split_ending};

#[derive(Clone)]
pub(super) struct ResolvedDefinitionLists;

pub(super) struct Item {
    terms: Vec<String>,
    defs: Vec<String>,
}

pub(super) fn resolve(options: Option<&DefinitionListOptions>) -> Option<ResolvedDefinitionLists> {
    let options = options?;
    if options.enabled == Some(false) {
        return None;
    }
    Some(ResolvedDefinitionLists)
}

pub(super) fn transform(source: &str) -> String {
    let lines: Vec<(&str, &str)> = source.split_inclusive('\n').map(split_ending).collect();
    let mut out = String::with_capacity(source.len() + 64);
    let mut index = 0usize;
    let mut in_fence = false;
    let mut fence_char = b'\0';
    let mut fence_len = 0usize;
    let mut in_comment = false;
    let mut raw_close: Option<&'static str> = None;

    while index < lines.len() {
        let (line, ending) = lines[index];

        if in_fence {
            push_line(&mut out, line, ending);
            if is_closing_fence(line, fence_char, fence_len) {
                in_fence = false;
            }
            index += 1;
            continue;
        }
        if in_comment {
            push_line(&mut out, line, ending);
            in_comment = !line.contains("-->");
            index += 1;
            continue;
        }
        if let Some(close) = raw_close {
            push_line(&mut out, line, ending);
            if contains_ignore_ascii_case(line, close) {
                raw_close = None;
            }
            index += 1;
            continue;
        }
        if let Some(open) = parse_opening_fence(line) {
            in_fence = true;
            fence_char = open.fence_char;
            fence_len = open.fence_len;
            push_line(&mut out, line, ending);
            index += 1;
            continue;
        }
        if is_indented_code_line(line) {
            push_line(&mut out, line, ending);
            index += 1;
            continue;
        }
        if line.trim_start().starts_with("<!--") {
            push_line(&mut out, line, ending);
            in_comment = !line.contains("-->");
            index += 1;
            continue;
        }
        if let Some(close) = raw_html_open(line) {
            push_line(&mut out, line, ending);
            if !contains_ignore_ascii_case(line, close) {
                raw_close = Some(close);
            }
            index += 1;
            continue;
        }

        let prev_blank = index == 0 || is_blank(lines[index - 1].0);
        if prev_blank && let Some((items, end)) = parse_list(&lines, index) {
            emit_list(&mut out, &items);
            index = end;
            continue;
        }

        push_line(&mut out, line, ending);
        index += 1;
    }

    out
}

fn emit_list(out: &mut String, items: &[Item]) {
    out.push_str("<dl class=\"ox-definition-list\">\n\n");
    for item in items {
        for term in &item.terms {
            emit_part(out, "dt", term);
        }
        for def in &item.defs {
            emit_part(out, "dd", def);
        }
    }
    out.push_str("</dl>\n");
}

fn emit_part(out: &mut String, tag: &str, text: &str) {
    out.push('<');
    out.push_str(tag);
    out.push_str(">\n\n");
    out.push_str(text);
    if !text.is_empty() && !text.ends_with('\n') {
        out.push('\n');
    }
    out.push_str("\n</");
    out.push_str(tag);
    out.push_str(">\n\n");
}

fn push_line(out: &mut String, line: &str, ending: &str) {
    out.push_str(line);
    out.push_str(ending);
}
