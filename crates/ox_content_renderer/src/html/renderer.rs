//! Stateful HTML renderer and public render entry point.
//!
//! The renderer struct lives here, while specialized child modules implement output
//! helpers, block and inline visitors, URL rewriting, callouts, and code block details.
//! This keeps the public constructor/render path visible without forcing unrelated
//! rendering rules into one large file.

mod blocks;
mod callout;
mod code_block;
mod inlines;
mod links;
mod visit;
mod write;

use ox_content_ast::{Document, Node};
use rustc_hash::FxHashMap;

use super::autolink::FirstByteIndex;
use super::escape::{write_escaped_into, write_url_escaped_into};
use super::options::HtmlRendererOptions;
use super::toc::{collect_inline_toc_entries, scan_document_for_render, InlineTocEntry};
use crate::render::{RenderResult, Renderer};

/// Stateful HTML renderer for Markdown AST documents.
///
/// A renderer instance owns reusable buffers for heading IDs, inline table-of-contents
/// entries, and autolink scanning. Reusing the same instance across renders avoids a
/// set of hot-path allocations while keeping the public API as simple as
/// [`HtmlRenderer::render`].
pub struct HtmlRenderer {
    options: HtmlRendererOptions,
    output: String,
    heading_id_counts: FxHashMap<String, usize>,
    toc_entries: Vec<InlineTocEntry>,
    /// Whether the document being rendered contains at least one
    /// `[[toc]]` directive paragraph. Cached at `render()` entry so each
    /// `visit_paragraph` can skip the marker check entirely when no
    /// directive exists (the common case). Kept separate from
    /// `toc_entries.is_empty()` because a document may have a marker
    /// AND zero entries (no headings, or all filtered by `toc_max_depth`)
    /// — in that case we still need to suppress the literal `[[toc]]`
    /// text from the output.
    document_has_toc_marker: bool,
    /// Reusable scratch buffer for the raw concatenated heading text in
    /// `heading_id`. A long-lived buffer avoids paying for a fresh
    /// `String` allocation per heading — `slugify_heading` previously
    /// allocated one `text` String per call.
    heading_text_scratch: String,
    /// Reusable scratch buffer for the slugified id. The final id String
    /// that ends up in `heading_id_counts` is cloned out of here on
    /// vacant inserts; the buffer itself stays around across renders.
    heading_slug_scratch: String,
    /// Suppresses URL auto-linking while we're already inside an `<a>` so
    /// the builtin can't nest anchors. Tracked manually rather than via
    /// the AST because `visit_text` can be reached through many parents
    /// (paragraphs, headings, emphasis, …) and only the link case needs
    /// to mask it out.
    in_link: bool,
    /// First-byte skip index for the autolink scanner. It depends only on
    /// `options.autolink_patterns`, which is immutable for the duration of a
    /// render, so it is built once at `render()` entry and reused for every
    /// text node instead of being rebuilt per node (the prior behaviour zeroed
    /// and filled a 256-byte table on the hottest inline path). `None` when
    /// autolinking is disabled or there are no patterns.
    autolink_index: Option<FirstByteIndex>,
}

impl HtmlRenderer {
    /// Creates a new HTML renderer with default options.
    #[must_use]
    pub fn new() -> Self {
        Self::with_options(HtmlRendererOptions::new())
    }

    /// Creates a new HTML renderer with the specified options.
    #[must_use]
    pub fn with_options(options: HtmlRendererOptions) -> Self {
        Self {
            options,
            output: String::new(),
            heading_id_counts: FxHashMap::default(),
            toc_entries: Vec::new(),
            document_has_toc_marker: false,
            // Pre-size the heading scratch buffers: a typical heading text
            // is well under 64 chars. Pre-allocating spares the first
            // heading from a `String::with_capacity(0)` → `reserve(N)`
            // round-trip without meaningful memory cost (these buffers
            // live for the renderer's lifetime regardless).
            heading_text_scratch: String::with_capacity(64),
            heading_slug_scratch: String::with_capacity(64),
            in_link: false,
            autolink_index: None,
        }
    }

    /// Renders a document to HTML string.
    #[must_use]
    pub fn render(&mut self, document: &Document<'_>) -> String {
        crate::profile_span!("renderer::render");
        self.output.clear();
        // Renderer setup is intentionally split into a cheap structural scan
        // and the expensive optional work. TOC collection walks every heading
        // and allocates a slug per entry, which used to fire on every render
        // regardless of whether a `[[toc]]` directive existed. The scan below
        // records only booleans/counts, so documents without TOC markers skip
        // all TOC allocation while the heading count still lets us reserve the
        // unique-id map once.
        self.toc_entries.clear();
        let document_scan = scan_document_for_render(document);
        self.document_has_toc_marker = document_scan.has_toc_marker;
        if self.document_has_toc_marker {
            collect_inline_toc_entries(document, self.options.toc_max_depth, &mut self.toc_entries);
        }
        self.heading_id_counts.clear();
        self.heading_id_counts.reserve(document_scan.heading_count);
        // Build the autolink first-byte index once per render. It depends only
        // on the immutable pattern list, not on the text node being rendered,
        // so reusing it avoids rebuilding a 256-byte table on every inline
        // text visit.
        self.autolink_index =
            if self.options.autolink_urls && !self.options.autolink_patterns.is_empty() {
                Some(FirstByteIndex::from_patterns(&self.options.autolink_patterns))
            } else {
                None
            };
        // HTML output is typically 2×–3× the markdown source (every
        // `**bold**` becomes `<strong>...</strong>` etc.) so the prior
        // 1.5× estimate kept undersizing the buffer and forcing 1–2
        // power-of-two reallocs per render on docs >32 KB. 2× hits the
        // realistic mean for the bundled corpora (rust-book / vite /
        // vue / typescript-handbook all land between 1.8× and 2.6×).
        let estimated_len = (document.span.len() as usize).saturating_mul(2);
        if self.output.capacity() < estimated_len {
            self.output.reserve(estimated_len - self.output.capacity());
        }
        self.render_document(document);
        std::mem::take(&mut self.output)
    }

    pub(in crate::html::renderer) fn render_document(&mut self, document: &Document<'_>) {
        for child in &document.children {
            self.render_node(child);
        }
    }

    #[inline]
    pub(in crate::html::renderer) fn render_node(&mut self, node: &Node<'_>) {
        match node {
            Node::Paragraph(node) => self.render_paragraph(node),
            Node::Heading(node) => self.render_heading(node),
            Node::ThematicBreak(node) => self.render_thematic_break(node),
            Node::BlockQuote(node) => self.render_block_quote(node),
            Node::List(node) => self.render_list(node),
            Node::ListItem(node) => self.render_list_item(node),
            Node::CodeBlock(node) => self.render_code_block(node),
            Node::Html(node) => self.render_html(node),
            Node::Table(node) => self.render_table(node),
            Node::Text(node) => self.render_text(node),
            Node::Emphasis(node) => self.render_emphasis(node),
            Node::Strong(node) => self.render_strong(node),
            Node::InlineCode(node) => self.render_inline_code(node),
            Node::Break(node) => self.render_break(node),
            Node::Link(node) => self.render_link(node),
            Node::Image(node) => self.render_image(node),
            Node::Delete(node) => self.render_delete(node),
            Node::FootnoteReference(node) => self.render_footnote_reference(node),
            Node::Definition(_) => {}
            Node::FootnoteDefinition(node) => self.render_footnote_definition(node),
        }
    }

    pub(in crate::html::renderer) fn render_inline_toc(&mut self) {
        use std::fmt::Write as _;

        if self.toc_entries.is_empty() {
            return;
        }

        self.write("<nav class=\"ox-toc\" aria-label=\"Table of contents\">\n<ul>\n");
        for entry in &self.toc_entries {
            self.output.push_str("<li class=\"ox-toc__item ox-toc__item--depth-");
            let _ = write!(self.output, "{}", entry.depth);
            self.output.push_str("\"><a href=\"#");
            write_url_escaped_into(&mut self.output, &entry.id);
            self.output.push_str("\">");
            write_escaped_into(&mut self.output, &entry.text);
            self.output.push_str("</a></li>\n");
        }
        self.write("</ul>\n</nav>\n");
    }
}

impl Default for HtmlRenderer {
    fn default() -> Self {
        Self::new()
    }
}

impl Renderer for HtmlRenderer {
    type Output = String;

    fn render(&mut self, document: &Document<'_>) -> RenderResult<Self::Output> {
        Ok(self.render(document))
    }
}
