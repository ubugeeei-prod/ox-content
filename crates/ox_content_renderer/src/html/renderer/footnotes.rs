//! Footnote reference and definition rendering.
//!
//! The default path keeps the current alpha markup: the source identifier is
//! the visible marker, and each definition is an independent
//! `<div class="footnote">`.
//!
//! The `semantic_footnotes` renderer option switches to document-order numeric
//! markers and one `<section class="footnotes"><ol>` of definitions. The
//! identifier is used only for lookup and slug generation. Numbering and
//! collection happen in the same AST walk: one list entry per unique
//! footnote, no extra allocation per later marker.

use std::fmt::Write as _;

use compact_str::CompactString;
use ox_content_ast::{FootnoteDefinition, FootnoteReference};

use super::super::escape::write_escaped_into;
use super::super::heading::slugify_heading_into;
use super::HtmlRenderer;

#[derive(Clone)]
pub(super) struct FootnoteRecord {
    pub(super) slug: CompactString,
    pub(super) ref_count: usize,
    pub(super) body_html: Option<String>,
}

impl HtmlRenderer {
    pub(in crate::html::renderer) fn render_footnote_reference(
        &mut self,
        footnote_ref: &FootnoteReference<'_>,
    ) {
        crate::profile_span_detail!("renderer::visit_footnote_ref");
        if self.options.semantic_footnotes {
            self.render_semantic_footnote_reference(footnote_ref.identifier);
            return;
        }
        self.render_legacy_footnote_reference(footnote_ref.identifier);
    }

    pub(in crate::html::renderer) fn render_footnote_definition(
        &mut self,
        footnote_def: &FootnoteDefinition<'_>,
    ) {
        crate::profile_span!("renderer::visit_footnote_def");
        if self.options.semantic_footnotes {
            self.collect_semantic_footnote_definition(footnote_def);
            return;
        }
        self.render_legacy_footnote_definition(footnote_def);
    }

    pub(in crate::html::renderer) fn finish_semantic_footnotes(&mut self) {
        if !self.options.semantic_footnotes
            || !self.footnote_records.iter().any(|record| record.body_html.is_some())
        {
            return;
        }

        crate::profile_span!("renderer::finish_footnotes");
        self.write("<section class=\"footnotes\" aria-label=\"Footnotes\">\n<ol>\n");
        for index in 0..self.footnote_records.len() {
            let Some(body) = self.footnote_records[index].body_html.take() else {
                continue;
            };
            let display = index + 1;
            let ref_count = self.footnote_records[index].ref_count;
            self.write("<li id=\"fn-");
            self.write_footnote_slug(index);
            self.write("\">\n");
            self.write(&body);
            self.write_semantic_backlinks(index, display, ref_count);
            self.write("</li>\n");
        }
        self.write("</ol>\n</section>\n");
    }

    pub(in crate::html::renderer) fn clear_footnote_state(&mut self) {
        self.footnote_ref_counts.clear();
        self.footnote_index.clear();
        self.footnote_records.clear();
    }

    fn render_legacy_footnote_reference(&mut self, identifier: &str) {
        // A footnote may be referenced repeatedly, so each occurrence
        // needs its own id: the first keeps `fnref-<id>` (which the
        // definition's back-link targets) and later ones get a `-N`
        // suffix. Without this the document carries duplicate ids, which
        // is invalid HTML and breaks in-page anchors.
        let occurrence = {
            let count = self.footnote_ref_counts.entry(identifier.to_owned()).or_insert(0);
            *count += 1;
            *count
        };

        self.write("<sup><a href=\"#fn-");
        self.write_escaped(identifier);
        self.write("\" id=\"fnref-");
        self.write_escaped(identifier);
        if occurrence > 1 {
            self.write("-");
            self.write_display(occurrence);
        }
        self.write("\">");
        self.write_escaped(identifier);
        self.write("</a></sup>");
    }

    fn render_legacy_footnote_definition(&mut self, footnote_def: &FootnoteDefinition<'_>) {
        self.write("<div id=\"fn-");
        self.write_escaped(footnote_def.identifier);
        self.write("\" class=\"footnote\">\n");
        for child in &footnote_def.children {
            self.render_node(child);
        }
        self.write("<a href=\"#fnref-");
        self.write_escaped(footnote_def.identifier);
        self.write("\">↩</a>\n</div>\n");
    }

    fn render_semantic_footnote_reference(&mut self, identifier: &str) {
        let index = self.footnote_index_of(identifier);
        let occurrence = {
            let record = &mut self.footnote_records[index];
            record.ref_count += 1;
            record.ref_count
        };

        self.write("<sup><a href=\"#fn-");
        self.write_footnote_slug(index);
        self.write("\" id=\"fnref-");
        self.write_footnote_slug(index);
        if occurrence > 1 {
            self.write("-");
            self.write_display(occurrence);
        }
        self.write("\">");
        self.write_display(index + 1);
        self.write("</a></sup>");
    }

    fn collect_semantic_footnote_definition(&mut self, footnote_def: &FootnoteDefinition<'_>) {
        let index = self.footnote_index_of(footnote_def.identifier);
        if self.footnote_records[index].body_html.is_some() {
            return;
        }
        let start = self.output.len();
        for child in &footnote_def.children {
            self.render_node(child);
        }
        self.footnote_records[index].body_html = Some(self.output.split_off(start));
    }

    fn write_semantic_backlinks(&mut self, index: usize, display: usize, ref_count: usize) {
        if ref_count == 0 {
            return;
        }
        for occurrence in 1..=ref_count {
            self.write("<a href=\"#fnref-");
            self.write_footnote_slug(index);
            if occurrence > 1 {
                self.write("-");
                self.write_display(occurrence);
            }
            self.write("\" aria-label=\"Back to reference ");
            self.write_display(display);
            if occurrence > 1 {
                self.write(", occurrence ");
                self.write_display(occurrence);
            }
            self.write("\">↩</a>");
            if occurrence < ref_count {
                self.write(" ");
            }
        }
        self.write("\n");
    }

    fn footnote_index_of(&mut self, identifier: &str) -> usize {
        if let Some(&index) = self.footnote_index.get(identifier) {
            return index as usize;
        }
        let index = self.footnote_records.len();
        let slug = self.assign_footnote_slug(identifier, index);
        self.footnote_index.insert(CompactString::from(identifier), index as u32);
        self.footnote_records.push(FootnoteRecord { slug, ref_count: 0, body_html: None });
        index
    }

    fn assign_footnote_slug(&mut self, identifier: &str, index: usize) -> CompactString {
        self.heading_slug_scratch.clear();
        slugify_heading_into(identifier, &mut self.heading_slug_scratch);
        if self.heading_slug_scratch == "section" && !identifier.chars().any(char::is_alphanumeric)
        {
            self.heading_slug_scratch.clear();
            let _ = write!(self.heading_slug_scratch, "footnote-{}", index + 1);
        }
        self.uniquify_footnote_slug();
        CompactString::from(self.heading_slug_scratch.as_str())
    }

    fn write_footnote_slug(&mut self, index: usize) {
        write_escaped_into(&mut self.output, &self.footnote_records[index].slug);
    }

    fn uniquify_footnote_slug(&mut self) {
        if !self.footnote_records.iter().any(|record| record.slug == self.heading_slug_scratch) {
            return;
        }
        let base_len = self.heading_slug_scratch.len();
        let mut suffix = 2usize;
        loop {
            self.heading_slug_scratch.truncate(base_len);
            let _ = write!(self.heading_slug_scratch, "-{suffix}");
            if !self.footnote_records.iter().any(|record| record.slug == self.heading_slug_scratch)
            {
                return;
            }
            suffix += 1;
        }
    }
}
