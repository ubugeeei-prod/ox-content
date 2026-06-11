use std::path::Path;

use crate::model::ApiDocModule;
use crate::string_builder::{join2, join3};

const OVERVIEW_TITLE: &str = "Overview";

pub(super) fn normalize_base_path(base_path: &str) -> String {
    let base_path = base_path.trim().trim_end_matches('/');

    if base_path.is_empty() || base_path == "/" {
        return String::new();
    }

    if base_path.starts_with('/') {
        base_path.to_string()
    } else {
        join2("/", base_path)
    }
}

pub(super) fn nav_route_path(base_path: &str, file_name: &str) -> String {
    let file_name = file_name.strip_suffix("/index").unwrap_or(file_name);
    if base_path.is_empty() {
        join2("/", file_name)
    } else {
        join3(base_path, "/", file_name)
    }
}

pub(super) fn get_doc_display_name(file_path: &str) -> String {
    let file_name = file_stem(file_path);

    if file_name == "index" || file_name == "index-module" {
        return OVERVIEW_TITLE.to_string();
    }

    format_doc_title(&file_name)
}

pub(super) fn get_doc_file_name(file_path: &str) -> String {
    file_stem(file_path)
}

pub(super) fn typedoc_module_route_name(doc: &ApiDocModule) -> String {
    module_file_name(&doc.file)
}

pub(super) fn typedoc_module_display_name(doc: &ApiDocModule) -> String {
    if !doc.source_path.is_empty() {
        return doc.file.clone();
    }

    let display_name = file_stem(&doc.file);
    if display_name.is_empty() {
        doc.file.clone()
    } else {
        display_name
    }
}

pub(super) fn sanitize_doc_path_segment(value: &str) -> String {
    let sanitized = value
        .chars()
        .map(|ch| match ch {
            '/' | '\\' | '?' | '#' | '[' | ']' | '<' | '>' | ':' | '"' | '|' | '*' => '-',
            _ => ch,
        })
        .collect::<String>();
    if sanitized.is_empty() {
        "symbol".to_string()
    } else {
        sanitized
    }
}

fn module_file_name(file_path: &str) -> String {
    let mut file_name = file_stem(file_path);
    if file_name == "index" {
        file_name = "index-module".to_string();
    }
    sanitize_doc_path_segment(&file_name)
}

fn file_stem(file_path: &str) -> String {
    Path::new(file_path).file_stem().and_then(|stem| stem.to_str()).unwrap_or_default().to_string()
}

fn format_doc_title(name: &str) -> String {
    let mut chars = name.chars().peekable();
    let mut result = String::new();

    while let Some(ch) = chars.next() {
        if matches!(ch, '-' | '_') {
            match chars.peek().copied() {
                Some(next) if next.is_ascii_lowercase() => {
                    result.push(' ');
                    result.push(next.to_ascii_uppercase());
                    chars.next();
                }
                _ => result.push(ch),
            }
        } else {
            result.push(ch);
        }
    }

    if let Some(first) = result.chars().next().filter(char::is_ascii_lowercase) {
        let uppercase = first.to_ascii_uppercase().to_string();
        result.replace_range(0..first.len_utf8(), &uppercase);
    }

    result
}
