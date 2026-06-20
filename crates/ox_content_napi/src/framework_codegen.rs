use std::collections::HashMap;

use napi_derive::napi;

#[napi(object)]
pub struct JsFrameworkComponentIsland {
    pub name: String,
    pub props: HashMap<String, serde_json::Value>,
    pub id: String,
    pub content: Option<String>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum FrameworkCodegenTarget {
    React,
    Vue,
}

impl FrameworkCodegenTarget {
    fn parse(value: &str) -> napi::Result<Self> {
        match value {
            "react" => Ok(Self::React),
            "vue" => Ok(Self::Vue),
            _ => Err(napi::Error::from_reason(format!(
                "Unsupported framework component render target: {value}"
            ))),
        }
    }
}

#[derive(Debug, Eq, PartialEq)]
enum HtmlNode {
    Element(HtmlElement),
    Text(String),
}

#[derive(Debug, Eq, PartialEq)]
struct HtmlElement {
    tag_name: String,
    attributes: Vec<HtmlAttribute>,
    children: Vec<HtmlNode>,
}

#[derive(Debug, Eq, PartialEq)]
struct HtmlAttribute {
    name: String,
    value: Option<String>,
}

#[derive(Debug)]
struct OpenElement {
    tag_name: String,
    attributes: Vec<HtmlAttribute>,
    children: Vec<HtmlNode>,
}

impl OpenElement {
    fn into_node(self) -> HtmlNode {
        HtmlNode::Element(HtmlElement {
            tag_name: self.tag_name,
            attributes: self.attributes,
            children: self.children,
        })
    }
}

/// Renders already-produced Markdown HTML into framework-native component code.
///
/// The `target` argument currently accepts `"react"` and `"vue"`. React output
/// uses `createElement(...)`; Vue output uses `h(...)`.
#[napi(js_name = "renderFrameworkComponentCode")]
pub fn render_framework_component_code(
    html: String,
    target: String,
    islands: Option<Vec<JsFrameworkComponentIsland>>,
) -> napi::Result<String> {
    let target = FrameworkCodegenTarget::parse(&target)?;
    let nodes = HtmlFragmentParser::new(&html).parse();
    let islands = islands.unwrap_or_default();
    Ok(FrameworkCodegen { target, islands: &islands }.render_root(&nodes))
}

/// Escapes Svelte expression delimiters before emitting static compiled markup.
#[napi(js_name = "escapeSvelteMarkup")]
pub fn escape_svelte_markup(html: String) -> String {
    if !html.contains('{') && !html.contains('}') {
        return html;
    }

    let mut output = String::with_capacity(html.len());
    for ch in html.chars() {
        match ch {
            '{' => output.push_str("&#123;"),
            '}' => output.push_str("&#125;"),
            _ => output.push(ch),
        }
    }
    output
}

struct HtmlFragmentParser<'a> {
    html: &'a str,
    pos: usize,
}

impl<'a> HtmlFragmentParser<'a> {
    fn new(html: &'a str) -> Self {
        Self { html, pos: 0 }
    }

    fn parse(mut self) -> Vec<HtmlNode> {
        let mut root = Vec::new();
        let mut stack = Vec::new();

        while self.pos < self.html.len() {
            let Some(tag_start) = find_byte(self.html, self.pos, b'<') else {
                push_text(&mut root, &mut stack, &self.html[self.pos..]);
                self.pos = self.html.len();
                break;
            };

            if tag_start > self.pos {
                push_text(&mut root, &mut stack, &self.html[self.pos..tag_start]);
            }

            if self.html[tag_start..].starts_with("<!--") {
                self.pos = find_comment_end(self.html, tag_start).unwrap_or(self.html.len());
                continue;
            }

            let Some(tag_end) = find_tag_end(self.html, tag_start) else {
                push_text(&mut root, &mut stack, &self.html[tag_start..]);
                self.pos = self.html.len();
                break;
            };

            let raw_tag = &self.html[tag_start + 1..tag_end];
            if raw_tag.trim_start().starts_with(['!', '?']) {
                self.pos = tag_end + 1;
                continue;
            }

            if let Some(tag_name) = parse_end_tag(raw_tag) {
                close_element(&mut root, &mut stack, &tag_name);
                self.pos = tag_end + 1;
                continue;
            }

            if let Some((tag_name, attributes, self_closing)) = parse_start_tag(raw_tag) {
                let is_void = is_void_element(&tag_name);
                let open = OpenElement { tag_name, attributes, children: Vec::new() };
                if self_closing || is_void {
                    append_node(&mut root, &mut stack, open.into_node());
                } else {
                    stack.push(open);
                }
                self.pos = tag_end + 1;
                continue;
            }

            push_text(&mut root, &mut stack, &self.html[tag_start..=tag_end]);
            self.pos = tag_end + 1;
        }

        while let Some(open) = stack.pop() {
            append_node(&mut root, &mut stack, open.into_node());
        }

        root
    }
}

fn find_byte(haystack: &str, from: usize, byte: u8) -> Option<usize> {
    memchr::memchr(byte, &haystack.as_bytes()[from..]).map(|index| from + index)
}

fn find_comment_end(html: &str, start: usize) -> Option<usize> {
    html[start + 4..].find("-->").map(|offset| start + 4 + offset + 3)
}

fn find_tag_end(html: &str, start: usize) -> Option<usize> {
    let bytes = html.as_bytes();
    let mut quote = None;

    for (index, byte) in bytes.iter().enumerate().skip(start + 1) {
        match (quote, *byte) {
            (Some(current), value) if value == current => quote = None,
            (Some(_), _) => {}
            (None, b'\'' | b'"') => quote = Some(*byte),
            (None, b'>') => return Some(index),
            _ => {}
        }
    }

    None
}

fn parse_end_tag(raw_tag: &str) -> Option<String> {
    let raw_tag = raw_tag.trim_start();
    let rest = raw_tag.strip_prefix('/')?.trim_start();
    let end = rest
        .bytes()
        .position(|byte| byte.is_ascii_whitespace() || byte == b'/')
        .unwrap_or(rest.len());
    if end == 0 {
        None
    } else {
        Some(rest[..end].to_string())
    }
}

fn parse_start_tag(raw_tag: &str) -> Option<(String, Vec<HtmlAttribute>, bool)> {
    let bytes = raw_tag.as_bytes();
    let mut cursor = skip_ascii_whitespace(bytes, 0);
    if cursor >= bytes.len() || matches!(bytes[cursor], b'/' | b'!' | b'?') {
        return None;
    }

    let name_start = cursor;
    while cursor < bytes.len()
        && !bytes[cursor].is_ascii_whitespace()
        && !matches!(bytes[cursor], b'/' | b'=')
    {
        cursor += 1;
    }
    if cursor == name_start {
        return None;
    }

    let tag_name = raw_tag[name_start..cursor].to_string();
    let mut attributes = Vec::new();
    let mut self_closing = false;

    while cursor < bytes.len() {
        cursor = skip_ascii_whitespace(bytes, cursor);
        if cursor >= bytes.len() {
            break;
        }

        if bytes[cursor] == b'/' {
            self_closing = true;
            cursor += 1;
            continue;
        }

        let attr_start = cursor;
        while cursor < bytes.len()
            && !bytes[cursor].is_ascii_whitespace()
            && !matches!(bytes[cursor], b'=' | b'/')
        {
            cursor += 1;
        }
        if cursor == attr_start {
            cursor += 1;
            continue;
        }

        let name = raw_tag[attr_start..cursor].to_string();
        cursor = skip_ascii_whitespace(bytes, cursor);
        let mut value = None;

        if cursor < bytes.len() && bytes[cursor] == b'=' {
            cursor += 1;
            cursor = skip_ascii_whitespace(bytes, cursor);

            if cursor < bytes.len() && matches!(bytes[cursor], b'\'' | b'"') {
                let quote = bytes[cursor];
                cursor += 1;
                let value_start = cursor;
                while cursor < bytes.len() && bytes[cursor] != quote {
                    cursor += 1;
                }
                value = Some(decode_html_entities(&raw_tag[value_start..cursor]));
                if cursor < bytes.len() {
                    cursor += 1;
                }
            } else {
                let value_start = cursor;
                while cursor < bytes.len()
                    && !bytes[cursor].is_ascii_whitespace()
                    && bytes[cursor] != b'/'
                {
                    cursor += 1;
                }
                value = Some(decode_html_entities(&raw_tag[value_start..cursor]));
            }
        }

        attributes.push(HtmlAttribute { name, value });
    }

    Some((tag_name, attributes, self_closing))
}

fn skip_ascii_whitespace(bytes: &[u8], mut cursor: usize) -> usize {
    while cursor < bytes.len() && bytes[cursor].is_ascii_whitespace() {
        cursor += 1;
    }
    cursor
}

fn push_text(root: &mut Vec<HtmlNode>, stack: &mut [OpenElement], value: &str) {
    if !value.is_empty() {
        append_node(root, stack, HtmlNode::Text(decode_html_entities(value)));
    }
}

fn append_node(root: &mut Vec<HtmlNode>, stack: &mut [OpenElement], node: HtmlNode) {
    if let Some(parent) = stack.last_mut() {
        parent.children.push(node);
    } else {
        root.push(node);
    }
}

fn close_element(root: &mut Vec<HtmlNode>, stack: &mut Vec<OpenElement>, tag_name: &str) {
    let Some(position) =
        stack.iter().rposition(|element| element.tag_name.eq_ignore_ascii_case(tag_name))
    else {
        return;
    };

    while stack.len() > position {
        let open = stack.pop().expect("stack length checked above");
        append_node(root, stack, open.into_node());
    }
}

fn is_void_element(tag_name: &str) -> bool {
    matches!(
        tag_name.to_ascii_lowercase().as_str(),
        "area"
            | "base"
            | "br"
            | "col"
            | "embed"
            | "hr"
            | "img"
            | "input"
            | "link"
            | "meta"
            | "param"
            | "source"
            | "track"
            | "wbr"
    )
}

struct FrameworkCodegen<'a> {
    target: FrameworkCodegenTarget,
    islands: &'a [JsFrameworkComponentIsland],
}

impl FrameworkCodegen<'_> {
    fn render_root(&self, nodes: &[HtmlNode]) -> String {
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

fn decode_html_entities(value: &str) -> String {
    if !value.contains('&') {
        return value.to_string();
    }

    let mut output = String::with_capacity(value.len());
    let mut cursor = 0;
    while let Some(relative_amp) = value[cursor..].find('&') {
        let amp = cursor + relative_amp;
        output.push_str(&value[cursor..amp]);

        let Some(relative_semicolon) = value[amp + 1..].find(';') else {
            output.push('&');
            cursor = amp + 1;
            continue;
        };

        let semicolon = amp + 1 + relative_semicolon;
        let entity = &value[amp + 1..semicolon];
        if entity.len() > 32 {
            output.push('&');
            cursor = amp + 1;
            continue;
        }

        if let Some(decoded) = decode_html_entity(entity) {
            output.push(decoded);
            cursor = semicolon + 1;
        } else {
            output.push('&');
            cursor = amp + 1;
        }
    }
    output.push_str(&value[cursor..]);
    output
}

fn decode_html_entity(entity: &str) -> Option<char> {
    match entity {
        "amp" => Some('&'),
        "lt" => Some('<'),
        "gt" => Some('>'),
        "quot" => Some('"'),
        "apos" => Some('\''),
        "nbsp" => Some('\u{00a0}'),
        value if value.starts_with("#x") || value.starts_with("#X") => {
            u32::from_str_radix(&value[2..], 16).ok().and_then(char::from_u32)
        }
        value if value.starts_with('#') => value[1..].parse::<u32>().ok().and_then(char::from_u32),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::*;

    #[test]
    fn parses_nested_fragment_and_decodes_entities() {
        let nodes = HtmlFragmentParser::new("<p title=\"Tom &amp; Jerry\">A &lt; B</p>").parse();
        assert_eq!(
            nodes,
            vec![HtmlNode::Element(HtmlElement {
                tag_name: "p".to_string(),
                attributes: vec![HtmlAttribute {
                    name: "title".to_string(),
                    value: Some("Tom & Jerry".to_string())
                }],
                children: vec![HtmlNode::Text("A < B".to_string())],
            })]
        );
    }

    #[test]
    fn renders_react_create_element_code() {
        let code = render_framework_component_code(
            [
                r#"<section class="lead" for="name" data-id="42" aria-label="Intro">"#,
                r#"<p style="font-weight: bold; --brand: red;">Hello <strong>world</strong></p>"#,
                "</section>",
            ]
            .join(""),
            "react".to_string(),
            None,
        )
        .unwrap();

        assert!(code.contains("createElement('div', { className: 'ox-content' }"));
        assert!(code.contains(r#""className": "lead""#));
        assert!(code.contains(r#""htmlFor": "name""#));
        assert!(code.contains(r#""data-id": "42""#));
        assert!(code.contains(r#""aria-label": "Intro""#));
        assert!(code.contains(r#""style": { "fontWeight": "bold", "--brand": "red" }"#));
        assert!(code.contains(r#"createElement("strong", null, "world")"#));
    }

    #[test]
    fn renders_vue_h_code() {
        let code = render_framework_component_code(
            r#"<label class="field" for="name"><span>Name</span><input disabled type="text"></label>"#
                .to_string(),
            "vue".to_string(),
            None,
        )
        .unwrap();

        assert!(code.contains("h('div', { class: 'ox-content' }"));
        assert!(code.contains(r#"h("label", { "class": "field", "for": "name" }"#));
        assert!(code.contains(r#"h("span", null, "Name")"#));
        assert!(code.contains(r#"h("input", { "disabled": true, "type": "text" })"#));
    }

    #[test]
    fn renders_framework_islands() {
        let mut props = HashMap::new();
        props.insert("tone".to_string(), json!("info"));
        props.insert("active".to_string(), json!(true));
        let islands = vec![JsFrameworkComponentIsland {
            id: "ox-island-0".to_string(),
            name: "Alert".to_string(),
            props,
            content: Some("Read docs".to_string()),
        }];

        let code = render_framework_component_code(
            r#"<p>Before</p><div data-ox-island="Alert" data-ox-id="ox-island-0"></div>"#
                .to_string(),
            "react".to_string(),
            Some(islands),
        )
        .unwrap();

        assert!(
            code.contains(
                r#"createElement(Alert, { "active": true, "tone": "info" }, "Read docs")"#
            ) || code.contains(
                r#"createElement(Alert, { "tone": "info", "active": true }, "Read docs")"#
            )
        );
    }

    #[test]
    fn escapes_svelte_expression_delimiters() {
        assert_eq!(
            escape_svelte_markup("<p>{count} and }</p>".to_string()),
            "<p>&#123;count&#125; and &#125;</p>"
        );
    }
}
