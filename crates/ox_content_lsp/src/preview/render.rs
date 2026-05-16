use ox_content_allocator::Allocator;
use ox_content_parser::{ParseError, Parser, ParserOptions};
use ox_content_renderer::HtmlRenderer;
use serde::Serialize;

use crate::document::TextDocumentState;
use crate::frontmatter::parse_frontmatter;
use crate::preview::html::wrap_preview_html;
use crate::preview::text::preview_title;

#[derive(Clone, Debug, Serialize)]
pub struct PreviewPayload {
    pub html: String,
    pub title: String,
}

pub fn render_preview(source: &str) -> Result<PreviewPayload, ParseError> {
    let document = TextDocumentState::new(source.to_string());
    let frontmatter = parse_frontmatter(&document);
    let block = frontmatter.block;
    let content = block
        .as_ref()
        .map_or(source, |block| &source[block.content_start_offset..block.content_end_offset]);

    let allocator = Allocator::new();
    let parser = Parser::with_options(&allocator, content, ParserOptions::gfm());
    let ast = parser.parse()?;
    let mut renderer = HtmlRenderer::new();
    let body = renderer.render(&ast);
    let title = preview_title(block.as_ref(), &ast.children)
        .unwrap_or_else(|| "Ox Content Preview".to_string());

    Ok(PreviewPayload { title: title.clone(), html: wrap_preview_html(&title, &body) })
}
