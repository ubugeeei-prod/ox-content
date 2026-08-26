//! Opt-in `csv-table` / `json-table` fences.
//!
//! Disabled by default. Fences stay ordinary code blocks until `dataTables` is
//! enabled. Enabled fences become a static `<table>` with a responsive wrapper.
//! Cell text is escaped. External files resolve from the content/project root
//! and cannot escape with `..`.

use std::path::{Path, PathBuf};

use crate::DataTableOptions;

use super::segments::{is_closing_fence, parse_opening_fence};

mod html;
mod parse;
mod path;

#[cfg(test)]
mod tests;

const CSV_LANGUAGE: &str = "csv-table";
const JSON_LANGUAGE: &str = "json-table";

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum MissingMode {
    Error,
    Warn,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) struct ResolvedDataTableOptions {
    root_dir: PathBuf,
    source_path: Option<PathBuf>,
    missing: MissingMode,
}

#[derive(Debug)]
pub(super) enum TableError {
    Missing(String),
    Other(String),
}

impl TableError {
    fn message(&self) -> &str {
        match self {
            Self::Missing(message) | Self::Other(message) => message,
        }
    }
}

#[derive(Clone, Copy)]
enum TableKind {
    Csv,
    Json,
}

struct FenceMeta {
    title: Option<String>,
    src: Option<String>,
}

pub(super) fn resolve(
    options: Option<&DataTableOptions>,
    source_path: Option<&str>,
) -> Option<ResolvedDataTableOptions> {
    let options = options?;
    if options.enabled == Some(false) {
        return None;
    }
    let root_dir = options.root_dir.as_deref().filter(|value| !value.is_empty()).map_or_else(
        || std::env::current_dir().unwrap_or_else(|_| PathBuf::from(".")),
        PathBuf::from,
    );
    Some(ResolvedDataTableOptions {
        root_dir,
        source_path: source_path.filter(|value| !value.is_empty()).map(PathBuf::from),
        missing: match options.missing.as_deref().map(str::trim) {
            Some("warn") => MissingMode::Warn,
            _ => MissingMode::Error,
        },
    })
}

pub(super) fn transform(
    source: &str,
    options: &ResolvedDataTableOptions,
    errors: &mut Vec<String>,
) -> String {
    if !source.contains(CSV_LANGUAGE) && !source.contains(JSON_LANGUAGE) {
        return source.to_string();
    }
    rewrite(source, options, errors)
}

fn rewrite(source: &str, options: &ResolvedDataTableOptions, errors: &mut Vec<String>) -> String {
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
            if indent < 4
                && let Some(kind) = kind_from_language(open.language.as_str())
            {
                let mut body = String::new();
                let mut close = None;
                for inner in lines.by_ref() {
                    let (inner_line, inner_end) = split_ending(inner);
                    if is_closing_fence(inner_line, open.fence_char, open.fence_len) {
                        close = Some((inner_line, inner_end));
                        break;
                    }
                    body.push_str(inner_line);
                    body.push_str(inner_end);
                }
                match render_fence(kind, &open.meta, &body, options) {
                    Ok(html) => out.push_str(&html),
                    Err(error) => {
                        let report = !matches!(
                            error,
                            TableError::Missing(_) if options.missing == MissingMode::Warn
                        );
                        if report {
                            errors.push(error.message().to_string());
                        }
                        out.push_str(line);
                        out.push_str(ending);
                        out.push_str(&body);
                        if let Some((close_line, close_end)) = close {
                            out.push_str(close_line);
                            out.push_str(close_end);
                        }
                    }
                }
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

fn render_fence(
    kind: TableKind,
    meta: &str,
    body: &str,
    options: &ResolvedDataTableOptions,
) -> Result<String, TableError> {
    let meta = parse_meta(meta);
    let source = if let Some(src) = meta.src.as_deref().filter(|value| !value.is_empty()) {
        load_external(src, options)?
    } else if let Some(src) = body_as_path(body) {
        load_external(src, options)?
    } else {
        let inline = body.trim();
        if inline.is_empty() {
            return Err(TableError::Other(
                "Data table fence is empty. Provide inline CSV/JSON or src=\"path\".".to_string(),
            ));
        }
        LoadedSource { text: inline.to_string(), kind }
    };
    let data = match source.kind {
        TableKind::Csv => parse::parse_csv(&source.text),
        TableKind::Json => parse::parse_json(&source.text),
    }
    .map_err(TableError::Other)?;
    let mut out = String::new();
    html::emit_table(&data, meta.title.as_deref(), &mut out);
    Ok(out)
}

struct LoadedSource {
    text: String,
    kind: TableKind,
}

fn load_external(
    src: &str,
    options: &ResolvedDataTableOptions,
) -> Result<LoadedSource, TableError> {
    let path = path::resolve_data_path(src, options)?;
    let text = std::fs::read_to_string(&path).map_err(|error| {
        TableError::Missing(format!(
            "Data table file {} could not be read: {error}",
            path.display()
        ))
    })?;
    Ok(LoadedSource { text, kind: kind_from_path(&path, kind_from_src(src)) })
}

fn kind_from_language(language: &str) -> Option<TableKind> {
    match language {
        CSV_LANGUAGE => Some(TableKind::Csv),
        JSON_LANGUAGE => Some(TableKind::Json),
        _ => None,
    }
}

fn kind_from_src(src: &str) -> TableKind {
    kind_from_path(Path::new(src), TableKind::Csv)
}

fn kind_from_path(path: &Path, fallback: TableKind) -> TableKind {
    match path.extension().and_then(|value| value.to_str()).map(str::to_ascii_lowercase) {
        Some(ext) if ext == "json" => TableKind::Json,
        Some(ext) if ext == "csv" => TableKind::Csv,
        _ => fallback,
    }
}

fn body_as_path(body: &str) -> Option<&str> {
    let trimmed = body.trim();
    if trimmed.is_empty() || trimmed.contains('\n') {
        return None;
    }
    if trimmed.starts_with("@/")
        || trimmed.starts_with("./")
        || trimmed.starts_with("../")
        || trimmed.starts_with('/')
    {
        return Some(trimmed);
    }
    let lower = trimmed.to_ascii_lowercase();
    if (lower.ends_with(".csv") || lower.ends_with(".json")) && !trimmed.contains(',') {
        return Some(trimmed);
    }
    None
}

fn parse_meta(meta: &str) -> FenceMeta {
    let mut title = None;
    let mut src = None;
    let mut rest = meta.trim();
    while !rest.is_empty() {
        let Some(eq) = rest.find('=') else { break };
        let key = rest[..eq].trim();
        rest = rest[eq + 1..].trim_start();
        let (value, next) = parse_meta_value(rest);
        match key {
            "title" => title = Some(value),
            "src" => src = Some(value),
            _ => {}
        }
        rest = next.trim_start();
    }
    FenceMeta { title, src }
}

fn parse_meta_value(rest: &str) -> (String, &str) {
    let bytes = rest.as_bytes();
    let Some(first) = bytes.first() else {
        return (String::new(), rest);
    };
    if *first == b'"' || *first == b'\'' {
        let quote = *first;
        if let Some(rel) = rest[1..].find(quote as char) {
            return (rest[1..1 + rel].to_string(), &rest[2 + rel..]);
        }
        return (rest[1..].to_string(), "");
    }
    let end = rest.find(char::is_whitespace).unwrap_or(rest.len());
    (rest[..end].to_string(), &rest[end..])
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
