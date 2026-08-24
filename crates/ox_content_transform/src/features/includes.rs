//! Opt-in Markdown file includes via `<!-- @include: PATH -->`.
//!
//! Disabled by default. Expansion runs at preprocess time, before Markdown is
//! parsed, and is skipped inside fenced, indented, and inline code.

use std::fs;
use std::path::{Path, PathBuf};

use crate::IncludeOptions;

use super::segments::{is_closing_fence, parse_opening_fence, transform_inline_code_segments};

#[cfg(test)]
mod tests;

const MAX_INCLUDE_DEPTH: usize = 16;

#[derive(Clone)]
pub(super) struct ResolvedIncludeOptions {
    root_dir: PathBuf,
    source_path: Option<PathBuf>,
}

pub(super) fn resolve(
    options: Option<&IncludeOptions>,
    source_path: Option<&str>,
) -> Option<ResolvedIncludeOptions> {
    let options = options?;
    if options.enabled == Some(false) {
        return None;
    }
    let root_dir = options.root_dir.as_deref().filter(|value| !value.is_empty()).map_or_else(
        || std::env::current_dir().unwrap_or_else(|_| PathBuf::from(".")),
        PathBuf::from,
    );
    Some(ResolvedIncludeOptions {
        root_dir,
        source_path: source_path.filter(|value| !value.is_empty()).map(PathBuf::from),
    })
}

pub(super) fn transform(
    source: &str,
    options: &ResolvedIncludeOptions,
    errors: &mut Vec<String>,
) -> String {
    let mut stack = Vec::new();
    if let Some(path) = &options.source_path
        && let Ok(canonical) = path.canonicalize()
    {
        stack.push(canonical);
    }
    expand(source, options, errors, &mut stack, options.source_path.as_deref(), 0)
}

fn expand(
    source: &str,
    options: &ResolvedIncludeOptions,
    errors: &mut Vec<String>,
    stack: &mut Vec<PathBuf>,
    current_source: Option<&Path>,
    depth: usize,
) -> String {
    let mut out = String::with_capacity(source.len());
    let mut in_fence = false;
    let mut fence_char = b'\0';
    let mut fence_len = 0usize;

    for line_with_end in source.split_inclusive('\n') {
        let (line, ending) = match line_with_end.strip_suffix('\n') {
            Some(line) => (line, "\n"),
            None => (line_with_end, ""),
        };

        if in_fence {
            out.push_str(line);
            out.push_str(ending);
            if is_closing_fence(line, fence_char, fence_len) {
                in_fence = false;
                fence_char = b'\0';
                fence_len = 0;
            }
            continue;
        }

        if let Some(open) = parse_opening_fence(line) {
            in_fence = true;
            fence_char = open.fence_char;
            fence_len = open.fence_len;
            out.push_str(line);
            out.push_str(ending);
            continue;
        }

        if is_indented_code_line(line) {
            out.push_str(line);
            out.push_str(ending);
            continue;
        }

        transform_inline_code_segments(line, &mut out, &mut |segment, out| {
            expand_text_segment(segment, options, errors, stack, current_source, depth, out);
        });
        out.push_str(ending);
    }

    out
}

fn expand_text_segment(
    segment: &str,
    options: &ResolvedIncludeOptions,
    errors: &mut Vec<String>,
    stack: &mut Vec<PathBuf>,
    current_source: Option<&Path>,
    depth: usize,
    out: &mut String,
) {
    let mut cursor = 0usize;
    while let Some(rel) = segment[cursor..].find("<!--") {
        let start = cursor + rel;
        out.push_str(&segment[cursor..start]);
        let after_open = start + 4;
        let Some(rel_close) = segment[after_open..].find("-->") else {
            out.push_str(&segment[start..]);
            return;
        };
        let close = after_open + rel_close;
        let directive = &segment[start..close + 3];
        if let Some(path) = parse_include_path(segment[after_open..close].trim()) {
            if let Some(content) =
                include_file(&path, options, errors, stack, current_source, depth)
            {
                out.push_str(&content);
            } else {
                out.push_str(directive);
            }
        } else {
            out.push_str(directive);
        }
        cursor = close + 3;
    }
    out.push_str(&segment[cursor..]);
}

fn parse_include_path(inner: &str) -> Option<String> {
    let rest = inner.strip_prefix("@include:")?;
    Some(unquote(rest.trim()))
}

fn unquote(value: &str) -> String {
    let bytes = value.as_bytes();
    if bytes.len() >= 2
        && ((bytes[0] == b'"' && bytes[bytes.len() - 1] == b'"')
            || (bytes[0] == b'\'' && bytes[bytes.len() - 1] == b'\''))
    {
        value[1..value.len() - 1].to_string()
    } else {
        value.to_string()
    }
}

fn include_file(
    path: &str,
    options: &ResolvedIncludeOptions,
    errors: &mut Vec<String>,
    stack: &mut Vec<PathBuf>,
    current_source: Option<&Path>,
    depth: usize,
) -> Option<String> {
    if path.is_empty() {
        errors.push("Include directive is missing a path.".to_string());
        return None;
    }
    if depth >= MAX_INCLUDE_DEPTH {
        errors.push(format!("Include nesting exceeds the maximum depth of {MAX_INCLUDE_DEPTH}."));
        return None;
    }
    let resolved_path = match resolve_include_path(path, options, current_source) {
        Ok(path) => path,
        Err(error) => {
            errors.push(error);
            return None;
        }
    };
    if stack.iter().any(|entry| entry == &resolved_path) {
        errors.push(format!("Include cycle detected: {}.", resolved_path.display()));
        return None;
    }
    let source = match fs::read_to_string(&resolved_path) {
        Ok(source) => source,
        Err(error) => {
            errors.push(format!("Failed to read include {}: {error}", resolved_path.display()));
            return None;
        }
    };
    stack.push(resolved_path.clone());
    let expanded = expand(&source, options, errors, stack, Some(&resolved_path), depth + 1);
    stack.pop();
    Some(expanded)
}

fn resolve_include_path(
    value: &str,
    options: &ResolvedIncludeOptions,
    current_source: Option<&Path>,
) -> Result<PathBuf, String> {
    let candidate = if let Some(rest) = value.strip_prefix("@/") {
        options.root_dir.join(rest)
    } else if let Some(rest) = value.strip_prefix('/') {
        options.root_dir.join(rest)
    } else if let Some(source_path) = current_source {
        source_path.parent().unwrap_or_else(|| Path::new(".")).join(value)
    } else {
        options.root_dir.join(value)
    };

    let canonical_root =
        options.root_dir.canonicalize().unwrap_or_else(|_| options.root_dir.clone());
    let canonical_candidate = candidate.canonicalize().map_err(|error| {
        format!("Include path {} could not be resolved: {error}", candidate.display())
    })?;

    if !canonical_candidate.starts_with(&canonical_root) {
        return Err(format!(
            "Include path {} is outside root {}.",
            canonical_candidate.display(),
            canonical_root.display()
        ));
    }

    Ok(canonical_candidate)
}

fn is_indented_code_line(line: &str) -> bool {
    line.starts_with('\t') || line.starts_with("    ")
}
