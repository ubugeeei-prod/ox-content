use std::borrow::Cow;

use ox_content_allocator::Allocator;
use ox_content_ast::Document;
use ox_content_parser::{ParseResult, Parser, ParserOptions};

use crate::boundary::stable_prefix_len;
use crate::provisional::complete_provisional_markdown;
use crate::result::IncrementalParseResult;

/// Append-only incremental Markdown parser.
///
/// The parser commits only stable prefixes. The unstable tail remains available
/// for provisional parsing so a UI can render partial headings, emphasis, and
/// other inline formatting while still replacing that provisional region on the
/// next append.
#[derive(Debug, Clone)]
pub struct IncrementalParser {
    options: ParserOptions,
    pending: String,
    committed_bytes: usize,
    total_bytes: usize,
    is_final: bool,
}

impl IncrementalParser {
    /// Creates an incremental parser with parser options.
    #[must_use]
    pub fn new(options: ParserOptions) -> Self {
        Self {
            options,
            pending: String::new(),
            committed_bytes: 0,
            total_bytes: 0,
            is_final: false,
        }
    }

    /// Returns parser options used for each committed fragment parse.
    #[must_use]
    pub fn options(&self) -> &ParserOptions {
        &self.options
    }

    /// Appends a Markdown chunk and parses the next stable prefix, if any.
    ///
    /// `map` is called while the arena-backed document is alive. This avoids
    /// returning self-referential AST values while keeping parsing in Rust.
    pub fn append<T>(
        &mut self,
        chunk: &str,
        is_final: bool,
        map: impl FnOnce(&str, usize, &Document<'_>) -> T,
    ) -> ParseResult<IncrementalParseResult<T>> {
        if self.is_final {
            self.reset();
        }

        self.pending.push_str(chunk);
        self.total_bytes = self.total_bytes.saturating_add(chunk.len());
        self.is_final = is_final;

        let commit_len =
            if is_final { self.pending.len() } else { stable_prefix_len(&self.pending) };

        if commit_len == 0 {
            return Ok(IncrementalParseResult::empty(
                self.pending.clone(),
                self.committed_bytes,
                self.pending.len(),
                self.total_bytes,
                self.is_final,
            ));
        }

        let committed_markdown = self.pending[..commit_len].to_string();
        let committed_byte_start = self.committed_bytes;
        let committed = {
            let allocator = Allocator::for_source_len(committed_markdown.len());
            let parser =
                Parser::with_options(&allocator, &committed_markdown, self.options.clone());
            let document = parser.parse()?;
            map(&committed_markdown, committed_byte_start, &document)
        };

        self.pending.drain(..commit_len);
        self.committed_bytes = self.committed_bytes.saturating_add(commit_len);

        Ok(IncrementalParseResult {
            committed: Some(committed),
            committed_markdown,
            committed_byte_start,
            committed_byte_end: self.committed_bytes,
            committed_bytes: commit_len,
            pending_markdown: self.pending.clone(),
            pending_bytes: self.pending.len(),
            total_bytes: self.total_bytes,
            did_commit: true,
            is_final: self.is_final,
        })
    }

    /// Finalizes the stream and parses any remaining tail as committed input.
    pub fn finish<T>(
        &mut self,
        map: impl FnOnce(&str, usize, &Document<'_>) -> T,
    ) -> ParseResult<IncrementalParseResult<T>> {
        self.append("", true, map)
    }

    /// Parses the unstable tail for provisional UI rendering.
    ///
    /// When `complete_inline` is true, unmatched inline delimiters are closed
    /// in a temporary copy before parsing so partial `**strong`, `_emphasis`,
    /// `` `code ``, and `~~delete` spans can render progressively.
    pub fn parse_pending<T>(
        &self,
        complete_inline: bool,
        map: impl FnOnce(&str, usize, &Document<'_>) -> T,
    ) -> ParseResult<Option<T>> {
        if self.pending.is_empty() {
            return Ok(None);
        }

        let source = if complete_inline {
            complete_provisional_markdown(&self.pending, &self.options)
        } else {
            Cow::Borrowed(self.pending.as_str())
        };
        let allocator = Allocator::for_source_len(source.len());
        let parser = Parser::with_options(&allocator, &source, self.options.clone());
        let document = parser.parse()?;
        Ok(Some(map(&source, self.committed_bytes, &document)))
    }

    /// Clears all stream state.
    pub fn reset(&mut self) {
        self.pending.clear();
        self.committed_bytes = 0;
        self.total_bytes = 0;
        self.is_final = false;
    }

    /// Returns the current unstable Markdown tail.
    #[must_use]
    pub fn pending_markdown(&self) -> &str {
        &self.pending
    }

    /// Returns the number of bytes committed from the stream.
    #[must_use]
    pub fn committed_bytes(&self) -> usize {
        self.committed_bytes
    }

    /// Returns the total number of bytes observed from the stream.
    #[must_use]
    pub fn total_bytes(&self) -> usize {
        self.total_bytes
    }

    /// Returns whether the stream has been finalized.
    #[must_use]
    pub fn is_final(&self) -> bool {
        self.is_final
    }
}

impl Default for IncrementalParser {
    fn default() -> Self {
        Self::new(ParserOptions::default())
    }
}
