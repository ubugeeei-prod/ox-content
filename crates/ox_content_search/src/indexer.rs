//! Document indexer using the Visitor pattern.

use ox_content_ast::{CodeBlock, Document, Heading, InlineCode, Node, Text, Visit, walk_document};

use crate::index::SearchDocument;

/// Extracts searchable content from a Markdown AST using the Visitor pattern.
#[derive(Debug, Default)]
pub struct DocumentIndexer {
    /// Collected title (first h1 heading).
    title: Option<String>,
    /// All headings in the document.
    headings: Vec<String>,
    /// Body text content.
    body: String,
    /// Code snippets.
    code: Vec<String>,
    /// Current heading being built.
    current_heading: String,
    /// Whether we're inside a heading.
    in_heading: bool,
}

impl DocumentIndexer {
    /// Creates a new document indexer.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Extracts searchable content from a document.
    pub fn extract<'a>(&mut self, doc: &Document<'a>) {
        walk_document(self, doc);
    }

    /// Creates a `SearchDocument` from the extracted content.
    #[must_use]
    pub fn into_search_document(self, id: String, url: String) -> SearchDocument {
        SearchDocument {
            id,
            title: self.title.unwrap_or_default(),
            url,
            body: self.body,
            headings: self.headings,
            code: self.code,
        }
    }

    /// Returns the extracted title.
    #[must_use]
    pub fn title(&self) -> Option<&str> {
        self.title.as_deref()
    }

    /// Returns the extracted body text.
    #[must_use]
    pub fn body(&self) -> &str {
        &self.body
    }

    /// Returns the extracted headings.
    #[must_use]
    pub fn headings(&self) -> &[String] {
        &self.headings
    }

    /// Returns the extracted code snippets.
    #[must_use]
    pub fn code(&self) -> &[String] {
        &self.code
    }
}

impl<'a> Visit<'a> for DocumentIndexer {
    fn visit_heading(&mut self, heading: &Heading<'a>) {
        self.in_heading = true;
        self.current_heading.clear();

        // Visit children to collect heading text
        for child in &heading.children {
            self.visit_node(child);
        }

        self.in_heading = false;

        let heading_text = std::mem::take(&mut self.current_heading);
        if !heading_text.is_empty() {
            // First h1 becomes the title
            if heading.depth == 1 && self.title.is_none() {
                self.title = Some(heading_text.clone());
            }
            self.headings.push(heading_text);
        }
    }

    fn visit_text(&mut self, text: &Text<'a>) {
        if self.in_heading {
            self.current_heading.push_str(text.value);
        } else {
            if !self.body.is_empty() {
                self.body.push(' ');
            }
            self.body.push_str(text.value);
        }
    }

    fn visit_inline_code(&mut self, inline_code: &InlineCode<'a>) {
        if self.in_heading {
            self.current_heading.push_str(inline_code.value);
        } else {
            if !self.body.is_empty() {
                self.body.push(' ');
            }
            self.body.push_str(inline_code.value);
        }
    }

    fn visit_code_block(&mut self, code_block: &CodeBlock<'a>) {
        self.code.push(code_block.value.to_string());
    }

    fn visit_node(&mut self, node: &Node<'a>) {
        match node {
            Node::Text(n) => self.visit_text(n),
            Node::Heading(n) => self.visit_heading(n),
            Node::InlineCode(n) => self.visit_inline_code(n),
            Node::CodeBlock(n) => self.visit_code_block(n),
            Node::Paragraph(n) => {
                for child in &n.children {
                    self.visit_node(child);
                }
            }
            Node::BlockQuote(n) => {
                for child in &n.children {
                    self.visit_node(child);
                }
            }
            Node::List(n) => {
                for item in &n.children {
                    for child in &item.children {
                        self.visit_node(child);
                    }
                }
            }
            Node::ListItem(n) => {
                for child in &n.children {
                    self.visit_node(child);
                }
            }
            Node::Table(n) => {
                for row in &n.children {
                    for cell in &row.children {
                        for child in &cell.children {
                            self.visit_node(child);
                        }
                    }
                }
            }
            Node::Emphasis(n) => {
                for child in &n.children {
                    self.visit_node(child);
                }
            }
            Node::Strong(n) => {
                for child in &n.children {
                    self.visit_node(child);
                }
            }
            Node::Link(n) => {
                for child in &n.children {
                    self.visit_node(child);
                }
            }
            Node::Delete(n) => {
                for child in &n.children {
                    self.visit_node(child);
                }
            }
            Node::FootnoteDefinition(n) => {
                for child in &n.children {
                    self.visit_node(child);
                }
            }
            Node::MdxJsxFlowElement(n) => {
                for child in &n.children {
                    self.visit_node(child);
                }
            }
            Node::MdxJsxTextElement(n) => {
                for child in &n.children {
                    self.visit_node(child);
                }
            }
            _ => {}
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ox_content_allocator::Allocator;
    use ox_content_ast::Span;

    #[test]
    fn test_extract_title() {
        let allocator = Allocator::new();
        let mut children = ox_content_allocator::Vec::new_in(&allocator);

        // Create a heading
        let mut heading_children = ox_content_allocator::Vec::new_in(&allocator);
        heading_children.push(Node::Text(Text { value: "Test Title", span: Span::new(0, 10) }));

        children.push(Node::Heading(allocator.boxed(Heading {
            depth: 1,
            children: heading_children,
            span: Span::new(0, 12),
        })));

        let doc = Document { children, span: Span::new(0, 12) };

        let mut indexer = DocumentIndexer::new();
        indexer.extract(&doc);

        assert_eq!(indexer.title(), Some("Test Title"));
    }
}
