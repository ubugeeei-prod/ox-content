use std::path::Path;

use super::{
    sanitize_doc_path_segment, MarkdownDocsOptions, MarkdownLinkStyle, MarkdownPathStrategy,
};
use crate::model::{ApiDocMember, ApiDocModule};
use crate::string_builder::{join2, join3, join4, StringBuilder};

pub(super) fn doc_page_href(
    options: &MarkdownDocsOptions,
    file_name: &str,
    anchor: Option<&str>,
) -> String {
    doc_page_href_from(options, "", file_name, anchor)
}

pub(super) fn doc_page_href_from(
    options: &MarkdownDocsOptions,
    current_file_name: &str,
    target_file_name: &str,
    anchor: Option<&str>,
) -> String {
    if target_file_name == current_file_name {
        if let Some(anchor) = anchor.filter(|anchor| !anchor.is_empty()) {
            return join2("#", anchor);
        }
    }

    let mut href = String::new();
    let target_file_name = route_file_name(options, target_file_name);

    if let Some(base_path) =
        options.base_path.as_deref().map(str::trim).filter(|base| !base.is_empty())
    {
        let base_path = normalize_base_path(base_path);
        if !base_path.is_empty() {
            href.push_str(&base_path);
        }
        href.push('/');
        href.push_str(&target_file_name);
    } else {
        href.push_str(&relative_doc_href_path(current_file_name, &target_file_name));
    }

    if options.link_style == MarkdownLinkStyle::Markdown {
        href.push_str(".md");
    }
    if let Some(anchor) = anchor.filter(|anchor| !anchor.is_empty()) {
        href.push('#');
        href.push_str(anchor);
    }

    href
}

fn route_file_name(options: &MarkdownDocsOptions, file_name: &str) -> String {
    if options.link_style == MarkdownLinkStyle::Clean {
        file_name.strip_suffix("/index").unwrap_or(file_name).to_string()
    } else {
        file_name.to_string()
    }
}

fn relative_doc_href_path(current_file_name: &str, target_file_name: &str) -> String {
    let current_dir =
        current_file_name.rsplit_once('/').map_or("", |(directory, _)| directory).trim_matches('/');
    let current_parts = current_dir.split('/').filter(|part| !part.is_empty()).collect::<Vec<_>>();
    let target_parts =
        target_file_name.split('/').filter(|part| !part.is_empty()).collect::<Vec<_>>();

    let mut common = 0;
    while current_parts.get(common) == target_parts.get(common) {
        if current_parts.get(common).is_none() {
            break;
        }
        common += 1;
    }

    let mut parts = Vec::new();
    parts.extend(std::iter::repeat_n("..", current_parts.len().saturating_sub(common)));
    parts.extend(target_parts.iter().skip(common).copied());

    let path = if parts.is_empty() { target_file_name.to_string() } else { parts.join("/") };
    if path.starts_with("../") {
        path
    } else {
        join2("./", &path)
    }
}

fn normalize_base_path(base_path: &str) -> String {
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

pub(super) fn entry_anchor(name: &str) -> String {
    name.to_lowercase()
}

pub(super) fn member_anchor(
    entry_name: &str,
    member: &ApiDocMember,
    path_strategy: MarkdownPathStrategy,
) -> String {
    match path_strategy {
        MarkdownPathStrategy::Flat => {
            join3(&entry_anchor(entry_name), "-", &entry_anchor(&member.name))
        }
        MarkdownPathStrategy::TypeDoc => {
            let prefix = match member.kind.as_str() {
                "constructor" => return "constructor".to_string(),
                "method" => "method",
                "getter" | "setter" => "accessor",
                "enumMember" => "enumeration-member",
                _ => "property",
            };
            join3(prefix, "-", &entry_anchor(&member.name))
        }
    }
}

pub(super) fn module_file_name(file_path: &str) -> String {
    let mut file_name = file_stem(file_path);
    if file_name == "index" {
        file_name = "index-module".to_string();
    }
    sanitize_doc_path_segment(&file_name)
}

pub(super) fn module_route_name(doc: &ApiDocModule) -> String {
    module_file_name(&doc.file)
}

pub(super) fn module_display_name(doc: &ApiDocModule) -> String {
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

fn normalize_doc_file_path(file_path: &str) -> String {
    let normalized = file_path.replace('\\', "/");

    for marker in ["npm/", "packages/", "crates/", "src/"] {
        if let Some(index) = normalized.find(marker) {
            if index == 0 || normalized.as_bytes().get(index - 1) == Some(&b'/') {
                return normalized[index..].to_string();
            }
        }
    }

    normalized.trim_start_matches('/').to_string()
}

pub(super) fn generate_source_href(
    file_path: &str,
    github_url: &str,
    line_number: Option<u32>,
    end_line_number: Option<u32>,
) -> String {
    let relative_path = normalize_doc_file_path(file_path);
    let fragment = if let Some(line_number) = line_number {
        let mut fragment = StringBuilder::with_capacity(24);
        fragment.push_str("#L");
        fragment.push_usize(line_number as usize);
        if let Some(end_line_number) =
            end_line_number.filter(|end_line_number| *end_line_number > line_number)
        {
            fragment.push_str("-L");
            fragment.push_usize(end_line_number as usize);
        }
        fragment.into_string()
    } else {
        String::new()
    };

    join4(github_url, "/blob/main/", &relative_path, &fragment)
}

pub(super) fn generate_source_link(
    file_path: &str,
    github_url: &str,
    line_number: Option<u32>,
    end_line_number: Option<u32>,
) -> String {
    let href = generate_source_href(file_path, github_url, line_number, end_line_number);
    join3("**[Source](", &href, ")**")
}

pub(super) fn file_name(file_path: &str) -> String {
    Path::new(file_path)
        .file_name()
        .and_then(|value| value.to_str())
        .map_or_else(|| file_path.to_string(), ToString::to_string)
}

pub(super) fn file_stem(file_path: &str) -> String {
    Path::new(file_path)
        .file_stem()
        .and_then(|value| value.to_str())
        .map_or_else(|| file_path.to_string(), ToString::to_string)
}

pub(super) fn capitalize_ascii(value: &str) -> String {
    let mut chars = value.chars();
    match chars.next() {
        Some(first) => {
            let rest = chars.as_str();
            let mut out = StringBuilder::with_capacity(first.len_utf8() + rest.len());
            out.push_char(first.to_ascii_uppercase());
            out.push_str(rest);
            out.into_string()
        }
        None => String::new(),
    }
}
