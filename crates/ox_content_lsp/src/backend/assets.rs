//! Asset and link path completion for Markdown.
//!
//! Triggered when the cursor sits inside the `()` of a Markdown link
//! (`[text](`), image (`![alt](`), or a raw HTML `src=`/`href=`
//! attribute. The completion is a list of filesystem entries — files
//! and directories — that live under the partial path written so far,
//! anchored to the document's own directory (or the workspace `src_dir`
//! when the partial path starts with `/`).
//!
//! Image-context triggers (`!` opener or HTML `<img src=`) narrow the
//! file list to known asset extensions. Plain link contexts return
//! every entry so the user can link to Markdown, JSON, etc.
//!
//! The detection is intentionally textual rather than AST-based: an
//! `![alt](./foo` line is still mid-edit when the user is asking for
//! completion, so the parser would refuse it.

use std::fs;
use std::path::{Path, PathBuf};

use tower_lsp::lsp_types::{CompletionItem, CompletionItemKind};

const IMAGE_EXTENSIONS: &[&str] =
    &["png", "jpg", "jpeg", "gif", "webp", "svg", "avif", "ico", "bmp", "tiff"];

const VIDEO_EXTENSIONS: &[&str] = &["mp4", "webm", "ogv", "mov"];

const AUDIO_EXTENSIONS: &[&str] = &["mp3", "ogg", "wav", "flac"];

/// Context the trigger detector returned. `None` means the cursor is
/// not inside a link/image opener and completion should fall through.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AssetContext {
    /// Cursor is inside the `()` of an image link (`![…](…|)`) or a
    /// raw HTML `<img src="…|">`. We narrow to media extensions.
    Image,
    /// Cursor is inside the `()` of a plain link or an HTML `href=`.
    /// Every file (and directory) qualifies.
    Link,
}

/// Try to extract the asset context the cursor sits inside, given the
/// portion of the line *before* the cursor. Returns the context plus
/// the partial path already typed (used to choose the directory to
/// list and the prefix to filter by).
#[must_use]
pub fn detect_context(line_prefix: &str) -> Option<(AssetContext, &str)> {
    if let Some((context, partial)) = detect_markdown_opener(line_prefix) {
        return Some((context, partial));
    }
    detect_html_attribute(line_prefix)
}

fn detect_markdown_opener(line_prefix: &str) -> Option<(AssetContext, &str)> {
    // Walk backwards from the cursor and find the last unmatched `(`
    // that is immediately preceded by `]`. Anything past it is the
    // partial path being typed.
    let bytes = line_prefix.as_bytes();
    let mut depth: i32 = 0;
    let mut i = bytes.len();
    while i > 0 {
        i -= 1;
        match bytes[i] {
            b')' => depth += 1,
            b'(' if depth == 0 => {
                if i == 0 || bytes[i - 1] != b']' {
                    return None;
                }
                let partial = &line_prefix[i + 1..];
                // Reject if the partial already contains a space (URLs
                // can't have unescaped spaces in `()`).
                if partial.contains(' ') {
                    return None;
                }
                // The `]` at `i-1` belongs to `[link text]`. Walk back
                // to its matching `[` so we can check whether the
                // construct is a link (`[…]`) or an image (`![…]`).
                let bracket_start = match_opening_bracket(bytes, i - 1)?;
                let context = if bracket_start > 0 && bytes[bracket_start - 1] == b'!' {
                    AssetContext::Image
                } else {
                    AssetContext::Link
                };
                return Some((context, partial));
            }
            b'(' => depth -= 1,
            _ => {}
        }
    }
    None
}

/// Walk backwards from a `]` and return the offset of the matching
/// `[`. Returns `None` when the brackets are unbalanced (in which case
/// the construct isn't a Markdown link/image and completion should
/// fall through).
fn match_opening_bracket(bytes: &[u8], close: usize) -> Option<usize> {
    debug_assert_eq!(bytes[close], b']');
    let mut depth: i32 = 0;
    let mut i = close;
    while i > 0 {
        i -= 1;
        match bytes[i] {
            b']' => depth += 1,
            b'[' if depth == 0 => return Some(i),
            b'[' => depth -= 1,
            _ => {}
        }
    }
    None
}

fn detect_html_attribute(line_prefix: &str) -> Option<(AssetContext, &str)> {
    // Look back for the last `src="` or `href="` (also handles
    // single-quoted variants). The attribute value must not contain a
    // closing quote between the opener and the cursor.
    for opener in ["src=\"", "src='", "href=\"", "href='"] {
        if let Some(start) = line_prefix.rfind(opener) {
            let value_start = start + opener.len();
            let value = &line_prefix[value_start..];
            let quote = &opener[opener.len() - 1..];
            if value.contains(quote) {
                continue;
            }
            let context = if opener.starts_with("src") {
                detect_html_image_context(&line_prefix[..start])
            } else {
                AssetContext::Link
            };
            return Some((context, value));
        }
    }
    None
}

fn detect_html_image_context(prefix: &str) -> AssetContext {
    // The src attribute can appear on <img>, <video>, <audio>, <source>,
    // <embed>, etc. We assume Image context whenever the nearest open
    // tag has one of those names — otherwise downgrade to Link.
    let Some(tag_start) = prefix.rfind('<') else {
        return AssetContext::Link;
    };
    let after_lt = &prefix[tag_start + 1..];
    let tag_name = after_lt
        .split(|ch: char| ch.is_ascii_whitespace() || ch == '>' || ch == '/')
        .next()
        .unwrap_or("")
        .to_ascii_lowercase();
    match tag_name.as_str() {
        "img" | "video" | "audio" | "source" | "embed" | "iframe" | "picture" => {
            AssetContext::Image
        }
        _ => AssetContext::Link,
    }
}

/// Build completion items for the given context and partial path.
/// `doc_dir` is the document's directory; `src_dir` is the workspace
/// root used to resolve paths that start with `/`.
#[must_use]
pub fn completion_items(
    context: AssetContext,
    partial: &str,
    doc_dir: Option<&Path>,
    src_dir: Option<&Path>,
) -> Vec<CompletionItem> {
    let (base, prefix) = split_partial(partial);
    let Some(directory) = resolve_directory(base, doc_dir, src_dir) else {
        return Vec::new();
    };

    let mut items = Vec::new();
    let Ok(entries) = fs::read_dir(&directory) else {
        return items;
    };
    for entry in entries.flatten() {
        let name = entry.file_name().to_string_lossy().to_string();
        if name.starts_with('.') {
            // Hidden files / dotdirs are noise in editor completion;
            // skip them to keep the list tight.
            continue;
        }
        if !name.starts_with(prefix) {
            continue;
        }
        let file_type = entry.file_type().ok();
        let is_dir = file_type.as_ref().is_some_and(std::fs::FileType::is_dir);
        if !is_dir && !file_matches_context(context, &name) {
            continue;
        }
        items.push(CompletionItem {
            label: if is_dir { format!("{name}/") } else { name.clone() },
            kind: Some(if is_dir { CompletionItemKind::FOLDER } else { CompletionItemKind::FILE }),
            insert_text: Some(if is_dir { format!("{name}/") } else { name.clone() }),
            detail: Some(detail_for(context, is_dir, &name)),
            ..Default::default()
        });
    }
    items.sort_by(|a, b| a.label.cmp(&b.label));
    items
}

fn split_partial(partial: &str) -> (&str, &str) {
    partial.rsplit_once('/').map_or(("", partial), |(base, prefix)| (base, prefix))
}

fn resolve_directory(
    base: &str,
    doc_dir: Option<&Path>,
    src_dir: Option<&Path>,
) -> Option<PathBuf> {
    if base.starts_with('/') {
        let stripped = base.trim_start_matches('/');
        return src_dir.or(doc_dir).map(|root| root.join(stripped));
    }
    let doc = doc_dir?;
    Some(if base.is_empty() { doc.to_path_buf() } else { doc.join(base) })
}

fn file_matches_context(context: AssetContext, file_name: &str) -> bool {
    match context {
        AssetContext::Link => true,
        AssetContext::Image => {
            let Some((_, ext)) = file_name.rsplit_once('.') else {
                return false;
            };
            let lower = ext.to_ascii_lowercase();
            IMAGE_EXTENSIONS.contains(&lower.as_str())
                || VIDEO_EXTENSIONS.contains(&lower.as_str())
                || AUDIO_EXTENSIONS.contains(&lower.as_str())
        }
    }
}

fn detail_for(context: AssetContext, is_dir: bool, name: &str) -> String {
    if is_dir {
        return "directory".into();
    }
    match context {
        AssetContext::Image => format!("asset: {name}"),
        AssetContext::Link => format!("file: {name}"),
    }
}

/// Slice the document line up to the cursor column, regardless of UTF-16
/// vs byte indexing. The caller (`features.rs`) already has the
/// `TextDocumentState` so this is a thin wrapper to keep the call site
/// tidy.
#[must_use]
pub fn line_prefix(line_text: &str, position_character: u32) -> &str {
    let mut utf16 = 0usize;
    for (idx, ch) in line_text.char_indices() {
        if utf16 == position_character as usize {
            return &line_text[..idx];
        }
        utf16 += ch.len_utf16();
        if utf16 > position_character as usize {
            return &line_text[..idx];
        }
    }
    line_text
}

#[cfg(test)]
mod tests;
