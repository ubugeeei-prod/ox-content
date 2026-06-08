use ox_content_parser::{ParseResult, ParserOptions};
use ox_content_renderer::HtmlRenderer;

use crate::parser::IncrementalParser;
use crate::result::{IncrementalParseResult, IncrementalRenderOptions, IncrementalRenderResult};

/// Incremental Markdown-to-HTML renderer for append-only streams.
pub struct IncrementalHtmlRenderer {
    parser: IncrementalParser,
    renderer: HtmlRenderer,
    committed_html: String,
}

impl IncrementalHtmlRenderer {
    /// Creates an incremental renderer with parser options.
    #[must_use]
    pub fn new(parser_options: ParserOptions) -> Self {
        Self {
            parser: IncrementalParser::new(parser_options),
            renderer: HtmlRenderer::new(),
            committed_html: String::new(),
        }
    }

    /// Appends Markdown and returns committed delta plus provisional pending HTML.
    pub fn append(
        &mut self,
        chunk: &str,
        options: IncrementalRenderOptions,
    ) -> ParseResult<IncrementalRenderResult> {
        let parser = &mut self.parser;
        let renderer = &mut self.renderer;
        let mut outcome = parser.append(chunk, options.is_final, |_, _, document| {
            renderer.render_incremental_fragment(document)
        })?;

        let delta_html = outcome.committed.take().unwrap_or_default();
        if !delta_html.is_empty() {
            self.committed_html.push_str(&delta_html);
        }

        self.build_render_result(delta_html, outcome, options)
    }

    /// Finalizes the stream and commits any remaining Markdown.
    pub fn finish(&mut self) -> ParseResult<IncrementalRenderResult> {
        self.append(
            "",
            IncrementalRenderOptions {
                is_final: true,
                render_pending: false,
                complete_inline: true,
            },
        )
    }

    /// Clears parser, renderer, and accumulated HTML state.
    pub fn reset(&mut self) {
        self.parser.reset();
        self.renderer.reset_incremental_state();
        self.committed_html.clear();
    }

    /// Returns all committed HTML so far.
    #[must_use]
    pub fn committed_html(&self) -> &str {
        &self.committed_html
    }

    /// Returns the current unstable Markdown tail.
    #[must_use]
    pub fn pending_markdown(&self) -> &str {
        self.parser.pending_markdown()
    }

    fn build_render_result(
        &mut self,
        delta_html: String,
        outcome: IncrementalParseResult<String>,
        options: IncrementalRenderOptions,
    ) -> ParseResult<IncrementalRenderResult> {
        let pending_html = if options.render_pending {
            self.parser
                .parse_pending(options.complete_inline, |_, _, document| {
                    self.renderer.render_provisional_fragment(document)
                })?
                .unwrap_or_default()
        } else {
            String::new()
        };

        let html = if pending_html.is_empty() {
            self.committed_html.clone()
        } else {
            let mut html = String::with_capacity(self.committed_html.len() + pending_html.len());
            html.push_str(&self.committed_html);
            html.push_str(&pending_html);
            html
        };

        Ok(IncrementalRenderResult {
            delta_html,
            committed_html: self.committed_html.clone(),
            pending_html,
            html,
            committed_markdown: outcome.committed_markdown,
            pending_markdown: outcome.pending_markdown,
            committed_byte_start: outcome.committed_byte_start,
            committed_byte_end: outcome.committed_byte_end,
            committed_bytes: outcome.committed_bytes,
            pending_bytes: outcome.pending_bytes,
            total_bytes: outcome.total_bytes,
            did_commit: outcome.did_commit,
            is_final: outcome.is_final,
        })
    }
}

impl Default for IncrementalHtmlRenderer {
    fn default() -> Self {
        Self::new(ParserOptions::default())
    }
}
