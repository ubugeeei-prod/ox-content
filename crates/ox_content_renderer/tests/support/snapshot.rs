//! Shared helpers for full-document HTML snapshot tests.

use ox_content_allocator::Allocator;
use ox_content_parser::{Parser, ParserOptions};
use ox_content_renderer::{HtmlRenderer, HtmlRendererOptions};

pub fn render(
    source: &str,
    parser_options: ParserOptions,
    renderer_options: HtmlRendererOptions,
) -> String {
    let allocator = Allocator::new();
    let doc = Parser::with_options(&allocator, source, parser_options)
        .parse()
        .expect("parser should not fail on snapshot fixtures");
    let mut renderer = HtmlRenderer::with_options(renderer_options);
    renderer.render(&doc)
}

pub fn check(
    name: &str,
    source: &str,
    parser_options: ParserOptions,
    renderer_options: HtmlRendererOptions,
) {
    let html = render(source, parser_options, renderer_options);
    insta::with_settings!({
        snapshot_path => "../snapshots/renderer",
        prepend_module_to_snapshot => false,
        description => source.to_string(),
        omit_expression => true,
    }, {
        insta::assert_snapshot!(name, html);
    });
}
