//! Opt-in figures, captions, and lazy-loaded images.
//!
//! Disabled by default. Markdown image rendering is unchanged until this
//! option is enabled.

use crate::ImageOptions;

use super::attr_tokens::{ParsedAttrs, write_attrs_except};
use super::{escape_html_attr, escape_html_text};

#[cfg(test)]
mod tests;

#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) struct ResolvedImageOptions {
    pub(super) lazy: bool,
    pub(super) attrs: bool,
}

pub(super) fn resolve(options: Option<&ImageOptions>, attrs: bool) -> Option<ResolvedImageOptions> {
    let options = options?;
    if options.enabled == Some(false) {
        return None;
    }
    Some(ResolvedImageOptions { lazy: options.lazy != Some(false), attrs })
}

pub(super) fn preprocess(source: &str, options: &ResolvedImageOptions) -> Option<String> {
    if !source.contains("![") {
        return None;
    }
    super::segments::transform_markdown_text_segments(source, |segment, out| {
        replace_images(segment, options, out);
    })
}

#[cfg(test)]
pub(super) fn transform(source: &str, options: &ResolvedImageOptions) -> String {
    let mut out = String::with_capacity(source.len() + 64);
    replace_images(source, options, &mut out);
    out
}

pub(super) fn replace_images(segment: &str, options: &ResolvedImageOptions, out: &mut String) {
    if is_indented_code_segment(segment) {
        out.push_str(segment);
        return;
    }

    // A `![` with no `]` after it cannot be an image, and `split_balanced`
    // only reports that after walking to the end of the segment — once per
    // `![`, which is quadratic over a run of them: 64 KiB of `![ ` took
    // 0.49 s and grew x16 for every x4 of input. The last `]` settles it for
    // every `![` in the segment at once.
    let last_close = memchr::memrchr(b']', segment.as_bytes());

    let mut cursor = 0usize;
    while let Some(relative) = segment[cursor..].find("![") {
        let start = cursor + relative;
        out.push_str(&segment[cursor..start]);
        if last_close.is_some_and(|last| last > start + 1)
            && let Some(parsed) = parse_image(&segment[start..], options)
        {
            emit_image(out, options, &parsed);
            cursor = start + parsed.end;
        } else {
            out.push_str("![");
            cursor = start + 2;
        }
    }
    out.push_str(&segment[cursor..]);
}

pub(super) struct ParsedImage<'a> {
    pub(super) alt: &'a str,
    pub(super) src: Option<&'a str>,
    pub(super) caption: Option<&'a str>,
    pub(super) attrs: Option<ParsedAttrs>,
    pub(super) width: Option<String>,
    pub(super) height: Option<String>,
    pub(super) end: usize,
}

#[derive(Default)]
struct ParsedImageAttrs {
    attrs: Option<ParsedAttrs>,
    width: Option<String>,
    height: Option<String>,
    consumed: usize,
}

fn is_indented_code_segment(segment: &str) -> bool {
    segment.starts_with('\t') || segment.starts_with("    ")
}

pub(super) fn parse_image<'a>(
    input: &'a str,
    options: &ResolvedImageOptions,
) -> Option<ParsedImage<'a>> {
    let rest = input.strip_prefix("![")?;
    let (alt, after_alt) = split_balanced(rest, b'[', b']')?;
    let after_alt = after_alt.strip_prefix('(')?;
    let (src_raw, after_src) = parse_destination(after_alt)?;
    let (caption, after_title) = parse_optional_title(after_src);
    let after_close = after_title.trim_start().strip_prefix(')')?;
    let trailing_attrs = parse_trailing_image_attrs(after_close, options.attrs).unwrap_or_default();
    let src = sanitize_src(src_raw);
    Some(ParsedImage {
        alt,
        src,
        caption,
        attrs: trailing_attrs.attrs,
        width: trailing_attrs.width,
        height: trailing_attrs.height,
        end: input.len() - after_close.len() + trailing_attrs.consumed,
    })
}

fn split_balanced(input: &str, open: u8, close: u8) -> Option<(&str, &str)> {
    let bytes = input.as_bytes();
    let mut depth = 1usize;
    let mut i = 0usize;
    while i < bytes.len() {
        match bytes[i] {
            b'\\' if i + 1 < bytes.len() => i += 2,
            b if b == open => {
                depth += 1;
                i += 1;
            }
            b if b == close => {
                depth -= 1;
                if depth == 0 {
                    return Some((&input[..i], &input[i + 1..]));
                }
                i += 1;
            }
            _ => i += 1,
        }
    }
    None
}

fn parse_destination(input: &str) -> Option<(&str, &str)> {
    let input = input.trim_start();
    if let Some(inner) = input.strip_prefix('<') {
        let close = inner.find('>')?;
        return Some((inner[..close].trim(), &inner[close + 1..]));
    }
    let bytes = input.as_bytes();
    let mut depth = 0isize;
    let mut i = 0usize;
    while i < bytes.len() {
        match bytes[i] {
            b'\\' if i + 1 < bytes.len() => i += 2,
            b'(' => {
                depth += 1;
                i += 1;
            }
            b')' if depth == 0 => break,
            b')' => {
                depth -= 1;
                i += 1;
            }
            b if b.is_ascii_whitespace() && depth == 0 => break,
            _ => i += 1,
        }
    }
    Some((input[..i].trim(), &input[i..]))
}

fn parse_optional_title(input: &str) -> (Option<&str>, &str) {
    let trimmed = input.trim_start();
    let bytes = trimmed.as_bytes();
    let Some(&quote) = bytes.first() else {
        return (None, input);
    };
    if quote != b'"' && quote != b'\'' {
        return (None, input);
    }
    let mut i = 1usize;
    while i < bytes.len() {
        if bytes[i] == b'\\' && i + 1 < bytes.len() {
            i += 2;
            continue;
        }
        if bytes[i] == quote {
            return (Some(&trimmed[1..i]), &trimmed[i + 1..]);
        }
        i += 1;
    }
    (None, input)
}

fn parse_trailing_image_attrs(input: &str, preserve_attrs: bool) -> Option<ParsedImageAttrs> {
    let trimmed = input.trim_start();
    let inner = trimmed.strip_prefix('{')?;
    let close = inner.find('}')?;
    let body = inner[..close].trim();
    let consumed = input.len() - inner.len() + close + 1;
    if !preserve_attrs {
        let (width, height) = parse_dimensions_only(body)?;
        return Some(ParsedImageAttrs { attrs: None, width, height, consumed });
    }
    let attrs = ParsedAttrs::parse(body)?;
    let width = parse_dimension_attr(&attrs, "width").ok()?;
    let height = parse_dimension_attr(&attrs, "height").ok()?;
    Some(ParsedImageAttrs { attrs: Some(attrs), width, height, consumed })
}

fn parse_dimensions_only(body: &str) -> Option<(Option<String>, Option<String>)> {
    let mut width = None;
    let mut height = None;
    if body.is_empty() {
        return None;
    }
    for token in body.split_whitespace() {
        let (name, raw_value) = token.split_once('=')?;
        let value = unsigned_int_value(raw_value)?.to_string();
        match name {
            "width" if width.is_none() => width = Some(value),
            "height" if height.is_none() => height = Some(value),
            _ => return None,
        }
    }
    if width.is_none() && height.is_none() { None } else { Some((width, height)) }
}

fn parse_dimension_attr(attrs: &ParsedAttrs, name: &str) -> Result<Option<String>, ()> {
    let Some(value) = attrs.attr_value(name) else {
        return Ok(None);
    };
    unsigned_int_value(value).map(|value| Some(value.to_string())).ok_or(())
}

fn unsigned_int_value(raw: &str) -> Option<&str> {
    let value = if (raw.starts_with('"') && raw.ends_with('"') && raw.len() >= 2)
        || (raw.starts_with('\'') && raw.ends_with('\'') && raw.len() >= 2)
    {
        &raw[1..raw.len() - 1]
    } else {
        raw
    };
    if !value.is_empty() && value.bytes().all(|byte| byte.is_ascii_digit()) {
        Some(value)
    } else {
        None
    }
}

fn sanitize_src(src: &str) -> Option<&str> {
    let trimmed = src.trim();
    if trimmed.is_empty() || is_dangerous_src(trimmed) { None } else { Some(trimmed) }
}

fn is_dangerous_src(src: &str) -> bool {
    let compact: String = src
        .chars()
        .filter(|ch| !ch.is_ascii_whitespace())
        .map(|ch| ch.to_ascii_lowercase())
        .collect();
    compact.starts_with("//")
        || compact.starts_with("javascript:")
        || compact.starts_with("data:")
        || compact.starts_with("vbscript:")
}

fn emit_image(out: &mut String, options: &ResolvedImageOptions, image: &ParsedImage<'_>) {
    if image.caption.is_some() {
        out.push_str("<figure class=\"ox-figure\">");
    }
    out.push_str("<img");
    if let Some(src) = image.src {
        out.push_str(" src=\"");
        escape_html_attr(src, out);
        out.push('"');
    }
    out.push_str(" alt=\"");
    escape_html_attr(image.alt, out);
    out.push('"');
    if let Some(attrs) = &image.attrs {
        write_attrs_except(out, attrs, &["src", "alt", "loading", "width", "height"]);
    }
    if options.lazy {
        out.push_str(" loading=\"lazy\"");
    }
    if let Some(width) = image.width.as_deref() {
        out.push_str(" width=\"");
        out.push_str(width);
        out.push('"');
    }
    if let Some(height) = image.height.as_deref() {
        out.push_str(" height=\"");
        out.push_str(height);
        out.push('"');
    }
    out.push('>');
    if let Some(caption) = image.caption {
        out.push_str("<figcaption>");
        let caption = unescape_markdown(caption);
        escape_html_text(&caption, out);
        out.push_str("</figcaption></figure>");
    }
}

pub(super) fn unescape_markdown(value: &str) -> String {
    let mut out = String::with_capacity(value.len());
    let mut chars = value.chars();
    while let Some(ch) = chars.next() {
        if ch == '\\' {
            if let Some(next) = chars.next() {
                out.push(next);
            }
        } else {
            out.push(ch);
        }
    }
    out
}
