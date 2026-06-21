use std::collections::HashMap;

use compact_str::CompactString;
use smallvec::SmallVec;

use super::{parser::HtmlElement, FrameworkComponentIsland};

pub(super) fn find_island<'a>(
    element: &HtmlElement,
    islands: &'a [FrameworkComponentIsland],
) -> Option<&'a FrameworkComponentIsland> {
    let island_id = get_attribute_value(element, "data-ox-id")
        .or_else(|| get_attribute_value(element, "dataOxId"))?;
    islands.iter().find(|island| island.id == island_id)
}

pub(super) fn push_attribute_value(output: &mut String, value: Option<&str>) {
    if let Some(value) = value {
        push_js_string_literal(output, value);
    } else {
        output.push_str("true");
    }
}

pub(super) fn render_json_object_literal(value: &HashMap<String, serde_json::Value>) -> String {
    let mut output = String::with_capacity(value.len().saturating_mul(24).max(4));
    push_json_object_literal(&mut output, value);
    output
}

pub(super) fn push_json_object_literal(
    output: &mut String,
    value: &HashMap<String, serde_json::Value>,
) {
    if value.is_empty() {
        output.push_str("null");
        return;
    }

    let mut properties = SmallVec::<[(&String, &serde_json::Value); 8]>::with_capacity(value.len());
    properties.extend(value.iter());
    properties.sort_unstable_by_key(|(key, _)| *key);

    output.push_str("{ ");
    for (index, (key, value)) in properties.iter().enumerate() {
        if index > 0 {
            output.push_str(", ");
        }
        push_js_string_literal(output, key);
        output.push_str(": ");
        output.push_str(&json_value_literal(value));
    }
    output.push_str(" }");
}

pub(super) fn push_object_entry_key(output: &mut String, has_entries: &mut bool, key: &str) {
    if *has_entries {
        output.push_str(", ");
    } else {
        output.push_str("{ ");
        *has_entries = true;
    }
    push_js_string_literal(output, key);
    output.push_str(": ");
}

pub(super) fn finish_object_literal(output: &mut String, has_entries: bool) {
    if has_entries {
        output.push_str(" }");
    } else {
        output.push_str("null");
    }
}

pub(super) fn should_skip_property(name: &str) -> bool {
    name.starts_with("data-ox-")
}

pub(super) fn to_data_or_aria_attribute_name(name: &str) -> CompactString {
    if (name.starts_with("data") || name.starts_with("aria"))
        && name.bytes().any(|byte| byte.is_ascii_uppercase())
    {
        camel_to_kebab(name)
    } else {
        CompactString::from(name)
    }
}

pub(super) fn js_string_literal(value: &str) -> String {
    let mut output = String::with_capacity(value.len() + 2);
    push_js_string_literal(&mut output, value);
    output
}

pub(super) fn push_js_string_literal(output: &mut String, value: &str) {
    output.push('"');
    for ch in value.chars() {
        match ch {
            '"' => output.push_str("\\\""),
            '\\' => output.push_str("\\\\"),
            '\n' => output.push_str("\\n"),
            '\r' => output.push_str("\\r"),
            '\t' => output.push_str("\\t"),
            '\u{08}' => output.push_str("\\b"),
            '\u{0c}' => output.push_str("\\f"),
            ch if ch.is_control() => push_json_unicode_escape(output, ch),
            ch => output.push(ch),
        }
    }
    output.push('"');
}

fn camel_to_kebab(value: &str) -> CompactString {
    let mut output = CompactString::with_capacity(value.len());
    for ch in value.chars() {
        if ch.is_ascii_uppercase() {
            output.push('-');
            output.push(ch.to_ascii_lowercase());
        } else {
            output.push(ch);
        }
    }
    output
}

fn json_value_literal(value: &serde_json::Value) -> String {
    serde_json::to_string(value).unwrap_or_else(|_| String::from("null"))
}

fn push_json_unicode_escape(output: &mut String, ch: char) {
    const HEX: &[u8; 16] = b"0123456789abcdef";
    let value = ch as u32;
    output.push_str("\\u");
    output.push(HEX[((value >> 12) & 0xf) as usize] as char);
    output.push(HEX[((value >> 8) & 0xf) as usize] as char);
    output.push(HEX[((value >> 4) & 0xf) as usize] as char);
    output.push(HEX[(value & 0xf) as usize] as char);
}

fn get_attribute_value<'a>(element: &'a HtmlElement, name: &str) -> Option<&'a str> {
    element
        .attributes
        .iter()
        .find(|attribute| attribute.name == name)
        .and_then(|attribute| attribute.value.as_deref())
}
