use compact_str::CompactString;

use super::{
    parser::{HtmlAttribute, HtmlElement},
    shared, FrameworkComponentIsland,
};

pub(super) fn render_root(children: &[String]) -> String {
    let mut output = String::with_capacity(48 + children.iter().map(String::len).sum::<usize>());
    output.push_str("createElement('div', { className: 'ox-content' }");
    push_children(&mut output, children);
    output.push(')');
    output
}

pub(super) fn render_element(element: &HtmlElement, children: &[String]) -> String {
    let props = render_props(&element.attributes);
    let mut output = String::with_capacity(
        18 + element.tag_name.len() + props.len() + children.iter().map(String::len).sum::<usize>(),
    );
    output.push_str("createElement(");
    shared::push_js_string_literal(&mut output, &element.tag_name);
    output.push_str(", ");
    output.push_str(&props);
    push_children(&mut output, children);
    output.push(')');
    output
}

pub(super) fn render_island(island: &FrameworkComponentIsland) -> String {
    let props = shared::render_json_object_literal(&island.props);
    let content_len = island.content.as_ref().map_or(0, |content| content.len() + 4);
    let mut output = String::with_capacity(17 + island.name.len() + props.len() + content_len);
    output.push_str("createElement(");
    output.push_str(&island.name);
    output.push_str(", ");
    output.push_str(&props);
    if let Some(content) = &island.content {
        output.push_str(", ");
        shared::push_js_string_literal(&mut output, content);
    }
    output.push(')');
    output
}

pub(super) fn render_function_module(expression: &str) -> String {
    let mut output = String::with_capacity(82 + expression.len());
    output.push_str("import { createElement } from 'react';\n\n");
    output.push_str("export function renderMarkdownContent() {\n  return ");
    output.push_str(expression);
    output.push_str(";\n}\n");
    output
}

pub(super) fn component_module(expression: &str) -> String {
    let mut output = String::with_capacity(88 + expression.len());
    output.push_str("import { createElement } from 'react';\n\n");
    output.push_str("export default function MarkdownContent() {\n  return ");
    output.push_str(expression);
    output.push_str(";\n}\n");
    output
}

pub(super) fn inner_html_component_module(html: &str) -> String {
    let mut output = String::with_capacity(184 + html.len());
    output.push_str("import { createElement } from 'react';\n\n");
    output.push_str("const rawHtml = ");
    shared::push_raw_html_js_string_literal(&mut output, html);
    output.push_str(";\n\n");
    output.push_str("export default function MarkdownContent() {\n");
    output.push_str("  return createElement('div', {\n");
    output.push_str("    className: 'ox-content',\n");
    output.push_str("    dangerouslySetInnerHTML: { __html: rawHtml },\n");
    output.push_str("  });\n");
    output.push_str("}\n");
    output
}

fn push_children(output: &mut String, children: &[String]) {
    for child in children {
        output.push_str(", ");
        output.push_str(child);
    }
}

fn render_props(attributes: &[HtmlAttribute]) -> String {
    let mut output = String::with_capacity(attributes.len().saturating_mul(24).max(4));
    let mut has_entries = false;
    for attribute in attributes {
        let prop_name = to_prop_name(&attribute.name);
        if shared::should_skip_property(prop_name.as_str()) {
            continue;
        }
        if prop_name == "style" {
            if let Some(value) = &attribute.value {
                shared::push_object_entry_key(&mut output, &mut has_entries, prop_name.as_str());
                push_style_object(&mut output, value);
            }
            continue;
        }
        shared::push_object_entry_key(&mut output, &mut has_entries, prop_name.as_str());
        shared::push_attribute_value(&mut output, attribute.value.as_deref());
    }
    shared::finish_object_literal(&mut output, has_entries);
    output
}

fn to_prop_name(name: &str) -> CompactString {
    match name {
        "class" | "className" => CompactString::const_new("className"),
        "for" | "htmlFor" => CompactString::const_new("htmlFor"),
        _ => shared::to_data_or_aria_attribute_name(name),
    }
}

fn push_style_object(output: &mut String, value: &str) {
    let mut has_entries = false;
    for declaration in value.split(';') {
        let Some((name, value)) = declaration.split_once(':') else {
            continue;
        };
        let name = name.trim();
        let value = value.trim();
        if name.is_empty() || value.is_empty() {
            continue;
        }
        let property_name = css_property_to_react_name(name);
        shared::push_object_entry_key(output, &mut has_entries, property_name.as_str());
        shared::push_js_string_literal(output, value);
    }
    shared::finish_object_literal(output, has_entries);
}

fn css_property_to_react_name(name: &str) -> CompactString {
    if name.starts_with("--") {
        return CompactString::from(name);
    }

    let mut output = CompactString::with_capacity(name.len());
    let mut uppercase_next = false;
    for ch in name.chars() {
        if ch == '-' {
            uppercase_next = true;
        } else if uppercase_next {
            output.push(ch.to_ascii_uppercase());
            uppercase_next = false;
        } else {
            output.push(ch);
        }
    }
    output
}
