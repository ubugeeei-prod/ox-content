use ox_content_allocator::Allocator;
use ox_content_parser::{Parser, ParserOptions};
use ox_content_renderer::{HtmlRenderer, HtmlRendererOptions};

pub fn render(
    source: &str,
    parser_options: ParserOptions,
    renderer_options: HtmlRendererOptions,
) -> String {
    let allocator = Allocator::new();
    let doc = Parser::with_options(&allocator, source, parser_options).parse().unwrap();
    let mut renderer = HtmlRenderer::with_options(renderer_options);
    renderer.render(&doc)
}
