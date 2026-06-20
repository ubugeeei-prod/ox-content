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

mod config;
mod parser;

#[cfg(test)]
mod tests;

use config::SanitizeConfig;
use parser::{find_ci, is_attr_name, scan_tag_end, ParsedAttr, ParsedTag};

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

fn write_sanitized_attr(out: &mut String, attr: &ParsedAttr<'_>, config: &SanitizeConfig) {
    if attr.name.starts_with("on") || !is_attr_name(&attr.name) || !config.allows_attr(&attr.name) {
        return;
    }
    if let Some(value) = attr.value {
        match attr.name.as_str() {
            "href" | "src" | "action" | "poster" if !config.allows_url(value) => return,
            "srcset" if !config.allows_srcset(value) => return,
            _ => {}
        }
    }
    out.push(' ');
    out.push_str(&attr.name);
    if let Some(value) = attr.value {
        out.push_str("=\"");
        escape_html_attr(value, out);
        out.push('"');
    }
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
