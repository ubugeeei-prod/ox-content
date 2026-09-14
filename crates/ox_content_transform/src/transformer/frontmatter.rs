use rustc_hash::FxHashMap;
use serde_json::{Map, Value};
use std::{
    collections::{BTreeMap, HashMap},
    hash::BuildHasher,
};

pub struct PreparedMarkdownSource {
    pub content: String,
    pub frontmatter: FxHashMap<String, serde_json::Value>,
    pub source_origin: SourceOrigin,
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct SourceOrigin {
    pub byte_offset: u32,
    pub offset: u32,
    pub line: u32,
    pub column: u32,
}

impl SourceOrigin {
    pub(super) fn to_bytes(self) -> Vec<u8> {
        let mut bytes = Vec::with_capacity(16);
        bytes.extend_from_slice(&self.byte_offset.to_le_bytes());
        bytes.extend_from_slice(&self.offset.to_le_bytes());
        bytes.extend_from_slice(&self.line.to_le_bytes());
        bytes.extend_from_slice(&self.column.to_le_bytes());
        bytes
    }
}

pub fn parse_frontmatter(source: &str) -> (String, FxHashMap<String, serde_json::Value>) {
    let prepared = parse_frontmatter_with_origin(source);
    (prepared.content, prepared.frontmatter)
}

pub fn parse_frontmatter_ordered(source: &str) -> (String, Map<String, Value>) {
    let Some((frontmatter_str, content)) = split_frontmatter(source) else {
        return (source.to_string(), Map::new());
    };

    let frontmatter = serde_yaml::from_str(frontmatter_str).unwrap_or_default();
    (content.to_string(), frontmatter)
}

pub fn stringify_frontmatter<S: BuildHasher>(
    frontmatter: &HashMap<String, serde_json::Value, S>,
    content: &str,
) -> Result<String, serde_yaml::Error> {
    if frontmatter.is_empty() {
        return Ok(content.to_string());
    }

    let sorted_frontmatter = frontmatter.iter().collect::<BTreeMap<_, _>>();
    let yaml = serde_yaml::to_string(&sorted_frontmatter)?;
    let yaml = yaml.trim_end();

    Ok(format!("---\n{yaml}\n---\n{content}"))
}

pub fn stringify_ordered_frontmatter(
    frontmatter: &Map<String, Value>,
    content: &str,
) -> Result<String, serde_yaml::Error> {
    stringify_frontmatter_entries(
        frontmatter.iter().map(|(key, value)| (key.as_str(), value)),
        content,
    )
}

fn stringify_frontmatter_entries<'a, I, K>(
    frontmatter: I,
    content: &str,
) -> Result<String, serde_yaml::Error>
where
    I: IntoIterator<Item = (K, &'a Value)>,
    K: AsRef<str>,
{
    let mut yaml_frontmatter = serde_yaml::Mapping::new();

    for (key, value) in frontmatter {
        yaml_frontmatter.insert(
            serde_yaml::Value::String(key.as_ref().to_string()),
            serde_yaml::to_value(value)?,
        );
    }

    if yaml_frontmatter.is_empty() {
        return Ok(content.to_string());
    }

    let yaml = serde_yaml::to_string(&yaml_frontmatter)?;
    let yaml = yaml.trim_end();

    Ok(format!("---\n{yaml}\n---\n{content}"))
}

pub(super) fn parse_frontmatter_with_origin(source: &str) -> PreparedMarkdownSource {
    let Some((frontmatter_str, content)) = split_frontmatter(source) else {
        return source_without_frontmatter(source);
    };

    let frontmatter = serde_yaml::from_str(frontmatter_str).unwrap_or_default();

    // Keep both byte and UTF-16 offsets for the stripped body. The byte offset
    // lets Rust spans be rebased without scanning again, while the UTF-16
    // offset gives JS/LSP consumers editor-native positions.
    let source_origin = source_origin_for_content(source, content);

    PreparedMarkdownSource { content: content.to_string(), frontmatter, source_origin }
}

fn split_frontmatter(source: &str) -> Option<(&str, &str)> {
    if !source.starts_with("---") {
        return None;
    }

    let rest = &source[3..];
    let end_pos = rest.find("\n---")?;

    let frontmatter_str = rest[..end_pos].trim_start_matches('\n');
    let content = strip_frontmatter_content_separator(&rest[end_pos + 4..]);

    Some((frontmatter_str, content))
}

fn strip_frontmatter_content_separator(content: &str) -> &str {
    content.strip_prefix("\r\n").or_else(|| content.strip_prefix('\n')).unwrap_or(content)
}

pub(super) fn source_without_frontmatter(source: &str) -> PreparedMarkdownSource {
    PreparedMarkdownSource {
        content: source.to_string(),
        frontmatter: FxHashMap::default(),
        source_origin: SourceOrigin { line: 1, column: 1, ..SourceOrigin::default() },
    }
}

fn source_origin_for_content(source: &str, content: &str) -> SourceOrigin {
    let prefix_len = source.len().saturating_sub(content.len());
    let prefix = &source[..prefix_len];
    let mut origin = SourceOrigin { line: 1, column: 1, ..SourceOrigin::default() };

    for character in prefix.chars() {
        origin.byte_offset += character.len_utf8() as u32;
        origin.offset += character.len_utf16() as u32;

        if character == '\n' {
            origin.line += 1;
            origin.column = 1;
        } else {
            origin.column += 1;
        }
    }

    origin
}
