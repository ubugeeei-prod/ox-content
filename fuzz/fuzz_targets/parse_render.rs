#![no_main]

//! Fuzz target: full parse-then-render pipeline must never panic.
//!
//! Splits the input into a tiny prefix byte that picks a parser-option preset
//! and a renderer-option preset, with the rest interpreted as Markdown source.
//! This exercises more of the option matrix in a single corpus.

use libfuzzer_sys::fuzz_target;
use ox_content_allocator::Allocator;
use ox_content_parser::{Parser, ParserOptions};
use ox_content_renderer::{HtmlRenderer, HtmlRendererOptions};

fuzz_target!(|data: &[u8]| {
    let (parser_options, renderer_options, body) = derive_options(data);
    let Ok(source) = std::str::from_utf8(body) else {
        return;
    };
    let allocator = Allocator::new();
    let Ok(doc) = Parser::with_options(&allocator, source, parser_options).parse() else {
        return;
    };
    let mut renderer = HtmlRenderer::with_options(renderer_options);
    let _ = renderer.render(&doc);
});

fn derive_options(data: &[u8]) -> (ParserOptions, HtmlRendererOptions, &[u8]) {
    let (parser, renderer, rest) = match data {
        [parser, renderer, rest @ ..] => (*parser, *renderer, rest),
        _ => (0, 0, data),
    };

    let parser_options = if parser & 1 != 0 { ParserOptions::gfm() } else { ParserOptions::default() };

    let mut renderer_options = HtmlRendererOptions::default();
    if renderer & 0b0001 != 0 {
        renderer_options.sanitize = true;
    }
    if renderer & 0b0010 != 0 {
        renderer_options.xhtml = true;
    }
    if renderer & 0b0100 != 0 {
        renderer_options.convert_md_links = true;
        renderer_options.base_url = "/docs/".to_string();
    }
    if renderer & 0b1000 != 0 {
        renderer_options.highlight = true;
    }
    if renderer & 0b0001_0000 != 0 {
        renderer_options.disallow_raw_html = true;
    }

    (parser_options, renderer_options, rest)
}
