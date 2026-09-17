use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};

use ox_content_mdc_checker::Registry;
use tower_lsp::lsp_types::Range;
use tree_sitter::{Node, Parser};

use crate::document::TextDocumentState;

#[derive(Debug, Clone)]
pub struct LoadedRegistry {
    pub registry: Registry,
    pub path: PathBuf,
    pub locations: RegistryLocations,
}

#[derive(Debug, Clone, Default)]
pub struct RegistryLocations {
    components: BTreeMap<String, Range>,
    attributes: BTreeMap<(String, String), Range>,
}

#[derive(Debug, Clone, Copy)]
struct ByteRange {
    start: usize,
    end: usize,
}

impl RegistryLocations {
    #[must_use]
    pub fn component(&self, name: &str) -> Option<Range> {
        self.components.get(name).copied()
    }

    #[must_use]
    pub fn attribute(&self, component: &str, name: &str) -> Option<Range> {
        self.attributes.get(&(component.to_string(), name.to_string())).copied()
    }
}

pub fn load(path: &Path) -> Option<LoadedRegistry> {
    let registry = Registry::from_path(path).ok().flatten()?;
    let source = fs::read_to_string(path).ok()?;
    let locations = locations_for_source(&source);
    Some(LoadedRegistry { registry, path: path.to_path_buf(), locations })
}

#[must_use]
pub fn locations_for_source(source: &str) -> RegistryLocations {
    let mut parser = Parser::new();
    if parser.set_language(&tree_sitter_json::LANGUAGE.into()).is_err() {
        return RegistryLocations::default();
    }
    let Some(tree) = parser.parse(source, None) else {
        return RegistryLocations::default();
    };
    let document = TextDocumentState::new(source.to_string());
    let bytes = source.as_bytes();
    let root = tree.root_node();
    let Some(object) = first_named_child(root, "object") else {
        return RegistryLocations::default();
    };
    let Some(components) =
        pair_value(object, bytes, "components").filter(|node| node.kind() == "array")
    else {
        return RegistryLocations::default();
    };

    let mut locations = RegistryLocations::default();
    let mut component_cursor = components.walk();
    for component in
        components.named_children(&mut component_cursor).filter(|node| node.kind() == "object")
    {
        let Some((component_name, component_range)) = string_pair(component, bytes, "name") else {
            continue;
        };
        locations.components.insert(
            component_name.clone(),
            document.range_from_offsets(component_range.start, component_range.end),
        );

        let Some(attributes) =
            pair_value(component, bytes, "attributes").filter(|node| node.kind() == "array")
        else {
            continue;
        };
        let mut attribute_cursor = attributes.walk();
        for attribute in
            attributes.named_children(&mut attribute_cursor).filter(|node| node.kind() == "object")
        {
            let Some((attribute_name, attribute_range)) = string_pair(attribute, bytes, "name")
            else {
                continue;
            };
            locations.attributes.insert(
                (component_name.clone(), attribute_name),
                document.range_from_offsets(attribute_range.start, attribute_range.end),
            );
        }
    }

    locations
}

fn first_named_child<'a>(node: Node<'a>, kind: &str) -> Option<Node<'a>> {
    let mut cursor = node.walk();
    node.named_children(&mut cursor).find(|child| child.kind() == kind)
}

fn pair_value<'a>(object: Node<'a>, bytes: &[u8], key: &str) -> Option<Node<'a>> {
    let mut cursor = object.walk();
    for pair in object.named_children(&mut cursor).filter(|node| node.kind() == "pair") {
        let key_node = pair.child_by_field_name("key")?;
        if decode_json_string(key_node, bytes).as_deref() == Some(key) {
            return pair.child_by_field_name("value");
        }
    }
    None
}

fn string_pair(object: Node<'_>, bytes: &[u8], key: &str) -> Option<(String, ByteRange)> {
    let value = pair_value(object, bytes, key)?;
    if value.kind() != "string" {
        return None;
    }
    let decoded = decode_json_string(value, bytes)?;
    let range_node = string_content_node(value).unwrap_or(value);
    Some((decoded, ByteRange { start: range_node.start_byte(), end: range_node.end_byte() }))
}

fn string_content_node(string: Node<'_>) -> Option<Node<'_>> {
    let mut cursor = string.walk();
    string.named_children(&mut cursor).find(|child| child.kind() == "string_content")
}

fn decode_json_string(node: Node<'_>, bytes: &[u8]) -> Option<String> {
    let text = node.utf8_text(bytes).ok()?;
    serde_json::from_str::<String>(text).ok()
}

#[cfg(test)]
mod tests;
