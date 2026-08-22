//! AST node definitions for Markdown.
//!
//! This module defines the AST structure based on mdast specification,
//! adapted for arena-based allocation.

use ox_content_allocator::Vec;

use crate::Span;

/// Root node of a Markdown document.
#[derive(Debug)]
pub struct Document<'a> {
    /// Child nodes.
    pub children: Vec<'a, Node<'a>>,
    /// Source span.
    pub span: Span,
}

/// A Markdown AST node.
#[derive(Debug)]
pub enum Node<'a> {
    // Block nodes
    /// Paragraph.
    Paragraph(Paragraph<'a>),
    /// Heading (h1-h6).
    Heading(Heading<'a>),
    /// Thematic break (horizontal rule).
    ThematicBreak(ThematicBreak),
    /// Block quote.
    BlockQuote(BlockQuote<'a>),
    /// Ordered or unordered list.
    List(List<'a>),
    /// List item.
    ListItem(ListItem<'a>),
    /// Fenced or indented code block.
    CodeBlock(CodeBlock<'a>),
    /// HTML block.
    Html(Html<'a>),
    /// Table (GFM extension).
    Table(Table<'a>),

    // Inline nodes
    /// Plain text.
    Text(Text<'a>),
    /// Emphasis (italic).
    Emphasis(Emphasis<'a>),
    /// Strong emphasis (bold).
    Strong(Strong<'a>),
    /// Inline code.
    InlineCode(InlineCode<'a>),
    /// Line break.
    Break(Break),
    /// Link.
    Link(Link<'a>),
    /// Image.
    Image(Image<'a>),
    /// Strikethrough (GFM extension).
    Delete(Delete<'a>),
    /// Footnote reference (GFM extension).
    FootnoteReference(FootnoteReference<'a>),

    // Definition nodes
    /// Link/image reference definition.
    Definition(Definition<'a>),
    /// Footnote definition (GFM extension).
    FootnoteDefinition(FootnoteDefinition<'a>),
}

// Block nodes

/// Paragraph node.
#[derive(Debug)]
pub struct Paragraph<'a> {
    /// Inline children.
    pub children: Vec<'a, Node<'a>>,
    /// Source span.
    pub span: Span,
}

/// Heading node (h1-h6).
#[derive(Debug)]
pub struct Heading<'a> {
    /// Heading depth (1-6).
    pub depth: u8,
    /// Inline children.
    pub children: Vec<'a, Node<'a>>,
    /// Source span.
    pub span: Span,
}

/// Thematic break (horizontal rule).
#[derive(Debug)]
pub struct ThematicBreak {
    /// Source span.
    pub span: Span,
}

/// Block quote.
#[derive(Debug)]
pub struct BlockQuote<'a> {
    /// Block children.
    pub children: Vec<'a, Node<'a>>,
    /// Source span.
    pub span: Span,
}

/// List (ordered or unordered).
#[derive(Debug)]
pub struct List<'a> {
    /// Whether the list is ordered.
    pub ordered: bool,
    /// Starting number for ordered lists.
    pub start: Option<u32>,
    /// Whether the list is spread (items separated by blank lines).
    pub spread: bool,
    /// List item children.
    pub children: Vec<'a, ListItem<'a>>,
    /// Source span.
    pub span: Span,
}

/// List item.
#[derive(Debug)]
pub struct ListItem<'a> {
    /// Whether the item is spread.
    pub spread: bool,
    /// Checkbox state for task lists (GFM).
    pub checked: Option<bool>,
    /// Block children.
    pub children: Vec<'a, Node<'a>>,
    /// Source span.
    pub span: Span,
}

/// Code block (fenced or indented).
#[derive(Debug)]
pub struct CodeBlock<'a> {
    /// Language identifier.
    pub lang: Option<&'a str>,
    /// Meta string (after language).
    pub meta: Option<&'a str>,
    /// Code content.
    pub value: &'a str,
    /// Source span.
    pub span: Span,
}

/// HTML block.
#[derive(Debug)]
pub struct Html<'a> {
    /// Raw HTML content.
    pub value: &'a str,
    /// Source span.
    pub span: Span,
}

/// Table (GFM extension).
#[derive(Debug)]
pub struct Table<'a> {
    /// Column alignments.
    pub align: Vec<'a, AlignKind>,
    /// Table rows (including header).
    pub children: Vec<'a, TableRow<'a>>,
    /// Source span.
    pub span: Span,
}

/// Table row.
#[derive(Debug)]
pub struct TableRow<'a> {
    /// Table cells.
    pub children: Vec<'a, TableCell<'a>>,
    /// Source span.
    pub span: Span,
}

/// Table cell.
#[derive(Debug)]
pub struct TableCell<'a> {
    /// Inline children.
    pub children: Vec<'a, Node<'a>>,
    /// Source span.
    pub span: Span,
}

/// Column alignment in tables.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AlignKind {
    /// No alignment specified.
    None,
    /// Left alignment.
    Left,
    /// Center alignment.
    Center,
    /// Right alignment.
    Right,
}

// Inline nodes

/// Plain text.
#[derive(Debug)]
pub struct Text<'a> {
    /// Text content.
    pub value: &'a str,
    /// Source span.
    pub span: Span,
}

/// Emphasis (italic).
#[derive(Debug)]
pub struct Emphasis<'a> {
    /// Inline children.
    pub children: Vec<'a, Node<'a>>,
    /// Source span.
    pub span: Span,
}

/// Strong emphasis (bold).
#[derive(Debug)]
pub struct Strong<'a> {
    /// Inline children.
    pub children: Vec<'a, Node<'a>>,
    /// Source span.
    pub span: Span,
}

/// Inline code.
#[derive(Debug)]
pub struct InlineCode<'a> {
    /// Code content.
    pub value: &'a str,
    /// Source span.
    pub span: Span,
}

/// Line break.
#[derive(Debug)]
pub struct Break {
    /// Source span.
    pub span: Span,
}

/// Link.
#[derive(Debug)]
pub struct Link<'a> {
    /// URL.
    pub url: &'a str,
    /// Title.
    pub title: Option<&'a str>,
    /// Inline children.
    pub children: Vec<'a, Node<'a>>,
    /// Source span.
    pub span: Span,
}

/// Image.
#[derive(Debug)]
pub struct Image<'a> {
    /// Image URL.
    pub url: &'a str,
    /// Alt text.
    pub alt: &'a str,
    /// Title.
    pub title: Option<&'a str>,
    /// Source span.
    pub span: Span,
}

/// Strikethrough (GFM extension).
#[derive(Debug)]
pub struct Delete<'a> {
    /// Inline children.
    pub children: Vec<'a, Node<'a>>,
    /// Source span.
    pub span: Span,
}

/// Footnote reference (GFM extension).
#[derive(Debug)]
pub struct FootnoteReference<'a> {
    /// Footnote identifier.
    pub identifier: &'a str,
    /// Label.
    pub label: Option<&'a str>,
    /// Source span.
    pub span: Span,
}

// Definition nodes

/// Link/image reference definition.
#[derive(Debug)]
pub struct Definition<'a> {
    /// Identifier.
    pub identifier: &'a str,
    /// Label.
    pub label: Option<&'a str>,
    /// URL.
    pub url: &'a str,
    /// Title.
    pub title: Option<&'a str>,
    /// Source span.
    pub span: Span,
}

/// Footnote definition (GFM extension).
#[derive(Debug)]
pub struct FootnoteDefinition<'a> {
    /// Footnote identifier.
    pub identifier: &'a str,
    /// Label.
    pub label: Option<&'a str>,
    /// Block children.
    pub children: Vec<'a, Node<'a>>,
    /// Source span.
    pub span: Span,
}

impl<'a> Node<'a> {
    /// Returns the span of this node.
    #[must_use]
    pub fn span(&self) -> Span {
        match self {
            Self::Paragraph(n) => n.span,
            Self::Heading(n) => n.span,
            Self::ThematicBreak(n) => n.span,
            Self::BlockQuote(n) => n.span,
            Self::List(n) => n.span,
            Self::ListItem(n) => n.span,
            Self::CodeBlock(n) => n.span,
            Self::Html(n) => n.span,
            Self::Table(n) => n.span,
            Self::Text(n) => n.span,
            Self::Emphasis(n) => n.span,
            Self::Strong(n) => n.span,
            Self::InlineCode(n) => n.span,
            Self::Break(n) => n.span,
            Self::Link(n) => n.span,
            Self::Image(n) => n.span,
            Self::Delete(n) => n.span,
            Self::FootnoteReference(n) => n.span,
            Self::Definition(n) => n.span,
            Self::FootnoteDefinition(n) => n.span,
        }
    }
}
