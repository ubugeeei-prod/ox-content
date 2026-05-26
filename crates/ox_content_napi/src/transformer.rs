use std::collections::HashMap;

use napi::bindgen_prelude::Uint8Array;
use ox_content_allocator::Allocator;
use ox_content_ast::{Document, Heading, Node};
use ox_content_parser::{ParseError, Parser, ParserOptions};
use ox_content_renderer::{HtmlRenderer, HtmlRendererOptions};

use crate::{
    mdast_raw::{
        self, MDAST_SECTION_CONTENT, MDAST_SECTION_FRONTMATTER, MDAST_SECTION_SOURCE_ORIGIN,
    },
    transfer::{TransferBufferBuilder, TransferPayloadKind},
    JsTransformOptions, TocEntry, TransformResult,
};

const PREPARED_SOURCE_PAYLOAD_VERSION: u32 = 1;
const PREPARED_SOURCE_SECTION_CONTENT: u32 = 1;
const PREPARED_SOURCE_SECTION_FRONTMATTER: u32 = 2;
const PREPARED_SOURCE_SECTION_SOURCE_ORIGIN: u32 = 3;

pub struct MarkdownTransformer {
    frontmatter: bool,
    toc_max_depth: u8,
    parser_options: ParserOptions,
    renderer_options: HtmlRendererOptions,
}

pub struct PreparedMarkdownSource {
    pub content: String,
    pub frontmatter: HashMap<String, serde_json::Value>,
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
    fn to_bytes(self) -> Vec<u8> {
        let mut bytes = Vec::with_capacity(16);
        bytes.extend_from_slice(&self.byte_offset.to_le_bytes());
        bytes.extend_from_slice(&self.offset.to_le_bytes());
        bytes.extend_from_slice(&self.line.to_le_bytes());
        bytes.extend_from_slice(&self.column.to_le_bytes());
        bytes
    }
}

impl MarkdownTransformer {
    pub(crate) fn with_frontmatter(frontmatter: bool) -> Self {
        Self {
            frontmatter,
            toc_max_depth: 3,
            parser_options: ParserOptions::default(),
            renderer_options: HtmlRendererOptions::new(),
        }
    }

    pub(crate) fn from_options(options: &JsTransformOptions) -> Self {
        Self {
            frontmatter: options.frontmatter.unwrap_or(true),
            toc_max_depth: options.toc_max_depth.unwrap_or(3),
            parser_options: transform_options_to_parser_options(options),
            renderer_options: transform_options_to_renderer_options(options),
        }
    }

    pub(crate) fn transform(&self, source: &str) -> TransformResult {
        let prepared = self.prepare_source(source);
        let allocator = Allocator::new();
        let parse_result = self.parse_document(&allocator, &prepared.content);

        match parse_result {
            Ok(document) => TransformResult {
                html: self.render_html(&document),
                frontmatter: serde_json::to_string(&prepared.frontmatter)
                    .unwrap_or_else(|_| "{}".to_string()),
                toc: extract_toc(&document, self.toc_max_depth),
                errors: vec![],
            },
            Err(error) => TransformResult {
                html: String::new(),
                frontmatter: "{}".to_string(),
                toc: vec![],
                errors: vec![error.to_string()],
            },
        }
    }

    pub(crate) fn transform_mdast_raw(&self, source: &str) -> napi::Result<Uint8Array> {
        let prepared = self.prepare_source(source);
        let content_bytes = prepared.content.as_bytes().to_vec();
        let frontmatter_bytes = serde_json::to_vec(&prepared.frontmatter)
            .map_err(|error| napi::Error::from_reason(error.to_string()))?;
        let allocator = Allocator::new();
        let document = self
            .parse_document(&allocator, &prepared.content)
            .map_err(|error| napi::Error::from_reason(error.to_string()))?;

        mdast_raw::to_mdast_raw_with_sections(
            &document,
            vec![
                (MDAST_SECTION_CONTENT, content_bytes),
                (MDAST_SECTION_FRONTMATTER, frontmatter_bytes),
                (MDAST_SECTION_SOURCE_ORIGIN, prepared.source_origin.to_bytes()),
            ],
        )
    }

    pub(crate) fn prepare_source_raw(&self, source: &str) -> napi::Result<Uint8Array> {
        let prepared = self.prepare_source(source);
        let frontmatter_bytes = serde_json::to_vec(&prepared.frontmatter)
            .map_err(|error| napi::Error::from_reason(error.to_string()))?;
        let mut builder = TransferBufferBuilder::new(
            TransferPayloadKind::PreparedSource,
            PREPARED_SOURCE_PAYLOAD_VERSION,
            0,
        );
        builder.push_section(PREPARED_SOURCE_SECTION_CONTENT, prepared.content.into_bytes());
        builder.push_section(PREPARED_SOURCE_SECTION_FRONTMATTER, frontmatter_bytes);
        builder
            .push_section(PREPARED_SOURCE_SECTION_SOURCE_ORIGIN, prepared.source_origin.to_bytes());
        builder.finish()
    }

    pub(crate) fn parse_document<'a>(
        &self,
        allocator: &'a Allocator,
        source: &'a str,
    ) -> Result<Document<'a>, ParseError> {
        Parser::with_options(allocator, source, self.parser_options.clone()).parse()
    }

    pub(crate) fn render_html(&self, document: &Document<'_>) -> String {
        let mut renderer = HtmlRenderer::with_options(self.renderer_options.clone());
        renderer.render(document)
    }

    pub(super) fn prepare_source(&self, source: &str) -> PreparedMarkdownSource {
        if self.frontmatter {
            parse_frontmatter_with_origin(source)
        } else {
            PreparedMarkdownSource {
                content: source.to_string(),
                frontmatter: HashMap::new(),
                source_origin: SourceOrigin { line: 1, column: 1, ..SourceOrigin::default() },
            }
        }
    }
}

pub fn parse_frontmatter(source: &str) -> (String, HashMap<String, serde_json::Value>) {
    let prepared = parse_frontmatter_with_origin(source);
    (prepared.content, prepared.frontmatter)
}

fn parse_frontmatter_with_origin(source: &str) -> PreparedMarkdownSource {
    let mut frontmatter = HashMap::new();

    if !source.starts_with("---") {
        return PreparedMarkdownSource {
            content: source.to_string(),
            frontmatter,
            source_origin: SourceOrigin { line: 1, column: 1, ..SourceOrigin::default() },
        };
    }

    let rest = &source[3..];
    let Some(end_pos) = rest.find("\n---") else {
        return PreparedMarkdownSource {
            content: source.to_string(),
            frontmatter,
            source_origin: SourceOrigin { line: 1, column: 1, ..SourceOrigin::default() },
        };
    };

    let frontmatter_str = rest[..end_pos].trim_start_matches('\n');
    let content = rest[end_pos + 4..].trim_start_matches('\n');
    frontmatter = serde_yaml::from_str(frontmatter_str).unwrap_or_default();

    let source_origin = source_origin_for_content(source, content);

    PreparedMarkdownSource { content: content.to_string(), frontmatter, source_origin }
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

fn extract_toc(doc: &Document, max_depth: u8) -> Vec<TocEntry> {
    let mut entries = Vec::new();
    let mut slug_counts = HashMap::new();

    for node in &doc.children {
        if let Node::Heading(heading) = node {
            if heading.depth <= max_depth {
                let text = extract_heading_text(heading);
                let slug = unique_slug(slugify(&text), &mut slug_counts);
                push_nested_toc_entry(
                    &mut entries,
                    TocEntry { depth: heading.depth, text, slug, children: Vec::new() },
                );
            }
        }
    }

    entries
}

fn push_nested_toc_entry(entries: &mut Vec<TocEntry>, entry: TocEntry) {
    if let Some(last) = entries.last_mut() {
        if last.depth < entry.depth {
            push_nested_toc_entry(&mut last.children, entry);
            return;
        }
    }

    entries.push(entry);
}

fn extract_heading_text(heading: &Heading) -> String {
    let mut text = String::new();
    for child in &heading.children {
        collect_text(child, &mut text);
    }
    text
}

fn collect_text(node: &Node, text: &mut String) {
    match node {
        Node::Text(t) => text.push_str(t.value),
        Node::Emphasis(e) => {
            for child in &e.children {
                collect_text(child, text);
            }
        }
        Node::Strong(s) => {
            for child in &s.children {
                collect_text(child, text);
            }
        }
        Node::InlineCode(c) => text.push_str(c.value),
        Node::Delete(d) => {
            for child in &d.children {
                collect_text(child, text);
            }
        }
        Node::Link(l) => {
            for child in &l.children {
                collect_text(child, text);
            }
        }
        _ => {}
    }
}

fn slugify(text: &str) -> String {
    text.to_lowercase()
        .chars()
        .map(|c| if c.is_alphanumeric() || c == ' ' || c == '-' { c } else { ' ' })
        .collect::<String>()
        .split_whitespace()
        .collect::<Vec<_>>()
        .join("-")
}

fn unique_slug(slug: String, counts: &mut HashMap<String, usize>) -> String {
    let slug = if slug.is_empty() { "section".to_string() } else { slug };
    let count = counts.entry(slug.clone()).or_insert(0);
    let unique = if *count == 0 { slug } else { format!("{slug}-{count}") };
    *count += 1;
    unique
}

fn transform_options_to_parser_options(opts: &JsTransformOptions) -> ParserOptions {
    let mut options =
        if opts.gfm.unwrap_or(false) { ParserOptions::gfm() } else { ParserOptions::default() };

    if let Some(v) = opts.footnotes {
        options.footnotes = v;
    }
    if let Some(v) = opts.task_lists {
        options.task_lists = v;
    }
    if let Some(v) = opts.tables {
        options.tables = v;
    }
    if let Some(v) = opts.strikethrough {
        options.strikethrough = v;
    }
    if let Some(v) = opts.autolinks {
        options.autolinks = v;
    }

    options
}

fn transform_options_to_renderer_options(opts: &JsTransformOptions) -> HtmlRendererOptions {
    let mut options = HtmlRendererOptions::new();

    options.toc_max_depth = opts.toc_max_depth.unwrap_or(options.toc_max_depth);

    if let Some(v) = opts.convert_md_links {
        options.convert_md_links = v;
    }

    if let Some(v) = &opts.base_url {
        options.base_url.clone_from(v);
    }

    if let Some(v) = &opts.source_path {
        options.source_path.clone_from(v);
    }
    if let Some(v) = opts.code_annotations {
        options.code_annotations = v;
    }
    if let Some(v) = &opts.code_annotation_meta_key {
        options.code_annotation_meta_key.clone_from(v);
    }
    if let Some(v) = &opts.code_annotation_syntax {
        options.code_annotation_syntax = match v.as_str() {
            "vitepress" => ox_content_renderer::CodeAnnotationSyntax::VitePress,
            "both" => ox_content_renderer::CodeAnnotationSyntax::Both,
            _ => ox_content_renderer::CodeAnnotationSyntax::Attribute,
        };
    }
    if let Some(v) = opts.code_annotation_default_line_numbers {
        options.code_annotation_default_line_numbers = v;
    }

    options
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn transforms_markdown_with_frontmatter_and_toc() {
        let transformer = MarkdownTransformer::from_options(&JsTransformOptions {
            gfm: Some(true),
            toc_max_depth: Some(2),
            ..Default::default()
        });
        let result =
            transformer.transform("---\ntitle: Example\n---\n# Hello\n\nThis is a paragraph.");

        assert!(result.errors.is_empty());
        assert!(result.html.contains("<h1 id=\"hello\">Hello</h1>"));
        assert!(result.frontmatter.contains("\"title\":\"Example\""));
        assert_eq!(result.toc.len(), 1);
        assert_eq!(result.toc[0].slug, "hello");
    }

    #[test]
    fn leaves_non_frontmatter_documents_untouched() {
        let (content, frontmatter) = parse_frontmatter("# Hello");

        assert_eq!(content, "# Hello");
        assert!(frontmatter.is_empty());
    }

    #[test]
    fn skips_frontmatter_extraction_when_disabled() {
        let source = "---\ntitle: Example\n---\n# Hello";
        let transformer = MarkdownTransformer::from_options(&JsTransformOptions {
            frontmatter: Some(false),
            ..Default::default()
        });
        let prepared = transformer.prepare_source(source);

        assert_eq!(prepared.content, source);
        assert!(prepared.frontmatter.is_empty());
    }

    #[test]
    fn tracks_source_origin_after_frontmatter() {
        let prepared =
            parse_frontmatter_with_origin("---\ntitle: こんにちは\nemoji: 😀\n---\n# Hello");

        assert_eq!(prepared.content, "# Hello");
        assert_eq!(
            prepared.source_origin,
            SourceOrigin { byte_offset: 43, offset: 31, line: 5, column: 1 }
        );
    }

    #[test]
    fn toc_slugs_are_unique_and_match_heading_ids() {
        let allocator = Allocator::new();
        let doc = Parser::new(&allocator, "## Setup!\n## Setup?\n##").parse().unwrap();

        let toc = extract_toc(&doc, 3);

        assert_eq!(toc[0].slug, "setup");
        assert_eq!(toc[1].slug, "setup-1");
        assert_eq!(toc[2].slug, "section");
    }

    #[test]
    fn toc_entries_are_nested_in_rust() {
        let allocator = Allocator::new();
        let doc =
            Parser::new(&allocator, "## Guide\n### Install\n#### CLI\n## API").parse().unwrap();

        let toc = extract_toc(&doc, 4);

        assert_eq!(toc.len(), 2);
        assert_eq!(toc[0].slug, "guide");
        assert_eq!(toc[0].children[0].slug, "install");
        assert_eq!(toc[0].children[0].children[0].slug, "cli");
        assert_eq!(toc[1].slug, "api");
    }
}
