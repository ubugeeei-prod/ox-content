//! Opt-in `<NotByAI />` authorship badge.
//!
//! Disabled by default. When enabled, self-closing `<NotByAI />` / `<NotByAI/>`
//! becomes a static link wrapping the official light and dark Not By AI SVGs.

use std::sync::OnceLock;

use crate::NotByAiOptions;

#[cfg(test)]
mod tests;

pub(super) const DEFAULT_LABEL: &str = "Written by human, not by AI";
pub(super) const DEFAULT_HREF: &str = "https://notbyai.fyi";

const OPEN: &str = "<NotByAI";
const LIGHT_SVG: &str = include_str!("not_by_ai/badge-light.svg");
const DARK_SVG: &str = include_str!("not_by_ai/badge-dark.svg");

#[derive(Clone)]
pub(super) struct ResolvedNotByAi {
    label: String,
    href: String,
}

pub(super) fn resolve(options: Option<&NotByAiOptions>) -> Option<ResolvedNotByAi> {
    let options = options?;
    if options.enabled == Some(false) {
        return None;
    }
    Some(ResolvedNotByAi {
        label: normalize_label(options.label.as_deref()),
        href: normalize_href(options.href.as_deref()),
    })
}

pub(super) fn apply(current: &mut std::borrow::Cow<'_, str>, options: Option<&ResolvedNotByAi>) {
    let Some(options) = options else {
        return;
    };
    if !current.contains(OPEN) {
        return;
    }
    if let Some(replaced) = super::transform_markdown_text_segments(current, |segment, out| {
        replace(segment, options, out);
    }) {
        *current = std::borrow::Cow::Owned(replaced);
    }
}

pub(super) fn replace(segment: &str, options: &ResolvedNotByAi, out: &mut String) {
    let mut cursor = 0usize;
    while let Some(relative) = segment[cursor..].find(OPEN) {
        let start = cursor + relative;
        out.push_str(&segment[cursor..start]);
        if let Some(end) = parse_self_closing(segment, start) {
            emit(options, out);
            cursor = end;
        } else {
            out.push_str(OPEN);
            cursor = start + OPEN.len();
        }
    }
    out.push_str(&segment[cursor..]);
}

fn parse_self_closing(segment: &str, start: usize) -> Option<usize> {
    let after_name = start + OPEN.len();
    let rest = segment.get(after_name..)?;
    let whitespace = rest.bytes().take_while(u8::is_ascii_whitespace).count();
    let closer = rest.get(whitespace..)?;
    closer.starts_with("/>").then_some(after_name + whitespace + 2)
}

fn emit(options: &ResolvedNotByAi, out: &mut String) {
    out.push_str("<a class=\"ox-not-by-ai\" href=\"");
    super::escape_html_attr(&options.href, out);
    out.push_str("\" aria-label=\"");
    super::escape_html_attr(&options.label, out);
    out.push_str("\" target=\"_blank\" rel=\"noopener noreferrer\">");
    out.push_str(light_badge());
    out.push_str(dark_badge());
    out.push_str("</a>");
}

fn normalize_label(label: Option<&str>) -> String {
    label.map(str::trim).filter(|value| !value.is_empty()).unwrap_or(DEFAULT_LABEL).to_string()
}

fn normalize_href(href: Option<&str>) -> String {
    href.map(str::trim).filter(|value| is_safe_href(value)).unwrap_or(DEFAULT_HREF).to_string()
}

fn is_safe_href(href: &str) -> bool {
    if href.is_empty()
        || href.bytes().any(|byte| {
            byte.is_ascii_control()
                || byte.is_ascii_whitespace()
                || matches!(byte, b'"' | b'\'' | b'<' | b'>' | b'`' | b'\\')
        })
    {
        return false;
    }
    if href.starts_with('/') && !href.starts_with("//") {
        return true;
    }
    let Some((scheme, rest)) = href.split_once("://") else {
        return false;
    };
    if !scheme.eq_ignore_ascii_case("http") && !scheme.eq_ignore_ascii_case("https") {
        return false;
    }
    let host = rest.split(['/', '?', '#']).next().unwrap_or("");
    !host.is_empty()
}

fn light_badge() -> &'static str {
    static SVG: OnceLock<String> = OnceLock::new();
    SVG.get_or_init(|| badge_markup(LIGHT_SVG, "light")).as_str()
}

fn dark_badge() -> &'static str {
    static SVG: OnceLock<String> = OnceLock::new();
    SVG.get_or_init(|| badge_markup(DARK_SVG, "dark")).as_str()
}

fn badge_markup(source: &str, variant: &str) -> String {
    let sanitized = sanitize_vendored_svg(source);
    if sanitized.is_empty() {
        return String::new();
    }
    sanitized.replacen(
        "<svg ",
        &format!(
            "<svg class=\"ox-not-by-ai__badge ox-not-by-ai__badge--{variant}\" aria-hidden=\"true\" "
        ),
        1,
    )
}

fn sanitize_vendored_svg(source: &str) -> String {
    let collapsed: String = source
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty() && !line.starts_with("<!--"))
        .collect();
    let lower = collapsed.to_ascii_lowercase();
    if !collapsed.starts_with("<svg")
        || !collapsed.ends_with("</svg>")
        || lower.contains("<script")
        || lower.contains("javascript:")
        || lower.contains("<foreignobject")
        || lower.contains("<use")
        || lower.contains("<image")
        || lower.contains("xlink:href")
        || ["onclick=", "onerror=", "onload=", "onmouseover=", "onfocus="]
            .iter()
            .any(|needle| lower.contains(needle))
    {
        return String::new();
    }
    collapsed
}
