use serde_json::{json, Map, Value};

pub(super) fn normalize_markdown_extensions(extensions: &[String]) -> Vec<String> {
    let source = if extensions.is_empty() {
        vec![".md".to_string(), ".markdown".to_string(), ".mdx".to_string()]
    } else {
        extensions.to_vec()
    };

    source
        .into_iter()
        .map(|extension| {
            let extension =
                if extension.starts_with('.') { extension } else { format!(".{extension}") };
            extension.to_ascii_lowercase()
        })
        .collect()
}

pub(super) fn markdown_extension(file_path: &str, extensions: &[String]) -> Option<String> {
    let lower_path = file_path.to_ascii_lowercase();
    extensions
        .iter()
        .filter(|extension| lower_path.ends_with(extension.as_str()))
        .max_by_key(|extension| extension.len())
        .map(|extension| file_path[file_path.len() - extension.len()..].to_string())
}

pub(super) fn collection_path(relative_path: &str, extensions: &[String]) -> String {
    let stem = strip_markdown_extension(relative_path, extensions);
    let mut segments: Vec<String> =
        stem.split('/').filter(|segment| !segment.is_empty()).map(strip_numeric_prefix).collect();

    if segments.last().is_some_and(|segment| segment == "index") {
        segments.pop();
    }

    if segments.is_empty() {
        "/".to_string()
    } else {
        format!("/{}", segments.join("/"))
    }
}

pub(super) fn string_field(frontmatter: &Map<String, Value>, key: &str) -> Option<String> {
    frontmatter.get(key).and_then(Value::as_str).map(str::to_string)
}

pub(super) fn first_h1_from_toc(toc: Option<&[Value]>) -> Option<String> {
    toc?.iter().find_map(|entry| {
        if entry.get("depth").and_then(Value::as_u64) == Some(1) {
            entry.get("text").and_then(Value::as_str).map(str::to_string)
        } else {
            None
        }
    })
}

pub(super) fn extract_first_heading(content: &str) -> Option<String> {
    for line in content.lines() {
        let trimmed = line.trim_start();
        let depth = trimmed.bytes().take_while(|byte| *byte == b'#').count();
        if !(1..=6).contains(&depth) || trimmed.as_bytes().get(depth) != Some(&b' ') {
            continue;
        }
        let text = trimmed[depth + 1..].trim().trim_end_matches('#').trim();
        if !text.is_empty() {
            return Some(text.to_string());
        }
    }
    None
}

pub(super) fn format_title_from_path(stem: &str) -> String {
    let last = stem.rsplit('/').find(|segment| !segment.is_empty()).unwrap_or(stem);
    let mut result = String::new();
    let mut uppercase_next = true;

    for ch in last.chars() {
        if matches!(ch, '-' | '_') {
            if !result.ends_with(' ') {
                result.push(' ');
            }
            uppercase_next = true;
        } else if uppercase_next && ch.is_ascii_lowercase() {
            result.push(ch.to_ascii_uppercase());
            uppercase_next = false;
        } else {
            result.push(ch);
            uppercase_next = false;
        }
    }

    result.trim().to_string()
}

pub(super) fn toc_entry_to_value(entry: ox_content_transform::TocEntry) -> Value {
    json!({
        "depth": entry.depth,
        "text": entry.text,
        "slug": entry.slug,
        "children": entry.children.into_iter().map(toc_entry_to_value).collect::<Vec<_>>(),
    })
}

fn strip_markdown_extension(file_path: &str, extensions: &[String]) -> String {
    markdown_extension(file_path, extensions).map_or_else(
        || file_path.to_string(),
        |extension| file_path[..file_path.len() - extension.len()].to_string(),
    )
}

fn strip_numeric_prefix(segment: &str) -> String {
    let split = segment.bytes().position(|byte| byte == b'.');
    match split {
        Some(index) if segment[..index].bytes().all(|byte| byte.is_ascii_digit()) => {
            segment[index + 1..].to_string()
        }
        _ => segment.to_string(),
    }
}
