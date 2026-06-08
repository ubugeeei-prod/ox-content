use napi_derive::napi;
use ox_content_incremental::{
    IncrementalHtmlRenderer as CoreIncrementalHtmlRenderer,
    IncrementalParser as CoreIncrementalParser,
    IncrementalRenderOptions as CoreIncrementalRenderOptions,
};
use ox_content_parser::ParserOptions;

use crate::incremental_result::{
    incremental_parse_error_result, incremental_render_error_result, map_incremental_render_result,
    usize_to_u32,
};
use crate::incremental_types::{
    IncrementalMarkdownParseResult, IncrementalMarkdownRenderResult, JsIncrementalParseOptions,
    JsIncrementalRenderOptions,
};
use crate::{mdast, JsParserOptions};

/// Stateful Markdown parser for append-only streams.
#[napi]
pub struct IncrementalMarkdownParser {
    inner: CoreIncrementalParser,
}

#[napi]
impl IncrementalMarkdownParser {
    /// Creates an incremental Markdown parser.
    #[napi(constructor)]
    pub fn new(options: Option<JsParserOptions>) -> Self {
        let parser_options = options.map(ParserOptions::from).unwrap_or_default();
        Self { inner: CoreIncrementalParser::new(parser_options) }
    }

    /// Appends Markdown and returns committed plus optional provisional AST JSON.
    #[napi]
    pub fn append(
        &mut self,
        chunk: String,
        options: Option<JsIncrementalParseOptions>,
    ) -> IncrementalMarkdownParseResult {
        let options = options.unwrap_or_default();
        self.append_inner(&chunk, options)
    }

    /// Finalizes the stream and commits any remaining Markdown.
    #[napi]
    pub fn finish(
        &mut self,
        options: Option<JsIncrementalParseOptions>,
    ) -> IncrementalMarkdownParseResult {
        let mut options = options.unwrap_or_default();
        options.is_final = Some(true);
        self.append_inner("", options)
    }

    /// Clears all stream state.
    #[napi]
    pub fn reset(&mut self) {
        self.inner.reset();
    }

    /// Returns the current unstable Markdown tail.
    #[napi(getter)]
    pub fn pending_markdown(&self) -> String {
        self.inner.pending_markdown().to_string()
    }

    /// Returns the number of bytes committed from the stream.
    #[napi(getter)]
    pub fn committed_bytes(&self) -> u32 {
        usize_to_u32(self.inner.committed_bytes())
    }

    /// Returns the total number of bytes observed from the stream.
    #[napi(getter)]
    pub fn total_bytes(&self) -> u32 {
        usize_to_u32(self.inner.total_bytes())
    }

    fn append_inner(
        &mut self,
        chunk: &str,
        options: JsIncrementalParseOptions,
    ) -> IncrementalMarkdownParseResult {
        let include_pending_ast = options.include_pending_ast.unwrap_or(false);
        let complete_inline = options.complete_inline.unwrap_or(true);
        let result = self.inner.append(chunk, options.is_final.unwrap_or(false), |_, _, doc| {
            mdast::to_mdast_json(doc)
        });

        match result {
            Ok(result) => self.parse_result(result, include_pending_ast, complete_inline),
            Err(error) => incremental_parse_error_result(
                self.inner.pending_markdown().to_string(),
                self.inner.committed_bytes(),
                self.inner.total_bytes(),
                self.inner.is_final(),
                error.to_string(),
            ),
        }
    }

    fn parse_result(
        &self,
        result: ox_content_incremental::IncrementalParseResult<String>,
        include_pending_ast: bool,
        complete_inline: bool,
    ) -> IncrementalMarkdownParseResult {
        let pending_ast = if include_pending_ast {
            match self.inner.parse_pending(complete_inline, |_, _, doc| mdast::to_mdast_json(doc)) {
                Ok(Some(ast)) => ast,
                Ok(None) => String::new(),
                Err(error) => {
                    return incremental_parse_error_result(
                        self.inner.pending_markdown().to_string(),
                        self.inner.committed_bytes(),
                        self.inner.total_bytes(),
                        self.inner.is_final(),
                        error.to_string(),
                    );
                }
            }
        } else {
            String::new()
        };

        IncrementalMarkdownParseResult {
            ast: result.committed.unwrap_or_default(),
            pending_ast,
            markdown: result.committed_markdown,
            pending_markdown: result.pending_markdown,
            committed_byte_start: usize_to_u32(result.committed_byte_start),
            committed_byte_end: usize_to_u32(result.committed_byte_end),
            committed_bytes: usize_to_u32(result.committed_bytes),
            pending_bytes: usize_to_u32(result.pending_bytes),
            total_bytes: usize_to_u32(result.total_bytes),
            did_commit: result.did_commit,
            is_final: result.is_final,
            errors: vec![],
        }
    }
}

/// Stateful Markdown-to-HTML renderer for append-only streams.
#[napi]
pub struct IncrementalMarkdownRenderer {
    inner: CoreIncrementalHtmlRenderer,
}

#[napi]
impl IncrementalMarkdownRenderer {
    /// Creates an incremental Markdown renderer.
    #[napi(constructor)]
    pub fn new(options: Option<JsParserOptions>) -> Self {
        let parser_options = options.map(ParserOptions::from).unwrap_or_default();
        Self { inner: CoreIncrementalHtmlRenderer::new(parser_options) }
    }

    /// Appends Markdown and returns committed delta plus provisional HTML.
    #[napi]
    pub fn append(
        &mut self,
        chunk: String,
        options: Option<JsIncrementalRenderOptions>,
    ) -> IncrementalMarkdownRenderResult {
        let options = options.unwrap_or_default();
        let render_options = CoreIncrementalRenderOptions {
            is_final: options.is_final.unwrap_or(false),
            render_pending: options.render_pending.unwrap_or(true),
            complete_inline: options.complete_inline.unwrap_or(true),
        };

        match self.inner.append(&chunk, render_options) {
            Ok(result) => map_incremental_render_result(result, Vec::new()),
            Err(error) => self.render_error_result(error.to_string()),
        }
    }

    /// Finalizes the stream and commits any remaining Markdown.
    #[napi]
    pub fn finish(&mut self) -> IncrementalMarkdownRenderResult {
        match self.inner.finish() {
            Ok(result) => map_incremental_render_result(result, Vec::new()),
            Err(error) => self.render_error_result(error.to_string()),
        }
    }

    /// Clears parser, renderer, and accumulated HTML state.
    #[napi]
    pub fn reset(&mut self) {
        self.inner.reset();
    }

    /// Returns all committed HTML so far.
    #[napi(getter)]
    pub fn committed_html(&self) -> String {
        self.inner.committed_html().to_string()
    }

    /// Returns the current unstable Markdown tail.
    #[napi(getter)]
    pub fn pending_markdown(&self) -> String {
        self.inner.pending_markdown().to_string()
    }

    fn render_error_result(&self, error: String) -> IncrementalMarkdownRenderResult {
        incremental_render_error_result(
            self.inner.committed_html().to_string(),
            self.inner.pending_markdown().to_string(),
            error,
        )
    }
}
