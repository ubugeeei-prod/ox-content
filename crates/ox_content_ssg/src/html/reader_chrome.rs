//! Opt-in reader chrome: copy buttons, outbound-link icons, and back-to-top.

use serde::{Deserialize, Serialize};

use super::utils::escape_html;

/// Copy, outbound-link, and back-to-top flags. All off unless enabled.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct ReaderChrome {
    /// Copy button on fenced `<pre>` blocks.
    #[serde(default)]
    pub copy: bool,
    /// Icon and `rel` on outbound `http(s)` links outside code.
    #[serde(default, rename = "externalLinks")]
    pub external_links: bool,
    /// Back-to-top control that appears after scroll.
    #[serde(default, rename = "backToTop")]
    pub back_to_top: bool,
}

impl ReaderChrome {
    /// Every control off. Omitted / `false` in JS maps here.
    pub fn disabled() -> Self {
        Self::default()
    }

    /// `true` in JS: enable all three controls.
    pub fn enabled() -> Self {
        Self { copy: true, external_links: true, back_to_top: true }
    }

    pub(super) fn is_enabled(self) -> bool {
        self.copy || self.external_links || self.back_to_top
    }

    pub(super) fn needs_js(self) -> bool {
        self.copy || self.back_to_top
    }
}

pub(super) const READER_CHROME_CSS: &str = include_str!("reader_chrome.css");
pub(super) const READER_CHROME_JS: &str = include_str!("reader_chrome.js");
const COPY_BUTTON: &str = concat!(
    "<button type=\"button\" class=\"ox-copy\" data-ox-copy ",
    "aria-label=\"Copy code\" title=\"Copy code\"></button>",
    "<span class=\"ox-copy-status\" data-ox-copy-status role=\"status\" ",
    "aria-live=\"polite\"></span>"
);

/// Rewrites article HTML. Fenced code is never treated as a link target.
pub(super) fn apply_reader_chrome(html: &str, chrome: ReaderChrome) -> String {
    if !chrome.copy && !chrome.external_links {
        return html.to_string();
    }

    let mut out = String::with_capacity(html.len().saturating_add(64));
    let mut rest = html;
    let mut in_pre = 0_u32;
    let mut in_code = 0_u32;

    while let Some(lt) = rest.find('<') {
        out.push_str(&rest[..lt]);
        let tag = &rest[lt..];
        let Some(end) = find_tag_end(tag) else {
            out.push_str(tag);
            return out;
        };
        let raw = &tag[..=end];
        let name = tag_name(raw);

        if name.eq_ignore_ascii_case("pre") {
            if !is_closing(raw) {
                if chrome.copy
                    && in_pre == 0
                    && let Some(element) = take_element(tag, "pre")
                {
                    out.push_str("<div class=\"ox-code\">");
                    out.push_str(COPY_BUTTON);
                    out.push_str(element);
                    out.push_str("</div>");
                    rest = &tag[element.len()..];
                    continue;
                }
                in_pre = in_pre.saturating_add(1);
            } else {
                in_pre = in_pre.saturating_sub(1);
            }
        } else if name.eq_ignore_ascii_case("code") {
            if is_closing(raw) {
                in_code = in_code.saturating_sub(1);
            } else if !is_self_closing(raw) {
                in_code = in_code.saturating_add(1);
            }
        } else if chrome.external_links
            && name.eq_ignore_ascii_case("a")
            && !is_closing(raw)
            && in_pre == 0
            && in_code == 0
            && let Some((rewritten, consumed)) = rewrite_anchor(tag)
        {
            out.push_str(&rewritten);
            rest = &tag[consumed..];
            continue;
        }

        out.push_str(raw);
        rest = &tag[end + 1..];
    }

    out.push_str(rest);
    out
}

fn rewrite_anchor(html: &str) -> Option<(String, usize)> {
    let tag_end = find_tag_end(html)?;
    let open = &html[..=tag_end];
    if is_self_closing(open) {
        return None;
    }
    let href = attr_value(open, "href")?;
    let decoded = decode_basic_entities(&href);
    match classify_href(&decoded) {
        HrefKind::Skip => None,
        HrefKind::Dangerous => {
            let (inner, close_len) = split_anchor_inner(html, tag_end)?;
            let markup = format!("<span class=\"ox-external-inert\">{inner}</span>");
            Some((markup, tag_end + 1 + inner.len() + close_len))
        }
        HrefKind::Outbound => {
            let (inner, close_len) = split_anchor_inner(html, tag_end)?;
            let title = attr_value(open, "title").map(|value| decode_basic_entities(&value));
            let class = merge_class(attr_value(open, "class").as_deref(), "ox-external");
            let id = attr_value(open, "id");
            let mut markup = String::from("<a href=\"");
            markup.push_str(&escape_html(&decoded));
            markup.push_str("\" class=\"");
            markup.push_str(&escape_html(&class));
            markup.push_str("\" rel=\"noopener noreferrer\" target=\"_blank\"");
            if let Some(id) = id.filter(|id| !id.is_empty()) {
                markup.push_str(" id=\"");
                markup.push_str(&escape_html(&decode_basic_entities(&id)));
                markup.push('"');
            }
            if let Some(title) = title.filter(|title| !title.is_empty()) {
                markup.push_str(" title=\"");
                markup.push_str(&escape_html(&title));
                markup.push('"');
            }
            markup.push('>');
            markup.push_str(inner);
            markup.push_str(external_marker::external_marker(inner));
            markup.push_str("</a>");
            Some((markup, tag_end + 1 + inner.len() + close_len))
        }
    }
}

fn split_anchor_inner(html: &str, tag_end: usize) -> Option<(&str, usize)> {
    let rest = html.get(tag_end + 1..)?;
    let close_at = find_close_tag(rest, "a")?;
    let close = rest.get(close_at..)?;
    let close_end = find_tag_end(close)?;
    Some((&rest[..close_at], close_end + 1))
}

fn take_element<'a>(html: &'a str, name: &str) -> Option<&'a str> {
    let tag_end = find_tag_end(html)?;
    if is_self_closing(&html[..=tag_end]) {
        return Some(&html[..=tag_end]);
    }
    let rest = html.get(tag_end + 1..)?;
    let close_at = find_close_tag(rest, name)?;
    let close = rest.get(close_at..)?;
    let close_end = find_tag_end(close)?;
    html.get(..tag_end + 1 + close_at + close_end + 1)
}

fn find_close_tag(html: &str, name: &str) -> Option<usize> {
    let mut depth = 1_u32;
    let mut offset = 0;
    while let Some(rel) = html[offset..].find('<') {
        let abs = offset + rel;
        let tag = &html[abs..];
        let end = find_tag_end(tag)?;
        let raw = &tag[..=end];
        if tag_name(raw).eq_ignore_ascii_case(name) {
            if is_closing(raw) {
                depth = depth.saturating_sub(1);
                if depth == 0 {
                    return Some(abs);
                }
            } else if !is_self_closing(raw) {
                depth = depth.saturating_add(1);
            }
        }
        offset = abs + end + 1;
    }
    None
}

fn find_tag_end(tag: &str) -> Option<usize> {
    let bytes = tag.as_bytes();
    let mut quote = None;
    for (i, byte) in bytes.iter().copied().enumerate().skip(1) {
        match quote {
            Some(q) if byte == q => quote = None,
            None if byte == b'"' || byte == b'\'' => quote = Some(byte),
            None if byte == b'>' => return Some(i),
            _ => {}
        }
    }
    None
}

fn tag_name(tag: &str) -> &str {
    let rest = tag.strip_prefix('<').unwrap_or(tag);
    let rest = rest.strip_prefix('/').unwrap_or(rest);
    let end = rest
        .find(|ch: char| ch.is_ascii_whitespace() || ch == '>' || ch == '/')
        .unwrap_or(rest.len());
    &rest[..end]
}

fn is_closing(tag: &str) -> bool {
    tag.as_bytes().get(1) == Some(&b'/')
}

fn is_self_closing(tag: &str) -> bool {
    tag.trim_end().ends_with("/>")
}

#[derive(Clone, Copy)]
enum HrefKind {
    Outbound,
    Skip,
    Dangerous,
}

fn classify_href(href: &str) -> HrefKind {
    let trimmed = href.trim();
    if trimmed.is_empty()
        || trimmed.starts_with('#')
        || trimmed.starts_with('?')
        || (trimmed.starts_with('/') && !trimmed.starts_with("//"))
        || trimmed.starts_with("./")
        || trimmed.starts_with("../")
    {
        return HrefKind::Skip;
    }

    let compact: String = trimmed.chars().filter(|ch| !ch.is_ascii_whitespace()).collect();
    let lower = compact.to_ascii_lowercase();
    if lower.starts_with("javascript:")
        || lower.starts_with("data:")
        || lower.starts_with("vbscript:")
    {
        return HrefKind::Dangerous;
    }
    if lower.starts_with("http://") || lower.starts_with("https://") || lower.starts_with("//") {
        return HrefKind::Outbound;
    }
    match lower.find(':') {
        Some(idx) if matches!(&lower[..idx], "mailto" | "tel") => HrefKind::Skip,
        Some(_) => HrefKind::Dangerous,
        None => HrefKind::Skip,
    }
}

fn attr_value(tag: &str, name: &str) -> Option<String> {
    let lower = tag.to_ascii_lowercase();
    let needle = name.to_ascii_lowercase();
    let mut from = 0;
    while let Some(rel) = lower[from..].find(&needle) {
        let abs = from + rel;
        let before = abs.checked_sub(1).and_then(|i| tag.as_bytes().get(i)).copied();
        if before.is_some_and(|byte| byte.is_ascii_whitespace()) {
            let after = tag[abs + name.len()..].trim_start();
            if let Some(assigned) = after.strip_prefix('=') {
                return Some(parse_attr_assigned(assigned.trim_start()));
            }
        }
        from = abs + 1;
    }
    None
}

fn parse_attr_assigned(rest: &str) -> String {
    match rest.as_bytes().first() {
        Some(&b'"') => take_quoted(rest, '"'),
        Some(&b'\'') => take_quoted(rest, '\''),
        _ => rest.chars().take_while(|ch| !ch.is_ascii_whitespace() && *ch != '>').collect(),
    }
}

fn take_quoted(rest: &str, quote: char) -> String {
    let mut chars = rest.chars();
    chars.next();
    chars.take_while(|ch| *ch != quote).collect()
}

fn merge_class(existing: Option<&str>, extra: &str) -> String {
    match existing.map(str::trim).filter(|value| !value.is_empty()) {
        Some(value) if value.split_whitespace().any(|class| class == extra) => value.to_string(),
        Some(value) => format!("{value} {extra}"),
        None => extra.to_string(),
    }
}

fn decode_basic_entities(value: &str) -> String {
    let mut out = String::with_capacity(value.len());
    let mut rest = value;
    while let Some(amp) = rest.find('&') {
        out.push_str(&rest[..amp]);
        let ent = &rest[amp..];
        if let Some((ch, skip)) = decode_one_entity(ent) {
            out.push(ch);
            rest = &ent[skip..];
        } else {
            out.push('&');
            rest = &ent[1..];
        }
    }
    out.push_str(rest);
    out
}

fn decode_one_entity(ent: &str) -> Option<(char, usize)> {
    const NAMED: &[(&str, char)] =
        &[("&amp;", '&'), ("&lt;", '<'), ("&gt;", '>'), ("&quot;", '"'), ("&apos;", '\'')];
    for (token, ch) in NAMED {
        if ent.starts_with(token) {
            return Some((*ch, token.len()));
        }
    }
    if let Some(rest) = ent.strip_prefix("&#x").or_else(|| ent.strip_prefix("&#X")) {
        let end = rest.find(';')?;
        let ch = char::from_u32(u32::from_str_radix(&rest[..end], 16).ok()?)?;
        return Some((ch, 3 + end + 1));
    }
    if let Some(rest) = ent.strip_prefix("&#") {
        let end = rest.find(';')?;
        let ch = char::from_u32(rest[..end].parse().ok()?)?;
        return Some((ch, 2 + end + 1));
    }
    None
}

mod external_marker;
#[cfg(test)]
mod tests;
