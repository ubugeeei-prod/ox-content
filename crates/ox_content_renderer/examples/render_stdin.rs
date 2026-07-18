//! Diff-fuzzing helper: render stdin markdown to HTML on stdout.
use std::io::Read;

fn main() {
    let mut source = String::new();
    std::io::stdin().read_to_string(&mut source).unwrap();
    let gfm = std::env::args().any(|a| a == "--gfm");
    let allocator = ox_content_allocator::Allocator::for_source_len(source.len());
    let options = if gfm {
        ox_content_parser::ParserOptions::gfm()
    } else {
        ox_content_parser::ParserOptions::default()
    };
    let parser = ox_content_parser::Parser::with_options(&allocator, &source, options);
    let doc = parser.parse().unwrap();
    let mut renderer = ox_content_renderer::HtmlRenderer::with_options(
        ox_content_renderer::HtmlRendererOptions::new(),
    );
    print!("{}", renderer.render(&doc));
}
