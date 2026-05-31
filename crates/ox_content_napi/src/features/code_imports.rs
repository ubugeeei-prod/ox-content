use std::fs;
use std::path::{Path, PathBuf};

use crate::JsCodeImportOptions;

#[derive(Clone)]
pub(super) struct CodeImportOptions {
    root_dir: PathBuf,
    source_path: Option<PathBuf>,
}

pub(super) fn resolve(
    options: Option<&JsCodeImportOptions>,
    source_path: Option<&str>,
) -> Option<CodeImportOptions> {
    let options = options?;
    if options.enabled == Some(false) {
        return None;
    }
    let root_dir = options.root_dir.as_deref().filter(|value| !value.is_empty()).map_or_else(
        || std::env::current_dir().unwrap_or_else(|_| PathBuf::from(".")),
        PathBuf::from,
    );
    Some(CodeImportOptions {
        root_dir,
        source_path: source_path.filter(|value| !value.is_empty()).map(PathBuf::from),
    })
}

pub(super) fn transform(
    source: &str,
    options: &CodeImportOptions,
    errors: &mut Vec<String>,
) -> String {
    let mut out = String::with_capacity(source.len());
    for line_with_end in source.split_inclusive('\n') {
        let (line, ending) = match line_with_end.strip_suffix('\n') {
            Some(line) => (line, "\n"),
            None => (line_with_end, ""),
        };
        let trimmed = line.trim_start();
        if !trimmed.starts_with("<<<") {
            out.push_str(line);
            out.push_str(ending);
            continue;
        }

        match render_code_import(trimmed[3..].trim(), options) {
            Ok(rendered) => out.push_str(&rendered),
            Err(error) => {
                errors.push(error);
                out.push_str(line);
            }
        }
        out.push_str(ending);
    }
    out
}

fn render_code_import(value: &str, options: &CodeImportOptions) -> Result<String, String> {
    if value.is_empty() {
        return Err("Code import directive is missing a path.".to_string());
    }

    let (path_part, selector) = split_import_selector(value);
    let path = resolve_import_path(path_part.trim(), options)?;
    let source = fs::read_to_string(&path)
        .map_err(|error| format!("Failed to read imported code {}: {error}", path.display()))?;
    let selected = select_import_region(&source, selector)?;
    let lang = language_from_path(&path);
    let mut out = String::with_capacity(selected.len() + lang.len() + 8);
    out.push_str("```");
    out.push_str(&lang);
    out.push('\n');
    out.push_str(selected.trim_end_matches('\n'));
    out.push_str("\n```");
    Ok(out)
}

fn split_import_selector(value: &str) -> (&str, Option<&str>) {
    let Some(close) = value.rfind('}') else {
        return (value, None);
    };
    let Some(open) = value[..close].rfind('{') else {
        return (value, None);
    };
    let selector = value[open + 1..close].trim();
    let path = value[..open].trim_end();
    if selector.is_empty() {
        (path, None)
    } else {
        (path, Some(selector))
    }
}

fn resolve_import_path(value: &str, options: &CodeImportOptions) -> Result<PathBuf, String> {
    let unquoted = value.trim_matches(|ch| ch == '"' || ch == '\'');
    let candidate = if let Some(rest) = unquoted.strip_prefix("@/") {
        options.root_dir.join(rest)
    } else if let Some(rest) = unquoted.strip_prefix('/') {
        options.root_dir.join(rest)
    } else if let Some(source_path) = &options.source_path {
        source_path.parent().unwrap_or_else(|| Path::new(".")).join(unquoted)
    } else {
        options.root_dir.join(unquoted)
    };

    let canonical_root =
        options.root_dir.canonicalize().unwrap_or_else(|_| options.root_dir.clone());
    let canonical_candidate = candidate.canonicalize().map_err(|error| {
        format!("Code import path {} could not be resolved: {error}", candidate.display())
    })?;

    if !canonical_candidate.starts_with(&canonical_root) {
        return Err(format!(
            "Code import path {} is outside root {}.",
            canonical_candidate.display(),
            canonical_root.display()
        ));
    }

    Ok(canonical_candidate)
}

fn select_import_region<'a>(source: &'a str, selector: Option<&str>) -> Result<&'a str, String> {
    let Some(selector) = selector else {
        return Ok(source);
    };
    if let Some((start, end)) = parse_line_selector(selector) {
        return select_line_range(source, start, end);
    }
    select_named_region(source, selector.trim_start_matches('#'))
}

fn parse_line_selector(selector: &str) -> Option<(usize, usize)> {
    let normalized = selector.trim().trim_start_matches('L');
    let (start, end) =
        normalized.split_once('-').map_or((normalized, normalized), |(start, end)| {
            (start.trim_start_matches('L'), end.trim_start_matches('L'))
        });
    let start = start.parse::<usize>().ok()?;
    let end = end.parse::<usize>().ok()?;
    if start == 0 || end < start {
        None
    } else {
        Some((start, end))
    }
}

fn select_line_range(source: &str, start: usize, end: usize) -> Result<&str, String> {
    let mut byte_start = None;
    let mut byte_end = source.len();
    let mut cursor = 0usize;
    for (line_no, line) in (1usize..).zip(source.split_inclusive('\n')) {
        if line_no == start {
            byte_start = Some(cursor);
        }
        if line_no == end + 1 {
            byte_end = cursor;
            break;
        }
        cursor += line.len();
    }
    let Some(byte_start) = byte_start else {
        return Err(format!("Code import line range {start}-{end} starts past end of file."));
    };
    Ok(&source[byte_start..byte_end])
}

fn select_named_region<'a>(source: &'a str, name: &str) -> Result<&'a str, String> {
    let mut region_start = None;
    let mut cursor = 0usize;
    for line in source.split_inclusive('\n') {
        let trimmed = line.trim();
        if region_start.is_none() && is_region_start(trimmed, name) {
            region_start = Some(cursor + line.len());
        } else if let Some(start) = region_start {
            if is_region_end(trimmed, name) {
                return Ok(&source[start..cursor]);
            }
        }
        cursor += line.len();
    }
    Err(format!("Code import region {name:?} was not found."))
}

fn is_region_start(line: &str, name: &str) -> bool {
    let Some(rest) = line.split("#region").nth(1) else {
        return false;
    };
    rest.trim().trim_end_matches("-->").trim_end_matches("*/").trim() == name
}

fn is_region_end(line: &str, name: &str) -> bool {
    let Some(rest) = line.split("#endregion").nth(1) else {
        return false;
    };
    let rest = rest.trim().trim_end_matches("-->").trim_end_matches("*/").trim();
    rest.is_empty() || rest == name
}

fn language_from_path(path: &Path) -> String {
    match path.extension().and_then(|value| value.to_str()).unwrap_or_default() {
        "mts" | "cts" => "ts".to_string(),
        "mjs" | "cjs" => "js".to_string(),
        "rs" => "rust".to_string(),
        "sh" | "bash" | "zsh" => "bash".to_string(),
        other => other.to_string(),
    }
}
