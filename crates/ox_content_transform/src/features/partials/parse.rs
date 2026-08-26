use rustc_hash::FxHashMap;

use super::super::escape::escape_html_text;
use super::MissingMode;

pub(super) struct PartialDirective {
    pub path: String,
    pub params: FxHashMap<String, String>,
}

pub(super) fn parse_partial_directive(inner: &str) -> Option<PartialDirective> {
    let rest = inner.strip_prefix("@partial:")?.trim();
    let (path, after_path) = parse_path_token(rest);
    Some(PartialDirective { path, params: parse_params(after_path) })
}

fn parse_path_token(rest: &str) -> (String, &str) {
    if rest.is_empty() {
        return (String::new(), "");
    }
    let bytes = rest.as_bytes();
    if bytes.first().is_some_and(|byte| *byte == b'"' || *byte == b'\'') {
        let quote = bytes[0];
        if let Some(rel) = rest[1..].find(quote as char) {
            return (rest[1..1 + rel].to_string(), rest[2 + rel..].trim_start());
        }
        return (rest[1..].to_string(), "");
    }
    let end = rest.find(char::is_whitespace).unwrap_or(rest.len());
    (rest[..end].to_string(), rest[end..].trim_start())
}

fn parse_params(mut rest: &str) -> FxHashMap<String, String> {
    let mut params = FxHashMap::default();
    while !rest.is_empty() {
        let Some(eq) = rest.find('=') else { break };
        let key = rest[..eq].trim();
        rest = rest[eq + 1..].trim_start();
        let (value, next) = parse_param_value(rest);
        if is_param_name(key) {
            params.insert(key.to_string(), value);
        }
        rest = next.trim_start();
    }
    params
}

fn parse_param_value(rest: &str) -> (String, &str) {
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

pub(super) fn substitute_params(
    source: &str,
    params: &FxHashMap<String, String>,
    missing: MissingMode,
    errors: &mut Vec<String>,
    location: &str,
) -> String {
    let mut out = String::with_capacity(source.len());
    let mut cursor = 0usize;
    while let Some(rel) = source[cursor..].find("{{") {
        let start = cursor + rel;
        out.push_str(&source[cursor..start]);
        let after = start + 2;
        let Some(rel_close) = source[after..].find("}}") else {
            out.push_str(&source[start..]);
            return out;
        };
        let close = after + rel_close;
        let name = source[after..close].trim();
        if is_param_name(name) {
            if let Some(value) = params.get(name) {
                escape_html_text(value, &mut out);
            } else {
                if missing == MissingMode::Error {
                    errors.push(format!("Partial parameter `{name}` is missing at {location}."));
                }
                out.push_str(&source[start..close + 2]);
            }
        } else {
            out.push_str(&source[start..close + 2]);
        }
        cursor = close + 2;
    }
    out.push_str(&source[cursor..]);
    out
}

fn is_param_name(name: &str) -> bool {
    let mut chars = name.chars();
    match chars.next() {
        Some(c) if c.is_ascii_alphabetic() || c == '_' => {}
        _ => return false,
    }
    chars.all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '-')
}
