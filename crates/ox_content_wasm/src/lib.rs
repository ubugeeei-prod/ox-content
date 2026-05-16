//! WebAssembly bindings for Ox Content.
//!
//! This crate provides WASM bindings for using Ox Content in browsers
//! and other WebAssembly environments.

use std::collections::HashMap;
use wasm_bindgen::prelude::*;

use ox_content_allocator::Allocator;
use ox_content_ast::{Document, Heading, Node};
use ox_content_parser::{Parser, ParserOptions};
use ox_content_renderer::HtmlRenderer;

/// Table of contents entry.
#[derive(serde::Serialize)]
pub struct TocEntry {
    pub depth: u8,
    pub text: String,
    pub slug: String,
}

/// Transform result containing HTML, frontmatter, and TOC.
#[derive(serde::Serialize)]
pub struct TransformResult {
    pub html: String,
    pub frontmatter: HashMap<String, serde_json::Value>,
    pub toc: Vec<TocEntry>,
    pub errors: Vec<String>,
}

/// Parser options.
#[wasm_bindgen]
#[derive(Default)]
pub struct WasmParserOptions {
    gfm: bool,
    footnotes: bool,
    task_lists: bool,
    tables: bool,
    strikethrough: bool,
    autolinks: bool,
    toc_max_depth: u8,
}

#[wasm_bindgen]
impl WasmParserOptions {
    #[wasm_bindgen(constructor)]
    pub fn new() -> Self {
        Self {
            gfm: false,
            footnotes: false,
            task_lists: false,
            tables: false,
            strikethrough: false,
            autolinks: false,
            toc_max_depth: 3,
        }
    }

    #[wasm_bindgen(setter)]
    pub fn set_gfm(&mut self, value: bool) {
        self.gfm = value;
    }

    #[wasm_bindgen(setter)]
    pub fn set_footnotes(&mut self, value: bool) {
        self.footnotes = value;
    }

    #[wasm_bindgen(setter = taskLists)]
    pub fn set_task_lists(&mut self, value: bool) {
        self.task_lists = value;
    }

    #[wasm_bindgen(setter)]
    pub fn set_tables(&mut self, value: bool) {
        self.tables = value;
    }

    #[wasm_bindgen(setter)]
    pub fn set_strikethrough(&mut self, value: bool) {
        self.strikethrough = value;
    }

    #[wasm_bindgen(setter)]
    pub fn set_autolinks(&mut self, value: bool) {
        self.autolinks = value;
    }

    #[wasm_bindgen(setter = tocMaxDepth)]
    pub fn set_toc_max_depth(&mut self, value: u8) {
        self.toc_max_depth = value;
    }
}

impl From<&WasmParserOptions> for ParserOptions {
    fn from(opts: &WasmParserOptions) -> Self {
        let mut options = if opts.gfm { ParserOptions::gfm() } else { ParserOptions::default() };

        options.footnotes = opts.footnotes;
        options.task_lists = opts.task_lists;
        options.tables = opts.tables;
        options.strikethrough = opts.strikethrough;
        options.autolinks = opts.autolinks;

        options
    }
}

/// Parses Markdown and renders to HTML.
#[wasm_bindgen(js_name = parseAndRender)]
pub fn parse_and_render(source: &str, options: Option<WasmParserOptions>) -> JsValue {
    let opts = options.unwrap_or_default();
    let allocator = Allocator::new();
    let parser_options = ParserOptions::from(&opts);
    let parser = Parser::with_options(&allocator, source, parser_options);

    let result = parser.parse();
    match result {
        Ok(doc) => {
            let mut renderer = HtmlRenderer::new();
            let html = renderer.render(&doc);
            serde_wasm_bindgen::to_value(&serde_json::json!({
                "html": html,
                "errors": Vec::<String>::new()
            }))
            .unwrap_or(JsValue::NULL)
        }
        Err(e) => serde_wasm_bindgen::to_value(&serde_json::json!({
            "html": "",
            "errors": [e.to_string()]
        }))
        .unwrap_or(JsValue::NULL),
    }
}

/// Transforms Markdown source into HTML, frontmatter, and TOC.
#[wasm_bindgen]
pub fn transform(source: &str, options: Option<WasmParserOptions>) -> JsValue {
    let opts = options.unwrap_or_default();
    let toc_max_depth = opts.toc_max_depth;

    // Parse frontmatter
    let (content, frontmatter) = parse_frontmatter(source);

    // Parse markdown
    let allocator = Allocator::new();
    let parser_options = ParserOptions::from(&opts);
    let parser = Parser::with_options(&allocator, &content, parser_options);

    let result = parser.parse();
    match result {
        Ok(doc) => {
            // Extract TOC from headings
            let toc = extract_toc(&doc, toc_max_depth);

            // Render to HTML
            let mut renderer = HtmlRenderer::new();
            let html = renderer.render(&doc);

            let transform_result = TransformResult { html, frontmatter, toc, errors: vec![] };

            serde_wasm_bindgen::to_value(&transform_result).unwrap_or(JsValue::NULL)
        }
        Err(e) => {
            let transform_result = TransformResult {
                html: String::new(),
                frontmatter: HashMap::new(),
                toc: vec![],
                errors: vec![e.to_string()],
            };

            serde_wasm_bindgen::to_value(&transform_result).unwrap_or(JsValue::NULL)
        }
    }
}

/// Returns the version of ox_content_wasm.
#[wasm_bindgen]
pub fn version() -> String {
    env!("CARGO_PKG_VERSION").to_string()
}

/// Parses YAML frontmatter from Markdown content.
fn parse_frontmatter(source: &str) -> (String, HashMap<String, serde_json::Value>) {
    let mut frontmatter = HashMap::new();

    if !source.starts_with("---") {
        return (source.to_string(), frontmatter);
    }

    let rest = &source[3..];
    let Some(end_pos) = rest.find("\n---") else {
        return (source.to_string(), frontmatter);
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

            let value = if value_str == "true" {
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
            };

            frontmatter.insert(key, value);
        }
    }

    (content.to_string(), frontmatter)
}

/// Extracts table of contents from document headings.
fn extract_toc(doc: &Document, max_depth: u8) -> Vec<TocEntry> {
    let mut entries = Vec::new();
    let mut slug_counts = HashMap::new();

    for node in &doc.children {
        if let Node::Heading(heading) = node {
            if heading.depth <= max_depth {
                let text = extract_heading_text(heading);
                let slug = unique_slug(slugify(&text), &mut slug_counts);
                entries.push(TocEntry { depth: heading.depth, text, slug });
            }
        }
    }

    entries
}

/// Extracts plain text from a heading node.
fn extract_heading_text(heading: &Heading) -> String {
    let mut text = String::new();
    for child in &heading.children {
        collect_text(child, &mut text);
    }
    text
}

/// Recursively collects text from nodes.
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

/// Converts text to URL-friendly slug.
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

#[cfg(test)]
mod tests {
    use ox_content_allocator::Allocator;
    use ox_content_parser::Parser;

    use super::extract_toc;

    #[test]
    fn toc_slugs_are_unique_and_match_heading_ids() {
        let allocator = Allocator::new();
        let doc = Parser::new(&allocator, "## Setup!\n## Setup?\n##").parse().unwrap();

        let toc = extract_toc(&doc, 3);

        assert_eq!(toc[0].slug, "setup");
        assert_eq!(toc[1].slug, "setup-1");
        assert_eq!(toc[2].slug, "section");
    }
}
