use std::collections::HashMap;

use super::{
    parser::{HtmlAttribute, HtmlElement, HtmlNode},
    FrameworkCodegenTarget, JsFrameworkComponentIsland,
};

pub(super) struct FrameworkCodegen<'a> {
    pub target: FrameworkCodegenTarget,
    pub islands: &'a [JsFrameworkComponentIsland],
}

impl FrameworkCodegen<'_> {
    pub(super) fn render_root(&self, nodes: &[HtmlNode]) -> String {
        let children = nodes.iter().filter_map(|node| self.render_node(node)).collect::<Vec<_>>();

        match self.target {
            FrameworkCodegenTarget::React => {
                format!(
                    "createElement('div', {{ className: 'ox-content' }}{})",
                    render_react_children(&children)
                )
            }
            FrameworkCodegenTarget::Vue => {
                format!("h('div', {{ class: 'ox-content' }}{})", render_vue_children(&children))
            }
        }
    }

    fn render_node(&self, node: &HtmlNode) -> Option<String> {
        match node {
            HtmlNode::Text(value) if value.is_empty() => None,
            HtmlNode::Text(value) => Some(js_string_literal(value)),
            HtmlNode::Element(element) => Some(self.render_element(element)),
        }
    }

    fn render_element(&self, element: &HtmlElement) -> String {
        if let Some(island) = self.find_island(element) {
            return self.render_island(island);
        }

        let children =
            element.children.iter().filter_map(|node| self.render_node(node)).collect::<Vec<_>>();

        match self.target {
            FrameworkCodegenTarget::React => format!(
                "createElement({}, {}{})",
                js_string_literal(&element.tag_name),
                render_react_props(&element.attributes),
                render_react_children(&children)
            ),
            FrameworkCodegenTarget::Vue => format!(
                "h({}, {}{})",
                js_string_literal(&element.tag_name),
                render_vue_props(&element.attributes),
                render_vue_children(&children)
            ),
        }
    }

    fn find_island(&self, element: &HtmlElement) -> Option<&JsFrameworkComponentIsland> {
        let island_id = get_attribute_value(element, "data-ox-id")
            .or_else(|| get_attribute_value(element, "dataOxId"))?;
        self.islands.iter().find(|island| island.id == island_id)
    }

    fn render_island(&self, island: &JsFrameworkComponentIsland) -> String {
        let props = render_json_object_literal(&island.props);
        let content = island
            .content
            .as_ref()
            .map(|content| format!(", {}", js_string_literal(content)))
            .unwrap_or_default();

        match self.target {
            FrameworkCodegenTarget::React => {
                format!("createElement({}, {props}{content})", island.name)
            }
            FrameworkCodegenTarget::Vue => format!("h({}, {props}{content})", island.name),
        }
    }
}

fn render_react_children(children: &[String]) -> String {
    if children.is_empty() {
        String::new()
    } else {
        format!(", {}", children.join(", "))
    }
}

fn render_vue_children(children: &[String]) -> String {
    match children {
        [] => String::new(),
        [child] => format!(", {child}"),
        _ => format!(", [{}]", children.join(", ")),
    }
}

fn render_react_props(attributes: &[HtmlAttribute]) -> String {
    let mut entries = Vec::new();
    for attribute in attributes {
        let prop_name = to_react_prop_name(&attribute.name);
        if should_skip_property(&prop_name) {
            continue;
        }
        if prop_name == "style" {
            if let Some(value) = &attribute.value {
                entries.push(format!(
                    "{}: {}",
                    js_string_literal(&prop_name),
                    render_style_object(value)
                ));
            }
            continue;
        }
        entries.push(format!(
            "{}: {}",
            js_string_literal(&prop_name),
            render_attribute_value(attribute.value.as_deref())
        ));
    }
    render_object_entries(&entries)
}

fn render_vue_props(attributes: &[HtmlAttribute]) -> String {
    let mut entries = Vec::new();
    for attribute in attributes {
        let prop_name = to_vue_prop_name(&attribute.name);
        if should_skip_property(&prop_name) {
            continue;
        }
        entries.push(format!(
            "{}: {}",
            js_string_literal(&prop_name),
            render_attribute_value(attribute.value.as_deref())
        ));
    }
    render_object_entries(&entries)
}

fn should_skip_property(name: &str) -> bool {
    name.starts_with("data-ox-")
}

fn to_react_prop_name(name: &str) -> String {
    match name {
        "class" | "className" => "className".to_string(),
        "for" | "htmlFor" => "htmlFor".to_string(),
        _ => to_data_or_aria_attribute_name(name),
    }
}

fn to_vue_prop_name(name: &str) -> String {
    match name {
        "className" => "class".to_string(),
        "htmlFor" => "for".to_string(),
        _ => to_data_or_aria_attribute_name(name),
    }
}

fn to_data_or_aria_attribute_name(name: &str) -> String {
    if (name.starts_with("data") || name.starts_with("aria"))
        && name.bytes().any(|byte| byte.is_ascii_uppercase())
    {
        camel_to_kebab(name)
    } else {
        name.to_string()
    }
}

fn camel_to_kebab(value: &str) -> String {
    let mut output = String::with_capacity(value.len());
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

fn render_attribute_value(value: Option<&str>) -> String {
    match value {
        Some(value) => js_string_literal(value),
        None => "true".to_string(),
    }
}

fn render_style_object(value: &str) -> String {
    let entries = value
        .split(';')
        .filter_map(|declaration| {
            let (name, value) = declaration.split_once(':')?;
            let name = name.trim();
            let value = value.trim();
            if name.is_empty() || value.is_empty() {
                return None;
            }
            Some(format!(
                "{}: {}",
                js_string_literal(&css_property_to_react_name(name)),
                js_string_literal(value)
            ))
        })
        .collect::<Vec<_>>();
    render_object_entries(&entries)
}

fn css_property_to_react_name(name: &str) -> String {
    if name.starts_with("--") {
        return name.to_string();
    }

    let mut output = String::with_capacity(name.len());
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

fn render_json_object_literal(value: &HashMap<String, serde_json::Value>) -> String {
    let mut properties = value.iter().collect::<Vec<_>>();
    properties.sort_unstable_by_key(|(key, _)| *key);
    let entries = properties
        .into_iter()
        .map(|(key, value)| format!("{}: {}", js_string_literal(key), json_value_literal(value)))
        .collect::<Vec<_>>();
    render_object_entries(&entries)
}

fn json_value_literal(value: &serde_json::Value) -> String {
    serde_json::to_string(value).unwrap_or_else(|_| "null".to_string())
}

fn render_object_entries(entries: &[String]) -> String {
    if entries.is_empty() {
        "null".to_string()
    } else {
        format!("{{ {} }}", entries.join(", "))
    }
}

fn js_string_literal(value: &str) -> String {
    serde_json::to_string(value).expect("string serialization should be infallible")
}

fn get_attribute_value<'a>(element: &'a HtmlElement, name: &str) -> Option<&'a str> {
    element
        .attributes
        .iter()
        .find(|attribute| attribute.name == name)
        .and_then(|attribute| attribute.value.as_deref())
}
