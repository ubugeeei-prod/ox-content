//! Opt-in `file-tree` fences.
//!
//! Disabled by default. Fences are rewritten to a static HTML tree. Names are
//! never read from the filesystem and never executed.

use crate::FileTreeOptions;

use super::segments::{is_closing_fence, parse_opening_fence};

#[cfg(test)]
mod tests;

const LANGUAGE: &str = "file-tree";

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(super) struct ResolvedFileTreeOptions;

pub(super) fn resolve(options: Option<&FileTreeOptions>) -> Option<ResolvedFileTreeOptions> {
    let options = options?;
    if options.enabled == Some(false) {
        return None;
    }
    Some(ResolvedFileTreeOptions)
}

pub(super) fn transform(source: &str, _: ResolvedFileTreeOptions) -> String {
    if !source.contains(LANGUAGE) {
        return source.to_string();
    }
    rewrite(source)
}

fn rewrite(source: &str) -> String {
    let mut out = String::with_capacity(source.len());
    let mut in_fence = false;
    let mut fence_char = b'\0';
    let mut fence_len = 0usize;
    let mut lines = source.split_inclusive('\n');

    while let Some(line_with_end) = lines.next() {
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
            let indent = leading_spaces(line);
            if indent < 4 && open.language.as_str() == LANGUAGE {
                let mut body = String::new();
                for inner in lines.by_ref() {
                    let (inner_line, inner_end) = split_ending(inner);
                    if is_closing_fence(inner_line, open.fence_char, open.fence_len) {
                        break;
                    }
                    body.push_str(inner_line);
                    body.push_str(inner_end);
                }
                emit_tree(&body, &mut out);
                continue;
            }
            in_fence = true;
            fence_char = open.fence_char;
            fence_len = open.fence_len;
        }

        out.push_str(line);
        out.push_str(ending);
    }

    out
}

fn emit_tree(body: &str, out: &mut String) {
    let items = parse_items(body);
    out.push_str("<div class=\"ox-file-tree\">\n<ul>\n");
    if items.is_empty() {
        out.push_str("</ul>\n</div>\n");
        return;
    }

    let mut depth = 0usize;
    for (index, item) in items.iter().enumerate() {
        if item.depth < depth {
            out.push_str("</li>\n");
            for _ in item.depth..depth {
                out.push_str("</ul></li>\n");
            }
        } else if item.depth == depth && index > 0 {
            out.push_str("</li>\n");
        } else if item.depth > depth {
            out.push_str("<ul>\n");
        }

        out.push_str("<li class=\"");
        out.push_str(if item.is_dir { "ox-file-tree__dir" } else { "ox-file-tree__file" });
        if item.highlight {
            out.push_str(" ox-file-tree__highlight");
        }
        out.push_str("\">");
        super::escape_html_text(&item.name, out);
        depth = item.depth;
    }

    out.push_str("</li>\n");
    for _ in 0..depth {
        out.push_str("</ul></li>\n");
    }
    out.push_str("</ul>\n</div>\n");
}

struct TreeItem {
    depth: usize,
    name: String,
    is_dir: bool,
    highlight: bool,
}

fn parse_items(body: &str) -> Vec<TreeItem> {
    let mut items = Vec::new();
    let mut previous_depth = 0usize;
    for line in body.lines() {
        if line.trim().is_empty() {
            continue;
        }
        let Some(item) = parse_item(line, previous_depth, items.is_empty()) else {
            continue;
        };
        previous_depth = item.depth;
        items.push(item);
    }
    items
}

fn parse_item(line: &str, previous_depth: usize, first: bool) -> Option<TreeItem> {
    let spaces = leading_spaces(line);
    let rest = line[spaces..].trim_start_matches('\t');
    let name_src = rest.strip_prefix("- ").or_else(|| rest.strip_prefix("-\t"))?;
    if name_src.trim().is_empty() {
        return None;
    }
    let (name, highlight) = parse_name(name_src);
    if name.is_empty() {
        return None;
    }
    let depth = if first { 0 } else { (spaces / 2).min(previous_depth + 1) };
    let is_dir = name.ends_with('/');
    Some(TreeItem { depth, name, is_dir, highlight })
}

fn parse_name(raw: &str) -> (String, bool) {
    let trimmed = raw.trim();
    let (body, trailing) = match trimmed.strip_suffix(" **") {
        Some(body) => (body.trim_end(), true),
        None => (trimmed, false),
    };
    if let Some(inner) = unwrap_bold(body) {
        (inner.to_string(), true)
    } else {
        (body.to_string(), trailing)
    }
}

fn unwrap_bold(value: &str) -> Option<&str> {
    let inner = value.strip_prefix("**")?.strip_suffix("**")?;
    (!inner.is_empty() && !inner.contains("**")).then_some(inner)
}

fn leading_spaces(line: &str) -> usize {
    line.bytes().take_while(|byte| *byte == b' ').count()
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
