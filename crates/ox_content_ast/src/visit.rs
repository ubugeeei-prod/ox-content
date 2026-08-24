//! AST visitor trait for traversing Markdown AST.

use crate::ast::*;
use crate::mdx::*;

/// Visitor trait for traversing the Markdown AST.
pub trait Visit<'a> {
    /// Visits a document.
    fn visit_document(&mut self, document: &Document<'a>) {
        walk_document(self, document);
    }

    /// Visits a node.
    fn visit_node(&mut self, node: &Node<'a>) {
        walk_node(self, node);
    }

    /// Visits a paragraph.
    fn visit_paragraph(&mut self, paragraph: &Paragraph<'a>) {
        walk_paragraph(self, paragraph);
    }

    /// Visits a heading.
    fn visit_heading(&mut self, heading: &Heading<'a>) {
        walk_heading(self, heading);
    }

    /// Visits a thematic break.
    fn visit_thematic_break(&mut self, _thematic_break: &ThematicBreak) {}

    /// Visits a block quote.
    fn visit_block_quote(&mut self, block_quote: &BlockQuote<'a>) {
        walk_block_quote(self, block_quote);
    }

    /// Visits a list.
    fn visit_list(&mut self, list: &List<'a>) {
        walk_list(self, list);
    }

    /// Visits a list item.
    fn visit_list_item(&mut self, list_item: &ListItem<'a>) {
        walk_list_item(self, list_item);
    }

    /// Visits a code block.
    fn visit_code_block(&mut self, _code_block: &CodeBlock<'a>) {}

    /// Visits an HTML block.
    fn visit_html(&mut self, _html: &Html<'a>) {}

    /// Visits a table.
    fn visit_table(&mut self, table: &Table<'a>) {
        walk_table(self, table);
    }

    /// Visits a table row.
    fn visit_table_row(&mut self, table_row: &TableRow<'a>) {
        walk_table_row(self, table_row);
    }

    /// Visits a table cell.
    fn visit_table_cell(&mut self, table_cell: &TableCell<'a>) {
        walk_table_cell(self, table_cell);
    }

    /// Visits text.
    fn visit_text(&mut self, _text: &Text<'a>) {}

    /// Visits emphasis.
    fn visit_emphasis(&mut self, emphasis: &Emphasis<'a>) {
        walk_emphasis(self, emphasis);
    }

    /// Visits strong emphasis.
    fn visit_strong(&mut self, strong: &Strong<'a>) {
        walk_strong(self, strong);
    }

    /// Visits inline code.
    fn visit_inline_code(&mut self, _inline_code: &InlineCode<'a>) {}

    /// Visits a line break.
    fn visit_break(&mut self, _break_node: &Break) {}

    /// Visits a link.
    fn visit_link(&mut self, link: &Link<'a>) {
        walk_link(self, link);
    }

    /// Visits an image.
    fn visit_image(&mut self, _image: &Image<'a>) {}

    /// Visits strikethrough.
    fn visit_delete(&mut self, delete: &Delete<'a>) {
        walk_delete(self, delete);
    }

    /// Visits a footnote reference.
    fn visit_footnote_reference(&mut self, _footnote_ref: &FootnoteReference<'a>) {}

    /// Visits a definition.
    fn visit_definition(&mut self, _definition: &Definition<'a>) {}

    /// Visits a footnote definition.
    fn visit_footnote_definition(&mut self, footnote_def: &FootnoteDefinition<'a>) {
        walk_footnote_definition(self, footnote_def);
    }

    /// Visits a block JSX element.
    fn visit_mdx_jsx_flow_element(&mut self, node: &MdxJsxFlowElement<'a>) {
        walk_mdx_jsx_flow_element(self, node);
    }

    /// Visits an inline JSX element.
    fn visit_mdx_jsx_text_element(&mut self, node: &MdxJsxTextElement<'a>) {
        walk_mdx_jsx_text_element(self, node);
    }

    /// Visits an MDX ESM `import` / `export`.
    fn visit_mdxjs_esm(&mut self, _node: &MdxjsEsm<'a>) {}

    /// Visits a block MDX expression.
    fn visit_mdx_flow_expression(&mut self, _node: &MdxFlowExpression<'a>) {}

    /// Visits an inline MDX expression.
    fn visit_mdx_text_expression(&mut self, _node: &MdxTextExpression<'a>) {}
}

/// Walks through a document's children.
pub fn walk_document<'a, V: Visit<'a> + ?Sized>(visitor: &mut V, document: &Document<'a>) {
    for child in &document.children {
        visitor.visit_node(child);
    }
}

/// Walks through a node, dispatching to the appropriate visit method.
pub fn walk_node<'a, V: Visit<'a> + ?Sized>(visitor: &mut V, node: &Node<'a>) {
    match node {
        Node::Paragraph(n) => visitor.visit_paragraph(n),
        Node::Heading(n) => visitor.visit_heading(n),
        Node::ThematicBreak(n) => visitor.visit_thematic_break(n),
        Node::BlockQuote(n) => visitor.visit_block_quote(n),
        Node::List(n) => visitor.visit_list(n),
        Node::ListItem(n) => visitor.visit_list_item(n),
        Node::CodeBlock(n) => visitor.visit_code_block(n),
        Node::Html(n) => visitor.visit_html(n),
        Node::Table(n) => visitor.visit_table(n),
        Node::Text(n) => visitor.visit_text(n),
        Node::Emphasis(n) => visitor.visit_emphasis(n),
        Node::Strong(n) => visitor.visit_strong(n),
        Node::InlineCode(n) => visitor.visit_inline_code(n),
        Node::Break(n) => visitor.visit_break(n),
        Node::Link(n) => visitor.visit_link(n),
        Node::Image(n) => visitor.visit_image(n),
        Node::Delete(n) => visitor.visit_delete(n),
        Node::FootnoteReference(n) => visitor.visit_footnote_reference(n),
        Node::Definition(n) => visitor.visit_definition(n),
        Node::FootnoteDefinition(n) => visitor.visit_footnote_definition(n),
        Node::MdxJsxFlowElement(n) => visitor.visit_mdx_jsx_flow_element(n),
        Node::MdxJsxTextElement(n) => visitor.visit_mdx_jsx_text_element(n),
        Node::MdxjsEsm(n) => visitor.visit_mdxjs_esm(n),
        Node::MdxFlowExpression(n) => visitor.visit_mdx_flow_expression(n),
        Node::MdxTextExpression(n) => visitor.visit_mdx_text_expression(n),
    }
}

/// Walks through a paragraph's children.
pub fn walk_paragraph<'a, V: Visit<'a> + ?Sized>(visitor: &mut V, paragraph: &Paragraph<'a>) {
    for child in &paragraph.children {
        visitor.visit_node(child);
    }
}

/// Walks through a heading's children.
pub fn walk_heading<'a, V: Visit<'a> + ?Sized>(visitor: &mut V, heading: &Heading<'a>) {
    for child in &heading.children {
        visitor.visit_node(child);
    }
}

/// Walks through a block quote's children.
pub fn walk_block_quote<'a, V: Visit<'a> + ?Sized>(visitor: &mut V, block_quote: &BlockQuote<'a>) {
    for child in &block_quote.children {
        visitor.visit_node(child);
    }
}

/// Walks through a list's children.
pub fn walk_list<'a, V: Visit<'a> + ?Sized>(visitor: &mut V, list: &List<'a>) {
    for child in &list.children {
        visitor.visit_list_item(child);
    }
}

/// Walks through a list item's children.
pub fn walk_list_item<'a, V: Visit<'a> + ?Sized>(visitor: &mut V, list_item: &ListItem<'a>) {
    for child in &list_item.children {
        visitor.visit_node(child);
    }
}

/// Walks through a table's children.
pub fn walk_table<'a, V: Visit<'a> + ?Sized>(visitor: &mut V, table: &Table<'a>) {
    for row in &table.children {
        visitor.visit_table_row(row);
    }
}

/// Walks through a table row's children.
pub fn walk_table_row<'a, V: Visit<'a> + ?Sized>(visitor: &mut V, table_row: &TableRow<'a>) {
    for cell in &table_row.children {
        visitor.visit_table_cell(cell);
    }
}

/// Walks through a table cell's children.
pub fn walk_table_cell<'a, V: Visit<'a> + ?Sized>(visitor: &mut V, table_cell: &TableCell<'a>) {
    for child in &table_cell.children {
        visitor.visit_node(child);
    }
}

/// Walks through emphasis children.
pub fn walk_emphasis<'a, V: Visit<'a> + ?Sized>(visitor: &mut V, emphasis: &Emphasis<'a>) {
    for child in &emphasis.children {
        visitor.visit_node(child);
    }
}

/// Walks through strong emphasis children.
pub fn walk_strong<'a, V: Visit<'a> + ?Sized>(visitor: &mut V, strong: &Strong<'a>) {
    for child in &strong.children {
        visitor.visit_node(child);
    }
}

/// Walks through a link's children.
pub fn walk_link<'a, V: Visit<'a> + ?Sized>(visitor: &mut V, link: &Link<'a>) {
    for child in &link.children {
        visitor.visit_node(child);
    }
}

/// Walks through strikethrough children.
pub fn walk_delete<'a, V: Visit<'a> + ?Sized>(visitor: &mut V, delete: &Delete<'a>) {
    for child in &delete.children {
        visitor.visit_node(child);
    }
}

/// Walks through a footnote definition's children.
pub fn walk_footnote_definition<'a, V: Visit<'a> + ?Sized>(
    visitor: &mut V,
    footnote_def: &FootnoteDefinition<'a>,
) {
    for child in &footnote_def.children {
        visitor.visit_node(child);
    }
}

/// Walks a block JSX element's children. Attributes are not nodes.
pub fn walk_mdx_jsx_flow_element<'a, V: Visit<'a> + ?Sized>(
    visitor: &mut V,
    node: &MdxJsxFlowElement<'a>,
) {
    for child in &node.children {
        visitor.visit_node(child);
    }
}

/// Walks an inline JSX element's children. Attributes are not nodes.
pub fn walk_mdx_jsx_text_element<'a, V: Visit<'a> + ?Sized>(
    visitor: &mut V,
    node: &MdxJsxTextElement<'a>,
) {
    for child in &node.children {
        visitor.visit_node(child);
    }
}
