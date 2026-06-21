use crate::html::{HtmlRenderer, HtmlRendererOptions};
use ox_content_allocator::Allocator;
use ox_content_parser::Parser;

#[test]
fn test_render_heading_with_link() {
    let allocator = Allocator::new();
    let doc = Parser::new(&allocator, "### [index](./index-module.md)").parse().unwrap();
    let mut renderer = HtmlRenderer::new();
    let html = renderer.render(&doc);
    assert_eq!(html, "<h3 id=\"index\"><a href=\"./index-module.md\">index</a></h3>\n");
}

#[test]
fn test_render_strikethrough() {
    let allocator = Allocator::new();
    let doc = Parser::with_options(&allocator, "~~done~~", ox_content_parser::ParserOptions::gfm())
        .parse()
        .unwrap();
    let mut renderer = HtmlRenderer::new();
    let html = renderer.render(&doc);
    assert_eq!(html, "<p><del>done</del></p>\n");
}

#[test]
fn test_render_hard_break() {
    let allocator = Allocator::new();
    let doc = Parser::new(&allocator, "line 1\\\nline 2").parse().unwrap();
    let mut renderer = HtmlRenderer::new();
    let html = renderer.render(&doc);
    assert_eq!(html, "<p>line 1<br>\nline 2</p>\n");
}

#[test]
fn test_render_image() {
    let allocator = Allocator::new();
    let doc = Parser::new(&allocator, "![Alt text](/path/to/image.png)").parse().unwrap();
    let mut renderer = HtmlRenderer::new();
    let html = renderer.render(&doc);
    insta::assert_snapshot!(html);
}

#[test]
fn test_render_image_xhtml() {
    let allocator = Allocator::new();
    let doc = Parser::new(&allocator, "![Logo](/logo.svg)").parse().unwrap();
    let mut renderer =
        HtmlRenderer::with_options(HtmlRendererOptions { xhtml: true, ..Default::default() });
    let html = renderer.render(&doc);
    insta::assert_snapshot!(html);
}
