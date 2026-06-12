use std::borrow::Cow;

use rustc_hash::FxHashMap;

/// Parses YAML frontmatter from Markdown content.
pub fn parse_frontmatter(source: &str) -> (Cow<'_, str>, FxHashMap<String, serde_json::Value>) {
    // wasm-bindgen exposes the JS string as `&str` for this call. Return
    // `Cow::Borrowed` for the Markdown body so stripping frontmatter adjusts
    // slice boundaries instead of copying the body into a new `String`.
    let mut frontmatter = FxHashMap::default();

    if !source.starts_with("---") {
        return (Cow::Borrowed(source), frontmatter);
    }

    let rest = &source[3..];
    let Some(end_pos) = rest.find("\n---") else {
        return (Cow::Borrowed(source), frontmatter);
    };

    let frontmatter_str = rest[..end_pos].trim_start_matches('\n');
    let content = rest[end_pos + 4..].trim_start_matches('\n');

    for line in frontmatter_str.lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }

        if let Some(colon_pos) = line.find(':') {
            let key = line[..colon_pos].trim().to_string();
            let value_str = line[colon_pos + 1..].trim();
            frontmatter.insert(key, parse_value(value_str));
        }
    }

    (Cow::Borrowed(content), frontmatter)
}

fn parse_value(value_str: &str) -> serde_json::Value {
    if value_str == "true" {
        serde_json::Value::Bool(true)
    } else if value_str == "false" {
        serde_json::Value::Bool(false)
    } else if let Ok(n) = value_str.parse::<i64>() {
        serde_json::Value::Number(n.into())
    } else if let Ok(n) = value_str.parse::<f64>() {
        serde_json::Number::from_f64(n).map_or_else(
            || serde_json::Value::String(value_str.to_string()),
            serde_json::Value::Number,
        )
    } else {
        let s = value_str.trim_matches('"').trim_matches('\'');
        serde_json::Value::String(s.to_string())
    }
}
