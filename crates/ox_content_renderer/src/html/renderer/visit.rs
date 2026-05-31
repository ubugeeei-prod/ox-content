//! Visitor trait glue for `HtmlRenderer`.
//!
//! Each trait method delegates to a focused helper module. The AST-facing behavior
//! remains concentrated in one impl block, while the larger HTML-writing logic stays
//! split by block and inline responsibilities.

use ox_content_ast::{
    BlockQuote, Break, CodeBlock, Definition, Delete, Document, Emphasis, FootnoteDefinition,
    FootnoteReference, Heading, Html, Image, InlineCode, Link, List, ListItem, Paragraph, Strong,
    Table, Text, ThematicBreak, Visit,
};

use super::HtmlRenderer;

impl<'a> Visit<'a> for HtmlRenderer {
    fn visit_document(&mut self, document: &Document<'a>) {
        for child in &document.children {
            self.visit_node(child);
        }
    }

    fn visit_paragraph(&mut self, paragraph: &Paragraph<'a>) {
        self.render_paragraph(paragraph);
    }

    fn visit_heading(&mut self, heading: &Heading<'a>) {
        self.render_heading(heading);
    }

    fn visit_thematic_break(&mut self, thematic_break: &ThematicBreak) {
        self.render_thematic_break(thematic_break);
    }

    fn visit_block_quote(&mut self, block_quote: &BlockQuote<'a>) {
        self.render_block_quote(block_quote);
    }

    fn visit_list(&mut self, list: &List<'a>) {
        self.render_list(list);
    }

    fn visit_list_item(&mut self, list_item: &ListItem<'a>) {
        self.render_list_item(list_item);
    }

    fn visit_code_block(&mut self, code_block: &CodeBlock<'a>) {
        self.render_code_block(code_block);
    }

    fn visit_html(&mut self, html: &Html<'a>) {
        self.render_html(html);
    }

    fn visit_table(&mut self, table: &Table<'a>) {
        self.render_table(table);
    }

    fn visit_text(&mut self, text: &Text<'a>) {
        self.render_text(text);
    }

    fn visit_emphasis(&mut self, emphasis: &Emphasis<'a>) {
        self.render_emphasis(emphasis);
    }

    fn visit_strong(&mut self, strong: &Strong<'a>) {
        self.render_strong(strong);
    }

    fn visit_inline_code(&mut self, inline_code: &InlineCode<'a>) {
        self.render_inline_code(inline_code);
    }

    fn visit_break(&mut self, break_node: &Break) {
        self.render_break(break_node);
    }

    fn visit_link(&mut self, link: &Link<'a>) {
        self.render_link(link);
    }

    fn visit_image(&mut self, image: &Image<'a>) {
        self.render_image(image);
    }

    fn visit_delete(&mut self, delete: &Delete<'a>) {
        self.render_delete(delete);
    }

    fn visit_footnote_reference(&mut self, footnote_ref: &FootnoteReference<'a>) {
        self.render_footnote_reference(footnote_ref);
    }

    fn visit_definition(&mut self, definition: &Definition<'a>) {
        let _ = definition;
        // Link definitions are lookup metadata for parsers and are not rendered directly.
    }

    fn visit_footnote_definition(&mut self, footnote_def: &FootnoteDefinition<'a>) {
        self.render_footnote_definition(footnote_def);
    }
}
