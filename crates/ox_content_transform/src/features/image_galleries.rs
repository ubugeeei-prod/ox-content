//! Opt-in `::: gallery` image groups.
//!
//! Disabled by default. Enabled galleries render static HTML and reuse the
//! Markdown image parser so URL sanitization, captions, dimensions, and attrs
//! stay aligned with the image feature.

use crate::ImageGalleryOptions;

use super::images::{self, ParsedImage, ResolvedImageOptions};
use super::segments::{is_closing_fence, is_indented_code_line, parse_opening_fence};

mod html;
#[cfg(test)]
mod tests;

use html::emit_gallery;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum ReportMode {
    Ignore,
    Warn,
    Error,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) struct ResolvedImageGalleryOptions {
    image: ResolvedImageOptions,
    missing_alt: ReportMode,
    empty: ReportMode,
}

struct GalleryOpen {
    colon_count: usize,
    caption: Option<String>,
}

struct GalleryItem<'a> {
    image: ParsedImage<'a>,
}

pub(super) fn resolve(
    options: Option<&ImageGalleryOptions>,
    attrs: bool,
    images: Option<&ResolvedImageOptions>,
) -> Option<ResolvedImageGalleryOptions> {
    let options = options?;
    if options.enabled == Some(false) {
        return None;
    }
    let lazy = options.lazy.unwrap_or_else(|| images.is_none_or(|image| image.lazy));
    Some(ResolvedImageGalleryOptions {
        image: ResolvedImageOptions { lazy, attrs },
        missing_alt: report_mode(options.missing_alt.as_deref(), ReportMode::Error),
        empty: report_mode(options.empty.as_deref(), ReportMode::Error),
    })
}

pub(super) fn transform(
    source: &str,
    options: &ResolvedImageGalleryOptions,
    errors: &mut Vec<String>,
) -> String {
    if !source.contains(":::") || !source.contains("gallery") {
        return source.to_string();
    }
    rewrite(source, options, errors)
}

fn rewrite(
    source: &str,
    options: &ResolvedImageGalleryOptions,
    errors: &mut Vec<String>,
) -> String {
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
            in_fence = true;
            fence_char = open.fence_char;
            fence_len = open.fence_len;
            out.push_str(line);
            out.push_str(ending);
            continue;
        }

        let Some(open) = parse_gallery_open(line) else {
            out.push_str(line);
            out.push_str(ending);
            continue;
        };

        let mut body = String::new();
        let mut close = None;
        for inner in lines.by_ref() {
            let (inner_line, inner_end) = split_ending(inner);
            if is_gallery_close(inner_line, open.colon_count) {
                close = Some((inner_line, inner_end));
                break;
            }
            body.push_str(inner_line);
            body.push_str(inner_end);
        }

        let mut original = String::with_capacity(line_with_end.len() + body.len() + 8);
        original.push_str(line);
        original.push_str(ending);
        original.push_str(&body);
        if let Some((close_line, close_end)) = close {
            original.push_str(close_line);
            original.push_str(close_end);
        } else {
            errors.push("Image gallery block is missing a closing ::: fence.".to_string());
            out.push_str(&original);
            continue;
        }

        match render_gallery(open.caption.as_deref(), &body, options) {
            Ok((html, diagnostics)) => {
                errors.extend(diagnostics);
                out.push_str(&html);
            }
            Err(diagnostics) => {
                errors.extend(diagnostics);
                out.push_str(&original);
            }
        }
    }

    out
}

fn render_gallery(
    caption: Option<&str>,
    body: &str,
    options: &ResolvedImageGalleryOptions,
) -> Result<(String, Vec<String>), Vec<String>> {
    let mut diagnostics = Vec::new();
    let mut items = Vec::new();

    for (index, line) in body.lines().enumerate() {
        let Some(candidate) = normalize_item_line(line) else {
            continue;
        };
        let Some(image) = images::parse_image(candidate, &options.image) else {
            diagnostics.push(format!("Image gallery item {} must be a Markdown image.", index + 1));
            return Err(diagnostics);
        };
        if !candidate[image.end..].trim().is_empty() {
            diagnostics.push(format!(
                "Image gallery item {} has unsupported trailing content.",
                index + 1
            ));
            return Err(diagnostics);
        }
        if image.alt.trim().is_empty() {
            let message = format!("Image gallery item {} is missing alt text.", index + 1);
            match options.missing_alt {
                ReportMode::Ignore => {}
                ReportMode::Warn => diagnostics.push(message),
                ReportMode::Error => {
                    diagnostics.push(message);
                    return Err(diagnostics);
                }
            }
        }
        items.push(GalleryItem { image });
    }

    if items.is_empty() {
        let diagnostics = match options.empty {
            ReportMode::Ignore => Vec::new(),
            ReportMode::Warn | ReportMode::Error => {
                vec!["Image gallery is empty. Add at least one Markdown image.".to_string()]
            }
        };
        return Err(diagnostics);
    }

    let mut html = String::new();
    emit_gallery(&mut html, caption, &items, &options.image);
    Ok((html, diagnostics))
}

fn normalize_item_line(line: &str) -> Option<&str> {
    let trimmed = line.trim();
    if trimmed.is_empty() {
        return None;
    }
    if let Some(rest) = trimmed.strip_prefix("- ").or_else(|| trimmed.strip_prefix("* ")) {
        return Some(rest.trim_start());
    }
    if let Some(rest) = strip_ordered_marker(trimmed) {
        return Some(rest.trim_start());
    }
    Some(trimmed)
}

fn strip_ordered_marker(value: &str) -> Option<&str> {
    let dot = value.find('.')?;
    if dot == 0 || !value[..dot].bytes().all(|byte| byte.is_ascii_digit()) {
        return None;
    }
    value[dot + 1..].starts_with(char::is_whitespace).then_some(&value[dot + 1..])
}

fn parse_gallery_open(line: &str) -> Option<GalleryOpen> {
    if is_indented_code_line(line) {
        return None;
    }
    let trimmed = line.trim_start();
    let colon_count = trimmed.bytes().take_while(|byte| *byte == b':').count();
    if colon_count < 3 {
        return None;
    }
    let rest = trimmed[colon_count..].trim();
    let mut parts = rest.splitn(2, char::is_whitespace);
    if parts.next()? != "gallery" {
        return None;
    }
    Some(GalleryOpen { colon_count, caption: parse_caption(parts.next().unwrap_or_default()) })
}

fn parse_caption(meta: &str) -> Option<String> {
    let meta = meta.trim();
    if meta.is_empty() {
        return None;
    }
    let mut rest = meta;
    while let Some(eq) = rest.find('=') {
        let key = rest[..eq].trim();
        rest = rest[eq + 1..].trim_start();
        let (value, next) = parse_meta_value(rest);
        if matches!(key, "title" | "caption") && !value.is_empty() {
            return Some(value);
        }
        rest = next.trim_start();
    }
    Some(unquote(meta).to_string()).filter(|value| !value.is_empty())
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

fn unquote(value: &str) -> &str {
    let bytes = value.as_bytes();
    if bytes.len() >= 2
        && matches!(bytes.first(), Some(b'"' | b'\''))
        && bytes.first() == bytes.last()
    {
        &value[1..value.len() - 1]
    } else {
        value
    }
}

fn is_gallery_close(line: &str, colon_count: usize) -> bool {
    let trimmed = line.trim();
    trimmed.len() >= colon_count && trimmed.bytes().all(|byte| byte == b':')
}

fn report_mode(value: Option<&str>, fallback: ReportMode) -> ReportMode {
    match value.map(str::trim) {
        Some("ignore") => ReportMode::Ignore,
        Some("warn") => ReportMode::Warn,
        Some("error") => ReportMode::Error,
        _ => fallback,
    }
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
