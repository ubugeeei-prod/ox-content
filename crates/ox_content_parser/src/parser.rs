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
mod line_scan;
mod list;
mod list_item;
mod mdx_jsx;
mod prepass;
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

    /// Recognize emphasis whose delimiters sit against East Asian punctuation.
    ///
    /// CommonMark decides whether a `*`/`_` run may open or close from the
    /// characters on either side, and punctuation on the outside blocks the
    /// run. The rule reads Unicode punctuation as a whole, so East Asian
    /// punctuation blocks it too — and because CJK text sets punctuation
    /// directly against the words it follows, `A**強調。**B` leaves the
    /// delimiters as literal text. Latin text rarely hits this, since a space
    /// usually separates the punctuation from the delimiter.
    ///
    /// With this enabled, East Asian punctuation is classified as an ordinary
    /// character for flanking purposes only, which lets those runs pair. It is
    /// off by default because it is a deliberate deviation: the parser renders
    /// every CommonMark 0.31.2 example per spec with it off.
    ///
    /// Default: `false`.
    pub cjk_emphasis: bool,

    /// Enable MDX. Off by default.
    ///
    /// When set, PascalCase JSX elements parse as [`ox_content_ast::MdxJsxFlowElement`]
    /// / [`ox_content_ast::MdxJsxTextElement`]. `import` / `export` and
    /// `{expression}` children are not parsed yet. Lowercase HTML stays HTML.
    ///
    /// Default: `false`.
    pub mdx: bool,

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
            // Not part of GFM: GitHub renders these runs per CommonMark too.
            cjk_emphasis: false,
            mdx: false,
            max_nesting_depth: 100,
        }
    }

    /// Creates parser options with MDX enabled and GFM left off.
    #[must_use]
    pub fn mdx() -> Self {
        Self { mdx: true, max_nesting_depth: 100, ..Self::default() }
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
    ///
    /// `None` and an empty map mean the same thing to every reader; the
    /// distinction exists so that documents without definitions — the
    /// overwhelming majority, and the ones where per-call cost is most
    /// visible — never pay for the `Rc` allocation at all. The same applies
    /// to the two collections below.
    definitions: Option<std::rc::Rc<reference::ReferenceMap<'a>>>,

    /// Footnote labels defined anywhere in the document, collected by the
    /// same kind of pre-pass as `definitions` so an inline `[^x]` can tell
    /// whether a definition exists before reaching it.
    footnote_labels: Option<std::rc::Rc<footnote::FootnoteLabels>>,

    /// Byte offsets (in `source`) of lines that entered this sub-source
    /// via lazy continuation. Such lines are paragraph text by
    /// construction and must not be reinterpreted as setext underlines
    /// during the re-parse.
    lazy_lines: Option<std::rc::Rc<rustc_hash::FxHashSet<u32>>>,
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
            definitions: None,
            footnote_labels: None,
            lazy_lines: None,
        };
        // A single fused pre-pass collects both the reference definitions
        // and the footnote labels (see `prepass.rs`).
        let (definitions, footnote_labels) = parser.build_prepass();
        parser.definitions = definitions;
        parser.footnote_labels = footnote_labels;
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
            definitions: self.definitions.clone(),
            footnote_labels: self.footnote_labels.clone(),
            // Most sub-sources are entered without any lazy continuation
            // line, and every block quote and list item builds one of these.
            lazy_lines: (!lazy_lines.is_empty()).then(|| std::rc::Rc::new(lazy_lines)),
        }
    }

    /// Parses the source into a document AST.
    pub fn parse(mut self) -> ParseResult<Document<'a>> {
        profile_span!("parser::parse");
        let mut children = self
            .allocator
            .new_vec_with_capacity(Self::document_children_capacity(self.source.len()));

        while !self.is_at_end() {
            if let Some(node) = self.parse_block()? {
                children.push(node);
            }
        }

        let span = Span::new(0, self.source.len() as u32);
        Ok(Document { children, span })
    }

    /// Slots to reserve for the document's top-level block list.
    ///
    /// The published sample (and concatenations of it) lands a block every
    /// ~40 bytes. Inline children already reserve from a similar density
    /// heuristic; the root list used to grow from zero, which recopied the
    /// whole array through the doubling ladder on every large parse. One
    /// vector per document, so over-reserve is cheap compared to that copy.
    fn document_children_capacity(source_len: usize) -> usize {
        const BYTES_PER_BLOCK: usize = 40;
        (source_len / BYTES_PER_BLOCK).max(4)
    }
}
