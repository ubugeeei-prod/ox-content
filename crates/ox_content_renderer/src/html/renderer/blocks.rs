//! Block-level HTML visitor helpers.
//!
//! The trait implementation delegates here for paragraphs, headings, lists, tables,
//! raw HTML, block quotes, and fenced code. Separating block rendering from the trait
//! glue keeps each file small while preserving the visitor behavior exactly.

use ox_content_ast::{
    BlockQuote, CodeBlock, Heading, Html, List, ListItem, Paragraph, Table, TableCell, TableRow,
    ThematicBreak,
};

use super::super::code_annotations::normalize_code_block_language;
use super::super::toc::is_toc_marker_paragraph;
use super::HtmlRenderer;

impl HtmlRenderer {
    pub(in crate::html::renderer) fn render_paragraph(&mut self, paragraph: &Paragraph<'_>) {
        crate::profile_span!("renderer::visit_paragraph");
        // Skip the `[[toc]]` byte scan entirely when the document has no
        // marker — pure overhead in the common case. When a marker IS
        // present we must run the check on every paragraph and suppress
        // the matching one, even if `toc_entries` is empty (e.g. document
        // has no headings or all are filtered by `toc_max_depth`).
        // Otherwise the literal `[[toc]]` would leak into the output.
        if self.document_has_toc_marker && is_toc_marker_paragraph(paragraph) {
            self.render_inline_toc();
            return;
        }

        self.output.push_str("<p>");
        for child in &paragraph.children {
            self.visit_inline_node(child);
        }
        self.output.push_str("</p>\n");
    }

    pub(in crate::html::renderer) fn render_heading(&mut self, heading: &Heading<'_>) {
        crate::profile_span!("renderer::visit_heading");
        // Avoid the heading.depth -> &str match per call: heading depth is
        // 1..=6 by construction, and "h%d" is a fixed shape we can splat
        // directly. Saves a branch and a `write` call.
        let depth = heading.depth.clamp(1, 6);
        self.output.push_str("<h");
        self.output.push((b'0' + depth) as char);
        self.output.push_str(" id=\"");
        // Heading ids are slugified: lowercase alnum + '-' separators. None
        // of those bytes need HTML escaping, so the unconditional
        // `write_escaped` pass over the id was pure overhead. We also
        // skip materializing the id as a return-value `String`; it's
        // written straight into `self.output`.
        self.write_heading_id(heading);
        self.output.push_str("\">");
        for child in &heading.children {
            self.visit_inline_node(child);
        }
        self.output.push_str("</h");
        self.output.push((b'0' + depth) as char);
        self.output.push_str(">\n");
    }

    pub(in crate::html::renderer) fn render_thematic_break(
        &mut self,
        _thematic_break: &ThematicBreak,
    ) {
        if self.options.xhtml {
            self.write("<hr />\n");
        } else {
            self.write("<hr>\n");
        }
    }

    pub(in crate::html::renderer) fn render_block_quote(&mut self, block_quote: &BlockQuote<'_>) {
        crate::profile_span!("renderer::visit_block_quote");
        if self.render_callout_block_quote(block_quote) {
            return;
        }

        self.write("<blockquote>\n");
        for child in &block_quote.children {
            self.render_node(child);
        }
        self.write("</blockquote>\n");
    }

    pub(in crate::html::renderer) fn render_list(&mut self, list: &List<'_>) {
        crate::profile_span!("renderer::visit_list");
        if list.ordered {
            if let Some(start) = list.start {
                if start != 1 {
                    self.write("<ol start=\"");
                    self.write_display(start);
                    self.write("\">\n");
                } else {
                    self.write("<ol>\n");
                }
            } else {
                self.write("<ol>\n");
            }
        } else {
            self.write("<ul>\n");
        }

        // A tight list renders item paragraphs without <p> wrappers
        // (CommonMark "Lists": loose lists are the ones whose items are
        // separated by blank lines or contain multiple blocks).
        let tight = !list.spread;
        for child in &list.children {
            self.render_list_item_with_tightness(child, tight);
        }

        if list.ordered {
            self.write("</ol>\n");
        } else {
            self.write("</ul>\n");
        }
    }

    pub(in crate::html::renderer) fn render_list_item(&mut self, list_item: &ListItem<'_>) {
        self.render_list_item_with_tightness(list_item, false);
    }

    pub(in crate::html::renderer) fn render_list_item_with_tightness(
        &mut self,
        list_item: &ListItem<'_>,
        tight: bool,
    ) {
        self.write("<li>");

        if let Some(checked) = list_item.checked {
            if checked {
                self.write("<input type=\"checkbox\" checked disabled> ");
            } else {
                self.write("<input type=\"checkbox\" disabled> ");
            }
        }

        for child in &list_item.children {
            if tight {
                if let ox_content_ast::Node::Paragraph(paragraph) = child {
                    for inline in &paragraph.children {
                        self.visit_inline_node(inline);
                    }
                    continue;
                }
                // Keep nested blocks on their own lines even when the
                // preceding paragraph was rendered inline.
                if !self.output.is_empty() && !self.output.ends_with('\n') {
                    self.write("\n");
                }
            }
            self.render_node(child);
        }

        self.write("</li>\n");
    }

    pub(in crate::html::renderer) fn render_code_block(&mut self, code_block: &CodeBlock<'_>) {
        crate::profile_span!("renderer::visit_code_block");
        if !self.options.code_annotations {
            self.write("<pre><code");
            if let Some(lang) = normalize_code_block_language(code_block.lang) {
                self.write(" class=\"language-");
                self.write_escaped(lang);
                self.write("\"");
            }
            self.write(">");
            self.write_escaped(code_block.value);
            self.write("</code></pre>\n");
            return;
        }

        let state = self.build_code_block_state(code_block);
        let block_classes = state.block_classes();

        self.write("<pre");
        if !block_classes.is_empty() {
            self.write(" class=\"");
            self.write(&block_classes.join(" "));
            self.write("\"");
        }
        if let Some(title) = state.title.as_deref() {
            self.write(" data-code-title=\"");
            self.write_escaped(title);
            self.write("\"");
        }
        if let Some(start) = state.line_numbers_start {
            self.write(" data-line-numbers=\"true\" data-line-number-start=\"");
            self.write_display(start);
            self.write("\"");
        }
        self.write("><code");
        if let Some(lang) = state.language.as_deref() {
            self.write(" class=\"language-");
            self.write_escaped(lang);
            self.write("\"");
        }
        self.write(">");
        if state.needs_line_wrappers() {
            self.write_code_lines(&state);
        } else {
            self.write_escaped(code_block.value);
        }
        self.write("</code></pre>\n");
    }

    pub(in crate::html::renderer) fn render_html(&mut self, html: &Html<'_>) {
        self.write_html_value(html.value);
        // Block-level HTML values captured from full source lines already
        // end with their newline; don't double it.
        if !self.output.ends_with('\n') {
            self.write("\n");
        }
    }

    pub(in crate::html::renderer) fn render_table(&mut self, table: &Table<'_>) {
        crate::profile_span!("renderer::visit_table");
        self.write("<table>\n");
        for (i, row) in table.children.iter().enumerate() {
            if i == 0 {
                self.write("<thead>\n");
            } else if i == 1 {
                self.write("<tbody>\n");
            }
            self.visit_table_row_with_header(row, i == 0, &table.align);
            if i == 0 {
                self.write("</thead>\n");
            }
        }
        if table.children.len() > 1 {
            self.write("</tbody>\n");
        }
        self.write("</table>\n");
    }
    pub(in crate::html::renderer) fn visit_table_row_with_header(
        &mut self,
        row: &TableRow<'_>,
        is_header: bool,
        align: &ox_content_allocator::Vec<'_, ox_content_ast::AlignKind>,
    ) {
        self.write("<tr>\n");
        let tag = if is_header { "th" } else { "td" };
        for (idx, cell) in row.children.iter().enumerate() {
            self.write("<");
            self.write(tag);
            match align.get(idx).copied().unwrap_or(ox_content_ast::AlignKind::None) {
                ox_content_ast::AlignKind::Left => self.write(" align=\"left\""),
                ox_content_ast::AlignKind::Center => self.write(" align=\"center\""),
                ox_content_ast::AlignKind::Right => self.write(" align=\"right\""),
                ox_content_ast::AlignKind::None => {}
            }
            self.write(">");
            self.visit_table_cell(cell);
            self.write("</");
            self.write(tag);
            self.write(">\n");
        }
        self.write("</tr>\n");
    }

    pub(in crate::html::renderer) fn visit_table_cell(&mut self, cell: &TableCell<'_>) {
        for child in &cell.children {
            self.visit_inline_node(child);
        }
    }
}
