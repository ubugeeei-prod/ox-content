use std::fs;
use std::path::Path;

use super::{DEFAULT_INDEX_TITLE, DEFAULT_UNTITLED_TITLE};

/// Collects Markdown files under `src_dir`, skipping common generated directories.
pub fn collect_markdown_files(src_dir: &str, extensions: &[String]) -> Vec<String> {
    let extensions = normalize_markdown_extensions(extensions);
    let mut files = Vec::new();
    collect_markdown_files_inner(Path::new(src_dir), &extensions, &mut files);
    files.sort();
    files
}

/// Extracts a display title from frontmatter or a rendered `<h1>`.
pub fn extract_title(content: &str, frontmatter_title: Option<&str>) -> String {
    if let Some(title) = frontmatter_title {
        if !title.is_empty() {
            return title.to_string();
        }
    }

    if let Some(title) = extract_h1_text(content) {
        return title;
    }

    DEFAULT_UNTITLED_TITLE.to_string()
}

/// Formats a file or directory name as a display title.
pub fn format_title(name: &str) -> String {
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

pub(super) fn get_display_title(file_path: &str) -> String {
    let path = Path::new(file_path);
    let file_name = path.file_stem().and_then(|name| name.to_str()).unwrap_or_default();

    if file_name == "index" {
        let dir_name = path.parent().and_then(Path::file_name).and_then(|name| name.to_str());
        return dir_name
            .filter(|name| !name.is_empty() && *name != ".")
            .map_or_else(|| DEFAULT_INDEX_TITLE.to_string(), format_title);
    }

    format_title(file_name)
}

fn collect_markdown_files_inner(dir: &Path, extensions: &[String], files: &mut Vec<String>) {
    let Ok(entries) = fs::read_dir(dir) else {
        return;
    };

    for entry in entries.flatten() {
        let path = entry.path();
        let Ok(file_type) = entry.file_type() else {
            continue;
        };

        if file_type.is_dir() {
            let name = entry.file_name();
            let name = name.to_string_lossy();
            if matches!(name.as_ref(), "node_modules" | "dist" | ".git") {
                continue;
            }
            collect_markdown_files_inner(&path, extensions, files);
        } else if file_type.is_file() && is_markdown_file(&path, extensions) {
            files.push(path.to_string_lossy().into_owned());
        }
    }
}

fn normalize_markdown_extensions(extensions: &[String]) -> Vec<String> {
    let source: Vec<String> = if extensions.is_empty() {
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

fn is_markdown_file(path: &Path, extensions: &[String]) -> bool {
    let path = path.to_string_lossy().to_ascii_lowercase();
    extensions.iter().any(|extension| path.ends_with(extension))
}

fn extract_h1_text(content: &str) -> Option<String> {
    let lower = content.to_ascii_lowercase();
    let h1_start = lower.find("<h1")?;
    let tag_end = lower[h1_start..].find('>')? + h1_start;
    let text_start = tag_end + 1;
    let close = lower[text_start..].find("</h1>")? + text_start;
    let text = content[text_start..close].trim();

    if text.is_empty() || text.contains('<') {
        None
    } else {
        Some(text.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn extracts_titles_like_the_ts_helper() {
        assert_eq!(extract_title("<h1>Rendered Title</h1>", None), "Rendered Title");
        assert_eq!(extract_title("<h1><span>Nested</span></h1>", None), "Untitled");
        assert_eq!(extract_title("<h1>Ignored</h1>", Some("Frontmatter")), "Frontmatter");
        assert_eq!(format_title("getting_started-now"), "Getting Started Now");
    }
}
