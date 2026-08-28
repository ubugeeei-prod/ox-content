use std::fs;
use std::path::Path;

use super::{DEFAULT_INDEX_TITLE, DEFAULT_UNTITLED_TITLE};

/// Class the renderer puts on the permalink control it appends to headings.
const HEADING_PERMALINK_CLASS: &str = "header-anchor";

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
    if let Some(title) = frontmatter_title
        && !title.is_empty()
    {
        return title.to_string();
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

/// Reads the title out of the first rendered `<h1>`.
///
/// The heading is HTML, not plain text: inline formatting survives as tags,
/// and `headingPermalinks` appends an `<a class="header-anchor">#</a>` after
/// the text. Both are stripped here so the page keeps its own title instead
/// of falling back to `Untitled`.
fn extract_h1_text(content: &str) -> Option<String> {
    let lower = content.to_ascii_lowercase();
    let h1_start = lower.find("<h1")?;
    let tag_end = lower[h1_start..].find('>')? + h1_start;
    let text_start = tag_end + 1;
    let close = lower[text_start..].find("</h1>")? + text_start;
    let text = heading_text(&content[text_start..close]);

    if text.is_empty() { None } else { Some(text) }
}

/// Reduces rendered heading markup to the text a reader sees.
fn heading_text(inner: &str) -> String {
    let mut text = String::with_capacity(inner.len());
    let mut cursor = 0usize;

    while cursor < inner.len() {
        let Some(open) = inner[cursor..].find('<').map(|rel| cursor + rel) else {
            text.push_str(&inner[cursor..]);
            break;
        };
        text.push_str(&inner[cursor..open]);

        let Some(open_end) = inner[open..].find('>').map(|rel| open + rel + 1) else {
            // An unterminated `<` is literal text in a heading, not a tag.
            text.push_str(&inner[open..]);
            break;
        };

        // A permalink anchor is chrome the renderer appended, so its `#`
        // marker goes with the tags rather than into the title.
        cursor = permalink_anchor_end(inner, open, open_end).unwrap_or(open_end);
    }

    collapse_whitespace(&decode_basic_entities(&text))
}

/// Offset just past `</a>` when the tag opened at `open` is a permalink anchor.
fn permalink_anchor_end(inner: &str, open: usize, open_end: usize) -> Option<usize> {
    let tag = &inner[open..open_end];
    // `<a\b`: `<abbr` is not an anchor.
    if !tag.to_ascii_lowercase().starts_with("<a")
        || tag.as_bytes().get(2).is_some_and(u8::is_ascii_alphanumeric)
    {
        return None;
    }
    if !class_attr_contains(tag, HEADING_PERMALINK_CLASS) {
        return None;
    }

    let rest = inner[open_end..].to_ascii_lowercase();
    rest.find("</a>").map(|rel| open_end + rel + "</a>".len())
}

fn class_attr_contains(tag: &str, class_name: &str) -> bool {
    for (needle, quote) in [("class=\"", '"'), ("class='", '\'')] {
        if let Some(index) = tag.find(needle) {
            let after = &tag[index + needle.len()..];
            if let Some(end) = after.find(quote) {
                return after[..end].split_whitespace().any(|class| class == class_name);
            }
        }
    }
    false
}

fn collapse_whitespace(value: &str) -> String {
    let mut out = String::with_capacity(value.len());
    let mut pending_space = false;
    for ch in value.chars() {
        if ch.is_whitespace() {
            pending_space = true;
            continue;
        }
        if pending_space && !out.is_empty() {
            out.push(' ');
        }
        pending_space = false;
        out.push(ch);
    }
    out
}

/// Undoes the entities the renderer writes, so a title reads as authored.
fn decode_basic_entities(value: &str) -> String {
    let mut out = String::with_capacity(value.len());
    let mut rest = value;
    while let Some(amp) = rest.find('&') {
        out.push_str(&rest[..amp]);
        let entity = &rest[amp..];
        if let Some((ch, len)) = decode_one_entity(entity) {
            out.push(ch);
            rest = &entity[len..];
        } else {
            out.push('&');
            rest = &entity[1..];
        }
    }
    out.push_str(rest);
    out
}

fn decode_one_entity(entity: &str) -> Option<(char, usize)> {
    const NAMED: &[(&str, char)] = &[
        ("&amp;", '&'),
        ("&lt;", '<'),
        ("&gt;", '>'),
        ("&quot;", '"'),
        ("&apos;", '\''),
        ("&#39;", '\''),
    ];
    for (token, ch) in NAMED {
        if entity.starts_with(token) {
            return Some((*ch, token.len()));
        }
    }
    if let Some(rest) = entity.strip_prefix("&#x").or_else(|| entity.strip_prefix("&#X")) {
        let end = rest.find(';')?;
        let ch = char::from_u32(u32::from_str_radix(&rest[..end], 16).ok()?)?;
        return Some((ch, "&#x".len() + end + 1));
    }
    let rest = entity.strip_prefix("&#")?;
    let end = rest.find(';')?;
    let ch = char::from_u32(rest[..end].parse().ok()?)?;
    Some((ch, "&#".len() + end + 1))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn extracts_titles_like_the_ts_helper() {
        assert_eq!(extract_title("<h1>Rendered Title</h1>", None), "Rendered Title");
        assert_eq!(extract_title("<h1><span>Nested</span></h1>", None), "Nested");
        assert_eq!(extract_title("<h1>Ignored</h1>", Some("Frontmatter")), "Frontmatter");
        assert_eq!(format_title("getting_started-now"), "Getting Started Now");
    }

    #[test]
    fn ignores_the_appended_heading_permalink() {
        let html = concat!(
            r##"<h1 id="自動テストについて">自動テストについて"##,
            r##"<a class="header-anchor" href="#自動テストについて" "##,
            r##"aria-label="Permalink to &quot;自動テストについて&quot;">#</a></h1>"##,
        );

        assert_eq!(extract_title(html, None), "自動テストについて");
    }

    #[test]
    fn keeps_inline_formatting_as_text() {
        assert_eq!(
            extract_title("<h1><strong>Bold</strong> and <code>code</code></h1>", None),
            "Bold and code"
        );
    }

    #[test]
    fn keeps_the_text_of_links_the_author_wrote() {
        assert_eq!(
            extract_title(r#"<h1>See <a href="/docs">the docs</a></h1>"#, None),
            "See the docs"
        );
    }

    #[test]
    fn decodes_entities_the_renderer_escaped() {
        assert_eq!(
            extract_title("<h1>Tips &amp; tricks for &lt;script&gt;</h1>", None),
            "Tips & tricks for <script>"
        );
    }

    #[test]
    fn falls_back_when_the_heading_is_only_a_permalink() {
        let html = r##"<h1 id="x"><a class="header-anchor" href="#x">#</a></h1>"##;

        assert_eq!(extract_title(html, None), "Untitled");
    }

    #[test]
    fn collapses_whitespace_around_nested_markup() {
        assert_eq!(extract_title("<h1>\n  Spaced   <em>out</em>\n</h1>", None), "Spaced out");
    }
}
