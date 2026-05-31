use std::collections::HashSet;

use crate::JsSanitizeOptions;

pub fn sanitize_html(html: &str, options: Option<&JsSanitizeOptions>) -> String {
    let Some(options) = options else {
        return sanitize_html_with_config(html, &SanitizeConfig::default());
    };
    if options.enabled == Some(false) {
        return html.to_string();
    }
    sanitize_html_with_config(html, &SanitizeConfig::from_js(options))
}

struct SanitizeConfig {
    tags: HashSet<String>,
    attributes: HashSet<String>,
    url_schemes: HashSet<String>,
}

impl Default for SanitizeConfig {
    fn default() -> Self {
        Self {
            tags: [
                "a",
                "blockquote",
                "br",
                "code",
                "del",
                "details",
                "div",
                "em",
                "h1",
                "h2",
                "h3",
                "h4",
                "h5",
                "h6",
                "hr",
                "iframe",
                "img",
                "input",
                "li",
                "nav",
                "ol",
                "p",
                "pre",
                "span",
                "strong",
                "summary",
                "sup",
                "table",
                "tbody",
                "td",
                "th",
                "thead",
                "tr",
                "ul",
            ]
            .into_iter()
            .map(ToString::to_string)
            .collect(),
            attributes: [
                "allow",
                "allowfullscreen",
                "alt",
                "aria-label",
                "checked",
                "class",
                "data-code-title",
                "data-group",
                "data-line",
                "data-line-number",
                "data-line-number-start",
                "data-line-numbers",
                "data-ox-tab-group",
                "disabled",
                "height",
                "href",
                "id",
                "loading",
                "name",
                "referrerpolicy",
                "rel",
                "sandbox",
                "src",
                "style",
                "target",
                "title",
                "type",
                "width",
            ]
            .into_iter()
            .map(ToString::to_string)
            .collect(),
            url_schemes: ["http", "https", "mailto", "tel"]
                .into_iter()
                .map(ToString::to_string)
                .collect(),
        }
    }
}

impl SanitizeConfig {
    fn from_js(options: &JsSanitizeOptions) -> Self {
        let mut config = Self::default();
        if let Some(tags) = &options.allowed_tags {
            config.tags = tags.iter().map(|value| value.to_ascii_lowercase()).collect();
        }
        if let Some(attrs) = &options.allowed_attributes {
            config.attributes = attrs.iter().map(|value| value.to_ascii_lowercase()).collect();
        }
        if let Some(schemes) = &options.allowed_url_schemes {
            config.url_schemes = schemes.iter().map(|value| value.to_ascii_lowercase()).collect();
        }
        config
    }

    fn allows_tag(&self, tag: &str) -> bool {
        self.tags.contains(tag)
    }

    fn allows_attr(&self, attr: &str) -> bool {
        if attr.starts_with("data-") || attr.starts_with("aria-") {
            return self.attributes.contains("data-*")
                || self.attributes.contains("aria-*")
                || self.attributes.contains(attr);
        }
        self.attributes.contains(attr)
    }

    fn allows_url(&self, value: &str) -> bool {
        let trimmed = value.trim_matches(|ch: char| ch.is_ascii_control() || ch.is_whitespace());
        if trimmed.is_empty()
            || trimmed.starts_with('/')
            || trimmed.starts_with("./")
            || trimmed.starts_with("../")
            || trimmed.starts_with('#')
        {
            return true;
        }
        let Some(colon) = trimmed.find(':') else {
            return true;
        };
        let first_path_marker = trimmed.find(&['/', '?', '#'][..]).unwrap_or(usize::MAX);
        if first_path_marker < colon {
            return true;
        }
        let scheme = trimmed[..colon]
            .chars()
            .filter(|ch| !ch.is_ascii_whitespace())
            .flat_map(char::to_lowercase)
            .collect::<String>();
        self.url_schemes.contains(&scheme)
    }
}

fn sanitize_html_with_config(html: &str, config: &SanitizeConfig) -> String {
    if !html.contains('<') {
        return html.to_string();
    }

    let bytes = html.as_bytes();
    let mut out = String::with_capacity(html.len());
    let mut cursor = 0usize;

    while cursor < bytes.len() {
        let Some(relative) = memchr::memchr(b'<', &bytes[cursor..]) else {
            break;
        };
        let tag_start = cursor + relative;
        out.push_str(&html[cursor..tag_start]);

        if html[tag_start..].starts_with("<!--") {
            if let Some(end) = html[tag_start + 4..].find("-->") {
                cursor = tag_start + 4 + end + 3;
            } else {
                cursor = bytes.len();
            }
            continue;
        }

        let Some(tag_end) = scan_tag_end(html, tag_start) else {
            escape_html_text(&html[tag_start..], &mut out);
            cursor = bytes.len();
            continue;
        };

        let tag = &html[tag_start + 1..tag_end - 1];
        if tag.starts_with('!') || tag.starts_with('?') {
            cursor = tag_end;
            continue;
        }

        let parsed = ParsedTag::parse(tag);
        let Some(parsed) = parsed else {
            escape_html_text(&html[tag_start..tag_end], &mut out);
            cursor = tag_end;
            continue;
        };

        if !config.allows_tag(&parsed.name) {
            if matches!(parsed.name.as_str(), "script" | "style") && !parsed.closing {
                let close = format!("</{}>", parsed.name);
                if let Some(end) = find_ci(html, tag_end, &close) {
                    cursor = end + close.len();
                } else {
                    cursor = tag_end;
                }
            } else {
                cursor = tag_end;
            }
            continue;
        }

        if parsed.closing {
            out.push_str("</");
            out.push_str(&parsed.name);
            out.push('>');
            cursor = tag_end;
            continue;
        }

        out.push('<');
        out.push_str(&parsed.name);
        for attr in parsed.attrs {
            write_sanitized_attr(&mut out, &attr, config);
        }
        if parsed.self_closing {
            out.push_str(" />");
        } else {
            out.push('>');
        }
        cursor = tag_end;
    }

    if cursor < bytes.len() {
        out.push_str(&html[cursor..]);
    }
    out
}

#[derive(Debug)]
struct ParsedTag<'a> {
    name: String,
    closing: bool,
    self_closing: bool,
    attrs: Vec<ParsedAttr<'a>>,
}

#[derive(Debug)]
struct ParsedAttr<'a> {
    name: String,
    value: Option<&'a str>,
}

impl<'a> ParsedTag<'a> {
    fn parse(raw: &'a str) -> Option<Self> {
        let raw = raw.trim();
        let closing = raw.starts_with('/');
        let raw = raw.strip_prefix('/').unwrap_or(raw).trim_start();
        let bytes = raw.as_bytes();
        let mut cursor = 0usize;
        while cursor < bytes.len()
            && !bytes[cursor].is_ascii_whitespace()
            && bytes[cursor] != b'/'
            && bytes[cursor] != b'>'
        {
            cursor += 1;
        }
        if cursor == 0 {
            return None;
        }
        let name = raw[..cursor].to_ascii_lowercase();
        if !is_tag_name(&name) {
            return None;
        }
        let attr_text = raw[cursor..].trim();
        let self_closing = attr_text.ends_with('/');
        let attr_text = attr_text.trim_end_matches('/').trim();
        Some(Self { name, closing, self_closing, attrs: parse_attrs(attr_text) })
    }
}

fn parse_attrs(mut raw: &str) -> Vec<ParsedAttr<'_>> {
    let mut attrs = Vec::new();
    while !raw.is_empty() {
        raw = raw.trim_start();
        if raw.is_empty() {
            break;
        }
        let bytes = raw.as_bytes();
        let mut name_end = 0usize;
        while name_end < bytes.len()
            && !bytes[name_end].is_ascii_whitespace()
            && bytes[name_end] != b'='
            && bytes[name_end] != b'/'
        {
            name_end += 1;
        }
        if name_end == 0 {
            break;
        }
        let name = raw[..name_end].to_ascii_lowercase();
        raw = &raw[name_end..];
        raw = raw.trim_start();
        let mut value = None;
        if raw.starts_with('=') {
            raw = raw[1..].trim_start();
            if let Some(quote) =
                raw.as_bytes().first().copied().filter(|b| *b == b'"' || *b == b'\'')
            {
                let value_start = 1usize;
                if let Some(end) = raw[value_start..].bytes().position(|byte| byte == quote) {
                    let value_end = value_start + end;
                    value = Some(&raw[value_start..value_end]);
                    raw = &raw[value_end + 1..];
                } else {
                    value = Some(&raw[value_start..]);
                    raw = "";
                }
            } else {
                let value_end = raw
                    .bytes()
                    .position(|byte| byte.is_ascii_whitespace() || byte == b'/')
                    .unwrap_or(raw.len());
                value = Some(&raw[..value_end]);
                raw = &raw[value_end..];
            }
        }
        attrs.push(ParsedAttr { name, value });
    }
    attrs
}

fn write_sanitized_attr(out: &mut String, attr: &ParsedAttr<'_>, config: &SanitizeConfig) {
    if attr.name.starts_with("on") || !is_attr_name(&attr.name) || !config.allows_attr(&attr.name) {
        return;
    }
    if matches!(attr.name.as_str(), "href" | "src" | "action")
        && attr.value.is_some_and(|value| !config.allows_url(value))
    {
        return;
    }
    out.push(' ');
    out.push_str(&attr.name);
    if let Some(value) = attr.value {
        out.push_str("=\"");
        escape_html_attr(value, out);
        out.push('"');
    }
}

fn scan_tag_end(html: &str, start: usize) -> Option<usize> {
    let bytes = html.as_bytes();
    let mut cursor = start;
    let mut quote = None;
    while cursor < bytes.len() {
        match quote {
            Some(q) if bytes[cursor] == q => quote = None,
            Some(_) => {}
            None if bytes[cursor] == b'"' || bytes[cursor] == b'\'' => quote = Some(bytes[cursor]),
            None if bytes[cursor] == b'>' => return Some(cursor + 1),
            None => {}
        }
        cursor += 1;
    }
    None
}

fn find_ci(haystack: &str, from: usize, needle: &str) -> Option<usize> {
    let hay = haystack.as_bytes();
    let pat = needle.as_bytes();
    if pat.is_empty() || from > hay.len() || hay.len() - from < pat.len() {
        return None;
    }
    let last_start = hay.len() - pat.len();
    let rest = &pat[1..];
    let mut base = from;
    while base <= last_start {
        let rel = memchr::memchr(pat[0], &hay[base..=last_start])?;
        let index = base + rel;
        if hay[index + 1..index + pat.len()].eq_ignore_ascii_case(rest) {
            return Some(index);
        }
        base = index + 1;
    }
    None
}

fn is_tag_name(value: &str) -> bool {
    value.bytes().all(|byte| byte.is_ascii_alphanumeric() || byte == b'-')
}

fn is_attr_name(value: &str) -> bool {
    value.bytes().all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b':' | b'_'))
}

fn escape_html_text(value: &str, out: &mut String) {
    for ch in value.chars() {
        match ch {
            '&' => out.push_str("&amp;"),
            '<' => out.push_str("&lt;"),
            '>' => out.push_str("&gt;"),
            _ => out.push(ch),
        }
    }
}

fn escape_html_attr(value: &str, out: &mut String) {
    for ch in value.chars() {
        match ch {
            '&' => out.push_str("&amp;"),
            '"' => out.push_str("&quot;"),
            '<' => out.push_str("&lt;"),
            '>' => out.push_str("&gt;"),
            _ => out.push(ch),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn removes_scripts_and_event_handlers() {
        let html =
            r#"<p onclick="x()">Hi<script>alert(1)</script><a href="javascript:x">x</a></p>"#;
        let sanitized = sanitize_html(html, Some(&JsSanitizeOptions::default()));

        assert!(!sanitized.contains("script"));
        assert!(!sanitized.contains("onclick"));
        assert!(!sanitized.contains("javascript"));
        assert!(sanitized.contains("<p>Hi<a>x</a></p>"));
    }

    #[test]
    fn keeps_safe_iframe_sources() {
        let html =
            r#"<iframe src="https://open.spotify.com/embed/track/a" loading="lazy"></iframe>"#;
        let sanitized = sanitize_html(html, Some(&JsSanitizeOptions::default()));

        assert!(sanitized.contains("<iframe"));
        assert!(sanitized.contains("src=\"https://open.spotify.com/embed/track/a\""));
    }
}
