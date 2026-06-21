use compact_str::CompactString;

use super::{
    parser::{HtmlAttribute, HtmlElement},
    shared, FrameworkComponentIsland,
};

pub(super) fn render_root(children: &[String]) -> String {
    let mut output = String::with_capacity(36 + children.iter().map(String::len).sum::<usize>());
    output.push_str("h('div', { class: 'ox-content' }");
    push_children(&mut output, children);
    output.push(')');
    output
}

pub(super) fn render_element(element: &HtmlElement, children: &[String]) -> String {
    let props = render_props(&element.attributes);
    let mut output = String::with_capacity(
        8 + element.tag_name.len() + props.len() + children.iter().map(String::len).sum::<usize>(),
    );
    output.push_str("h(");
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
    let mut output = String::with_capacity(5 + island.name.len() + props.len() + content_len);
    output.push_str("h(");
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
    let mut output = String::with_capacity(68 + expression.len());
    output.push_str("import { h } from 'vue';\n\n");
    output.push_str("export function renderMarkdownContent() {\n  return ");
    output.push_str(expression);
    output.push_str(";\n}\n");
    output
}

pub(super) fn component_module(expression: &str) -> String {
    let mut output = String::with_capacity(138 + expression.len());
    output.push_str("import { defineComponent, h } from 'vue';\n\n");
    output.push_str("export default defineComponent({\n");
    output.push_str("  name: 'MarkdownContent',\n");
    output.push_str("  setup() {\n");
    output.push_str("    return () => ");
    output.push_str(expression);
    output.push_str(";\n");
    output.push_str("  },\n");
    output.push_str("});\n");
    output
}

pub(super) fn inner_html_component_module(html: &str) -> String {
    let mut output = String::with_capacity(188 + html.len());
    output.push_str("import { defineComponent, h } from 'vue';\n\n");
    output.push_str("const rawHtml = ");
    shared::push_raw_html_js_string_literal(&mut output, html);
    output.push_str(";\n\n");
    output.push_str("export default defineComponent({\n");
    output.push_str("  name: 'MarkdownContent',\n");
    output.push_str("  setup() {\n");
    output.push_str("    return () => h('div', { class: 'ox-content', innerHTML: rawHtml });\n");
    output.push_str("  },\n");
    output.push_str("});\n");
    output
}

fn push_children(output: &mut String, children: &[String]) {
    match children {
        [] => {}
        [child] => {
            output.push_str(", ");
            output.push_str(child);
        }
        _ => {
            output.push_str(", [");
            for (index, child) in children.iter().enumerate() {
                if index > 0 {
                    output.push_str(", ");
                }
                output.push_str(child);
            }
            output.push(']');
        }
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
        shared::push_object_entry_key(&mut output, &mut has_entries, prop_name.as_str());
        shared::push_attribute_value(&mut output, attribute.value.as_deref());
    }
    shared::finish_object_literal(&mut output, has_entries);
    output
}

fn to_prop_name(name: &str) -> CompactString {
    match name {
        "className" => CompactString::const_new("class"),
        "htmlFor" => CompactString::const_new("for"),
        _ => shared::to_data_or_aria_attribute_name(name),
    }
}
