//! Inline-level HTML visitor helpers.
//!
//! Inline nodes primarily write escaped text or small tags. This module also owns link
//! and image URL sanitization so URL escaping and Markdown link conversion happen in a
//! single place.

use ox_content_ast::{
    Break, Delete, Emphasis, FootnoteDefinition, FootnoteReference, Image, InlineCode, Link,
    Strong, Text,
};

use super::HtmlRenderer;

impl HtmlRenderer {
    pub(in crate::html::renderer) fn render_text(&mut self, text: &Text<'_>) {
        crate::profile_span!("renderer::visit_text");
        // See the matching gate in `visit_inline_node`: the cached
        // `autolink_index` already encodes `autolink_urls && !patterns.is_empty()`.
        if self.autolink_index.is_some() && !self.in_link {
            self.write_text_with_autolinks(text.value);
        } else {
            self.write_escaped(text.value);
        }
    }

    pub(in crate::html::renderer) fn render_emphasis(&mut self, emphasis: &Emphasis<'_>) {
        self.write("<em>");
        for child in &emphasis.children {
            self.visit_inline_node(child);
        }
        self.write("</em>");
    }

    pub(in crate::html::renderer) fn render_strong(&mut self, strong: &Strong<'_>) {
        self.write("<strong>");
        for child in &strong.children {
            self.visit_inline_node(child);
        }
        self.write("</strong>");
    }

    pub(in crate::html::renderer) fn render_inline_code(&mut self, inline_code: &InlineCode<'_>) {
        self.write("<code>");
        self.write_escaped(inline_code.value);
        self.write("</code>");
    }

    pub(in crate::html::renderer) fn render_break(&mut self, _break_node: &Break) {
        self.output.push_str(self.options.hard_break.as_str());
    }

    pub(in crate::html::renderer) fn render_link(&mut self, link: &Link<'_>) {
        self.write("<a href=\"");
        let converted_url =
            if self.options.convert_md_links { self.convert_markdown_url(link.url) } else { None };
        let href = self.sanitized_url(converted_url.as_deref().unwrap_or(link.url), "#");
        self.write_url_escaped(href);
        self.write("\"");
        // Add target="_blank" for external links (http:// or https://)
        if href.starts_with("http://") || href.starts_with("https://") {
            self.write(" target=\"_blank\" rel=\"noopener noreferrer\"");
        }
        if let Some(title) = link.title {
            self.write(" title=\"");
            self.write_escaped(title);
            self.write("\"");
        }
        self.write(">");
        // Suppress URL auto-linking inside the anchor — children text nodes
        // may contain literal URLs that we must not wrap in a nested <a>.
        let prev_in_link = self.in_link;
        self.in_link = true;
        for child in &link.children {
            self.visit_inline_node(child);
        }
        self.in_link = prev_in_link;
        self.write("</a>");
    }

    pub(in crate::html::renderer) fn render_image(&mut self, image: &Image<'_>) {
        self.write("<img src=\"");
        let converted_url =
            if self.options.convert_md_links { self.convert_markdown_url(image.url) } else { None };
        let src = self.sanitized_url(converted_url.as_deref().unwrap_or(image.url), "");
        self.write_url_escaped(src);
        self.write("\" alt=\"");
        self.write_escaped(image.alt);
        self.write("\"");
        if let Some(title) = image.title {
            self.write(" title=\"");
            self.write_escaped(title);
            self.write("\"");
        }
        if self.options.xhtml {
            self.write(" />");
        } else {
            self.write(">");
        }
    }

    pub(in crate::html::renderer) fn render_delete(&mut self, delete: &Delete<'_>) {
        self.write("<del>");
        for child in &delete.children {
            self.visit_inline_node(child);
        }
        self.write("</del>");
    }

    pub(in crate::html::renderer) fn render_footnote_reference(
        &mut self,
        footnote_ref: &FootnoteReference<'_>,
    ) {
        // A footnote may be referenced repeatedly, so each occurrence
        // needs its own id: the first keeps `fnref-<id>` (which the
        // definition's back-link targets) and later ones get a `-N`
        // suffix. Without this the document carries duplicate ids, which
        // is invalid HTML and breaks in-page anchors.
        let occurrence = {
            let count =
                self.footnote_ref_counts.entry(footnote_ref.identifier.to_owned()).or_insert(0);
            *count += 1;
            *count
        };

        self.write("<sup><a href=\"#fn-");
        self.write_escaped(footnote_ref.identifier);
        self.write("\" id=\"fnref-");
        self.write_escaped(footnote_ref.identifier);
        if occurrence > 1 {
            self.write("-");
            self.write_display(occurrence);
        }
        self.write("\">");
        self.write_escaped(footnote_ref.identifier);
        self.write("</a></sup>");
    }

    pub(in crate::html::renderer) fn render_footnote_definition(
        &mut self,
        footnote_def: &FootnoteDefinition<'_>,
    ) {
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
}
