//! Opt-in `file-tree` fences.
//!
//! Disabled by default. Fences are rewritten to a static HTML tree. Names are
//! never read from the filesystem and never executed. Directory rows that have
//! children use `<details>`/`<summary>` so they open and close without JS.

use rustc_hash::FxHashMap;

use crate::FileTreeOptions;

use super::segments::{is_closing_fence, parse_opening_fence};

mod html;
mod icons;
mod parse;

#[cfg(test)]
mod tests;

const LANGUAGE: &str = "file-tree";

#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) struct ResolvedFileTreeOptions {
    default_open: bool,
    icons: bool,
    icon_folder: Option<String>,
    icon_folder_open: Option<String>,
    icon_file: Option<String>,
    icon_files: FxHashMap<String, String>,
}

impl Default for ResolvedFileTreeOptions {
    fn default() -> Self {
        Self {
            default_open: true,
            icons: true,
            icon_folder: None,
            icon_folder_open: None,
            icon_file: None,
            icon_files: FxHashMap::default(),
        }
    }
}

pub(super) fn resolve(options: Option<&FileTreeOptions>) -> Option<ResolvedFileTreeOptions> {
    let options = options?;
    if options.enabled == Some(false) {
        return None;
    }
    Some(ResolvedFileTreeOptions {
        default_open: options.default_open.unwrap_or(true),
        icons: options.icons.unwrap_or(true),
        icon_folder: nonempty(options.icon_folder.as_deref()),
        icon_folder_open: nonempty(options.icon_folder_open.as_deref()),
        icon_file: nonempty(options.icon_file.as_deref()),
        icon_files: normalize_files(options.icon_files.as_ref()),
    })
}

pub(super) fn transform(source: &str, options: &ResolvedFileTreeOptions) -> String {
    if !source.contains(LANGUAGE) {
        return source.to_string();
    }
    rewrite(source, options)
}

fn rewrite(source: &str, options: &ResolvedFileTreeOptions) -> String {
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
                html::emit_tree(&body, options, &mut out);
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

fn nonempty(value: Option<&str>) -> Option<String> {
    value.map(str::trim).filter(|value| !value.is_empty()).map(str::to_string)
}

fn normalize_files(files: Option<&FxHashMap<String, String>>) -> FxHashMap<String, String> {
    let Some(files) = files else {
        return FxHashMap::default();
    };
    files
        .iter()
        .filter_map(|(ext, svg)| {
            let ext = ext.trim().trim_start_matches('.').to_ascii_lowercase();
            let svg = svg.trim();
            (!ext.is_empty() && !svg.is_empty()).then(|| (ext, svg.to_string()))
        })
        .collect()
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
