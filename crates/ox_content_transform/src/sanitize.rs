use std::borrow::Cow;

use crate::SanitizeOptions;

pub fn sanitize_html(html: &str, options: Option<&SanitizeOptions>) -> String {
    let Some(options) = options else {
        return sanitize_html_with_config(html, &SanitizeConfig::default());
    };
    if options.enabled == Some(false) {
        return html.to_string();
    }
    sanitize_html_with_config(html, &SanitizeConfig::from_options(options))
}

mod attrs;
mod config;
mod parser;

#[cfg(test)]
mod tests;

use config::SanitizeConfig;
use parser::{ParsedAttr, ParsedTag, find_ci, is_attr_name, scan_tag_end};

fn sanitize_html_with_config(html: &str, config: &SanitizeConfig) -> String {
    if !html.contains('<') {
        return html.to_string();
    }

    let bytes = html.as_bytes();
    let mut out = String::with_capacity(html.len());
    let mut cursor = 0usize;
    let mut raw_text_ends = RawTextEnds::default();

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
            cursor = if parsed.closing {
                tag_end
            } else {
                raw_text_ends.skip(html, tag_end, &parsed.name).unwrap_or(tag_end)
            };
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
            write_sanitized_attr(&mut out, &parsed.name, &attr, config);
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

/// Where the content of a dropped `<script>` or `<style>` ends.
///
/// Its content is dropped with it, up to the matching closing tag. Once a
/// search for that tag has run off the end of the input, every later one
/// would too: the cursor only moves forward, so the answer is kept instead
/// of searching the rest of the input again for each opener.
#[derive(Default)]
struct RawTextEnds {
    /// Position from which `</script>` no longer occurs.
    script_missing_from: Option<usize>,
    /// Position from which `</style>` no longer occurs.
    style_missing_from: Option<usize>,
}

impl RawTextEnds {
    /// The position just past the closing tag of the `name` element whose
    /// opening tag ends at `from`, or `None` when it is not a raw-text
    /// element or nothing closes it.
    fn skip(&mut self, html: &str, from: usize, name: &str) -> Option<usize> {
        let (close, missing_from) = match name {
            "script" => ("</script>", &mut self.script_missing_from),
            "style" => ("</style>", &mut self.style_missing_from),
            _ => return None,
        };
        if missing_from.is_some_and(|missing| from >= missing) {
            return None;
        }
        let found = find_ci(html, from, close).map(|end| end + close.len());
        if found.is_none() {
            *missing_from = Some(from);
        }
        found
    }
}

fn write_sanitized_attr(
    out: &mut String,
    tag: &str,
    attr: &ParsedAttr<'_>,
    config: &SanitizeConfig,
) {
    if attr.name.starts_with("on")
        || !is_attr_name(&attr.name)
        || !config.allows_attr(&attr.name)
        || attrs::shadows_document_property(tag, &attr.name)
    {
        return;
    }
    // The input is already-escaped HTML, so the value has to be decoded
    // before anything looks at it: the checks below must see the URL the
    // browser will see, and escaping an escaped value again would turn a
    // query string's `&amp;` into a literal `&amp;`.
    let decoded = attr.value.map(|value| decode_char_refs(value));
    if let Some(value) = decoded.as_deref()
        && !attrs::allows_value(config, &attr.name, value)
    {
        return;
    }
    out.push(' ');
    out.push_str(&attr.name);
    if let Some(value) = decoded.as_deref() {
        out.push_str("=\"");
        escape_html_attr(value, out);
        out.push('"');
    }
}

/// Decodes the character references an escaped attribute value can hold:
/// the five HTML escapes and both numeric forms.
///
/// A reference this does not know stays exactly as written, so it is
/// re-escaped on the way out and reaches the page inert — the behaviour
/// every value had before, kept for the names (`&colon;`, `&sol;`, …) that
/// could otherwise disguise a scheme.
fn decode_char_refs(value: &str) -> Cow<'_, str> {
    if !value.contains('&') {
        return Cow::Borrowed(value);
    }
    let mut out = String::with_capacity(value.len());
    let mut rest = value;
    while let Some(at) = rest.find('&') {
        out.push_str(&rest[..at]);
        let tail = &rest[at..];
        if let Some((decoded, consumed)) = scan_char_ref(tail) {
            out.push(decoded);
            rest = &tail[consumed..];
        } else {
            out.push('&');
            rest = &tail[1..];
        }
    }
    out.push_str(rest);
    Cow::Owned(out)
}

/// Scans one character reference at the start of `rest`, which begins with
/// `&`. Returns the character and the bytes consumed including `&` and `;`.
fn scan_char_ref(rest: &str) -> Option<(char, usize)> {
    let bytes = rest.as_bytes();
    if bytes.get(1) != Some(&b'#') {
        for (name, ch) in
            [("amp;", '&'), ("lt;", '<'), ("gt;", '>'), ("quot;", '"'), ("apos;", '\'')]
        {
            if rest[1..].starts_with(name) {
                return Some((ch, 1 + name.len()));
            }
        }
        return None;
    }
    let (digits_start, radix) = match bytes.get(2) {
        Some(b'x' | b'X') => (3, 16),
        _ => (2, 10),
    };
    let mut end = digits_start;
    // Seven digits cover every code point in either radix; more is invalid.
    while end < bytes.len().min(digits_start + 8) && bytes[end].is_ascii_hexdigit() {
        end += 1;
    }
    if end == digits_start || bytes.get(end) != Some(&b';') {
        return None;
    }
    let code = u32::from_str_radix(&rest[digits_start..end], radix).ok()?;
    // U+0000 and anything out of range become the replacement character,
    // matching how the Markdown parser decodes the same references.
    let ch = char::from_u32(code).filter(|ch| *ch != '\0').unwrap_or('\u{FFFD}');
    Some((ch, end + 1))
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
            // A line break inside a quoted value is legal but survives
            // poorly through anything that reads the HTML back a line at a
            // time, and the values that carry them — code sources, titles —
            // arrive written this way.
            '\n' => out.push_str("&#10;"),
            '\r' => out.push_str("&#13;"),
            '\t' => out.push_str("&#9;"),
            _ => out.push(ch),
        }
    }
}
