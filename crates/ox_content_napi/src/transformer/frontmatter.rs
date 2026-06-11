use rustc_hash::FxHashMap;

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

pub(super) fn parse_frontmatter_with_origin(source: &str) -> PreparedMarkdownSource {
    if !source.starts_with("---") {
        return source_without_frontmatter(source);
    }

    let rest = &source[3..];
    let Some(end_pos) = rest.find("\n---") else {
        return source_without_frontmatter(source);
    };

    let frontmatter_str = rest[..end_pos].trim_start_matches('\n');
    let content = rest[end_pos + 4..].trim_start_matches('\n');
    let frontmatter = serde_yaml::from_str(frontmatter_str).unwrap_or_default();

    // Keep both byte and UTF-16 offsets for the stripped body. The byte offset
    // lets Rust spans be rebased without scanning again, while the UTF-16
    // offset gives JS/LSP consumers editor-native positions.
    let source_origin = source_origin_for_content(source, content);

    PreparedMarkdownSource { content: content.to_string(), frontmatter, source_origin }
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
