//! Markdown parser implementation.

use ox_content_allocator::Allocator;
use ox_content_ast::{Document, Span};

use crate::error::ParseResult;
#[allow(unused_imports)]
// The macro is no-op without the `profile` feature, which suppresses the use.
use crate::profile_span;

mod block;
mod block_quote;
mod cursor;
mod fenced_code;
mod footnote;
mod html;
mod indented_code;
mod inline;
mod inline_helpers;
mod inline_html;
mod leaf;
mod list;
mod list_item;
mod reference;
mod spans;
mod table;

#[cfg(test)]
mod tests;

/// Parser options.
///
/// `Default::default()` keeps optional Markdown extensions disabled. Use
/// [`ParserOptions::gfm`] to enable the GitHub Flavored Markdown profile.
#[derive(Debug, Clone, Default)]
pub struct ParserOptions {
    /// Enable the GFM convenience profile.
    ///
    /// When set through [`ParserOptions::gfm`], this also enables footnotes,
    /// task lists, tables, strikethrough, and autolinks.
    ///
    /// Default: `false`; [`ParserOptions::gfm`] sets this to `true`.
    pub gfm: bool,

    /// Enable footnote references and definitions.
    ///
    /// Default: `false`; [`ParserOptions::gfm`] sets this to `true`.
    pub footnotes: bool,

    /// Enable GFM task-list item markers such as `- [x]`.
    ///
    /// Default: `false`; [`ParserOptions::gfm`] sets this to `true`.
    pub task_lists: bool,

    /// Enable GFM pipe tables.
    ///
    /// Default: `false`; [`ParserOptions::gfm`] sets this to `true`.
    pub tables: bool,

    /// Enable GFM strikethrough spans.
    ///
    /// Default: `false`; [`ParserOptions::gfm`] sets this to `true`.
    pub strikethrough: bool,

    /// Enable GFM autolinks.
    ///
    /// Default: `false`; [`ParserOptions::gfm`] sets this to `true`.
    pub autolinks: bool,

    /// Maximum nesting depth for block elements.
    ///
    /// Default: `0`; [`ParserOptions::gfm`] sets this to `100`.
    pub max_nesting_depth: usize,
}

impl ParserOptions {
    /// Creates new parser options with GFM extensions enabled.
    #[must_use]
    pub fn gfm() -> Self {
        Self {
            gfm: true,
            footnotes: true,
            task_lists: true,
            tables: true,
            strikethrough: true,
            autolinks: true,
            max_nesting_depth: 100,
        }
    }
}

/// Markdown parser.
pub struct Parser<'a> {
    /// Arena allocator.
    allocator: &'a Allocator,

    /// Source text.
    source: &'a str,

    /// Parser options.
    options: ParserOptions,

    /// Current position in the source.
    position: usize,

    /// Current nesting depth.
    nesting_depth: usize,

    /// Link reference definitions collected by the root parser's
    /// pre-pass, shared with sub-parsers (block quote and list item
    /// contents) so references resolve document-wide.
    definitions: std::rc::Rc<reference::ReferenceMap<'a>>,

    /// Footnote labels defined anywhere in the document, collected by the
    /// same kind of pre-pass as `definitions` so an inline `[^x]` can tell
    /// whether a definition exists before reaching it.
    footnote_labels: std::rc::Rc<footnote::FootnoteLabels>,

    /// Byte offsets (in `source`) of lines that entered this sub-source
    /// via lazy continuation. Such lines are paragraph text by
    /// construction and must not be reinterpreted as setext underlines
    /// during the re-parse.
    lazy_lines: std::rc::Rc<rustc_hash::FxHashSet<u32>>,
}

impl<'a> Parser<'a> {
    /// Creates a new parser with default options.
    #[must_use]
    pub fn new(allocator: &'a Allocator, source: &'a str) -> Self {
        Self::with_options(allocator, source, ParserOptions::default())
    }

    /// Creates a new parser with the specified options.
    #[must_use]
    pub fn with_options(allocator: &'a Allocator, source: &'a str, options: ParserOptions) -> Self {
        let mut parser = Self {
            allocator,
            source,
            options,
            position: 0,
            nesting_depth: 0,
            definitions: std::rc::Rc::new(reference::ReferenceMap::default()),
            footnote_labels: std::rc::Rc::default(),
            lazy_lines: std::rc::Rc::default(),
        };
        // Footnote labels first: the reference collector consults them to
        // leave `[^label]:` blocks to the footnote parser.
        parser.footnote_labels = parser.build_footnote_labels();
        parser.definitions = parser.build_definitions();
        parser
    }

    /// Creates a parser for re-parsing a stripped sub-source (block quote
    /// or list item content) that shares this parser's reference
    /// definitions instead of re-collecting them.
    /// Sub-parser that also knows which of its lines were added by lazy
    /// continuation (offsets into `source`).
    pub(crate) fn sub_parser_with_lazy_lines(
        &self,
        source: &'a str,
        lazy_lines: rustc_hash::FxHashSet<u32>,
    ) -> Parser<'a> {
        Self {
            allocator: self.allocator,
            source,
            options: self.options.clone(),
            position: 0,
            nesting_depth: 0,
            definitions: std::rc::Rc::clone(&self.definitions),
            footnote_labels: std::rc::Rc::clone(&self.footnote_labels),
            lazy_lines: std::rc::Rc::new(lazy_lines),
        }
    }

    /// Parses the source into a document AST.
    pub fn parse(mut self) -> ParseResult<Document<'a>> {
        profile_span!("parser::parse");
        let mut children = self.allocator.new_vec();

        while !self.is_at_end() {
            if let Some(node) = self.parse_block()? {
                children.push(node);
            }
        }

        let span = Span::new(0, self.source.len() as u32);
        Ok(Document { children, span })
    }
}
