//! Transform context for Markdown processing.

use ox_content_allocator::Allocator;
use ox_content_ast::Document;
use ox_content_parser::{Parser, ParserOptions};
use ox_content_renderer::HtmlRenderer;

/// Transform context for processing Markdown files.
pub struct TransformContext {
    /// Parser options.
    parser_options: ParserOptions,
}

impl TransformContext {
    /// Creates a new transform context.
    #[must_use]
    pub fn new() -> Self {
        Self { parser_options: ParserOptions::gfm() }
    }

    /// Creates a new transform context with custom parser options.
    #[must_use]
    pub fn with_options(parser_options: ParserOptions) -> Self {
        Self { parser_options }
    }

    /// Parses Markdown source.
    pub fn parse<'a>(
        &self,
        allocator: &'a Allocator,
        source: &'a str,
    ) -> Result<Document<'a>, ox_content_parser::ParseError> {
        Parser::with_options(allocator, source, self.parser_options.clone()).parse()
    }

    /// Renders a document to HTML.
    #[must_use]
    pub fn render_html(&self, document: &Document<'_>) -> String {
        let mut renderer = HtmlRenderer::new();
        renderer.render(document)
    }

    /// Transforms Markdown source to HTML.
    pub fn transform(&self, source: &str) -> Result<String, ox_content_parser::ParseError> {
        let allocator = Allocator::new();
        let doc = self.parse(&allocator, source)?;
        Ok(self.render_html(&doc))
    }
}

impl Default for TransformContext {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_transform() {
        let ctx = TransformContext::new();
        let html = ctx.transform("# Hello World").unwrap();
        assert!(html.contains("<h1 id=\"hello-world\">"));
    }
}
