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
mod html;
mod inline;
mod inline_helpers;
mod inline_html;
mod leaf;
mod list;
mod list_item;
mod spans;
mod table;

#[cfg(test)]
mod tests;

/// Parser options.
#[derive(Debug, Clone, Default)]
pub struct ParserOptions {
    /// Enable GFM (GitHub Flavored Markdown) extensions.
    pub gfm: bool,
    /// Enable footnotes.
    pub footnotes: bool,
    /// Enable task lists.
    pub task_lists: bool,
    /// Enable tables.
    pub tables: bool,
    /// Enable strikethrough.
    pub strikethrough: bool,
    /// Enable autolinks.
    pub autolinks: bool,
    /// Maximum nesting depth for block elements.
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
}

impl<'a> Parser<'a> {
    /// Creates a new parser with default options.
    #[must_use]
    pub fn new(allocator: &'a Allocator, source: &'a str) -> Self {
        Self { allocator, source, options: ParserOptions::default(), position: 0, nesting_depth: 0 }
    }

    /// Creates a new parser with the specified options.
    #[must_use]
    pub fn with_options(allocator: &'a Allocator, source: &'a str, options: ParserOptions) -> Self {
        Self { allocator, source, options, position: 0, nesting_depth: 0 }
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
