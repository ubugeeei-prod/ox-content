//! Opt-in parameterized Markdown partials via `<!-- @partial: PATH k="v" -->`.
//!
//! Disabled by default. Expansion runs at preprocess time, before Markdown is
//! parsed, and is skipped inside fenced, indented, and inline code. Existing
//! `<!-- @include: -->` directives are left unchanged.

use std::fs;
use std::path::{Path, PathBuf};

use crate::PartialsOptions;

use super::segments::{
    is_closing_fence, is_indented_code_line, parse_opening_fence, transform_inline_code_segments,
};

mod parse;
mod path;

#[cfg(test)]
mod tests;

const MAX_PARTIAL_DEPTH: usize = 16;
pub(super) const DEFAULT_ROOT: &str = "_partials";

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum MissingMode {
    Literal,
    Error,
}

#[derive(Clone)]
pub(super) struct ResolvedPartials {
    pub(super) root_dir: PathBuf,
    source_path: Option<PathBuf>,
    pub(super) root: String,
    missing: MissingMode,
}

pub(super) fn resolve(
    options: Option<&PartialsOptions>,
    source_path: Option<&str>,
) -> Option<ResolvedPartials> {
    let options = options?;
    if options.enabled == Some(false) {
        return None;
    }
    let root_dir = options.root_dir.as_deref().filter(|value| !value.is_empty()).map_or_else(
        || std::env::current_dir().unwrap_or_else(|_| PathBuf::from(".")),
        PathBuf::from,
    );
    Some(ResolvedPartials {
        root_dir,
        source_path: source_path.filter(|value| !value.is_empty()).map(PathBuf::from),
        root: options
            .root
            .as_deref()
            .map(str::trim)
            .filter(|value| !value.is_empty())
            .unwrap_or(DEFAULT_ROOT)
            .to_string(),
        missing: match options.missing.as_deref().map(str::trim) {
            Some("error") => MissingMode::Error,
            _ => MissingMode::Literal,
        },
    })
}

pub(super) fn transform(
    source: &str,
    options: &ResolvedPartials,
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
    options: &ResolvedPartials,
    errors: &mut Vec<String>,
    stack: &mut Vec<PathBuf>,
    current_source: Option<&Path>,
    depth: usize,
) -> String {
    let mut out = String::with_capacity(source.len());
    let mut in_fence = false;
    let mut fence_char = b'\0';
    let mut fence_len = 0usize;
    for (line_idx, line_with_end) in source.split_inclusive('\n').enumerate() {
        let line_no = line_idx + 1;
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
            expand_text_segment(
                segment,
                options,
                errors,
                stack,
                (current_source, line_no),
                depth,
                out,
            );
        });
        out.push_str(ending);
    }

    out
}

fn expand_text_segment(
    segment: &str,
    options: &ResolvedPartials,
    errors: &mut Vec<String>,
    stack: &mut Vec<PathBuf>,
    origin: (Option<&Path>, usize),
    depth: usize,
    out: &mut String,
) {
    let (current_source, line_no) = origin;
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
        if let Some(parsed) = parse::parse_partial_directive(segment[after_open..close].trim()) {
            if let Some(content) =
                include_partial(&parsed, options, errors, stack, current_source, depth, line_no)
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

fn include_partial(
    directive: &parse::PartialDirective,
    options: &ResolvedPartials,
    errors: &mut Vec<String>,
    stack: &mut Vec<PathBuf>,
    current_source: Option<&Path>,
    depth: usize,
    line_no: usize,
) -> Option<String> {
    let location = display_location(current_source, line_no);
    if directive.path.is_empty() {
        errors.push(format!("Partial directive is missing a path at {location}."));
        return None;
    }
    if depth >= MAX_PARTIAL_DEPTH {
        errors.push(format!(
            "Partial nesting exceeds the maximum depth of {MAX_PARTIAL_DEPTH} at {location}."
        ));
        return None;
    }
    let resolved_path = match path::resolve_partial_path(&directive.path, options, current_source) {
        Ok(path) => path,
        Err(error) => {
            errors.push(format!("{error} ({location})"));
            return None;
        }
    };
    if stack.iter().any(|entry| entry == &resolved_path) {
        errors.push(format!("Partial cycle detected at {location}: {}.", resolved_path.display()));
        return None;
    }
    let source = match fs::read_to_string(&resolved_path) {
        Ok(source) => source,
        Err(error) => {
            errors.push(format!(
                "Failed to read partial {} at {location}: {error}",
                resolved_path.display()
            ));
            return None;
        }
    };
    let substituted = parse::substitute_params(
        &source,
        &directive.params,
        options.missing,
        errors,
        &format!("{}", resolved_path.display()),
    );
    stack.push(resolved_path.clone());
    let expanded = expand(&substituted, options, errors, stack, Some(&resolved_path), depth + 1);
    stack.pop();
    Some(expanded)
}

fn display_location(current_source: Option<&Path>, line_no: usize) -> String {
    match current_source {
        Some(path) => format!("{}:{line_no}", path.display()),
        None => format!("<input>:{line_no}"),
    }
}
