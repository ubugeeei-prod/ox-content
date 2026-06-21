use crate::html::HtmlRenderer;
use ox_content_allocator::Allocator;
use ox_content_parser::Parser;

#[test]
fn test_render_nested_list() {
    let allocator = Allocator::new();
    // Indent with 2 spaces for nesting
    let doc = Parser::new(&allocator, "- item 1\n  - sub 1\n- item 2").parse().unwrap();
    let mut renderer = HtmlRenderer::new();
    let html = renderer.render(&doc);

    insta::assert_snapshot!(html);
}

#[test]
fn test_render_table() {
    let allocator = Allocator::new();
    let parser_options = ox_content_parser::ParserOptions::gfm();
    let doc = Parser::with_options(&allocator, "| head |\n| --- |\n| body |", parser_options)
        .parse()
        .unwrap();
    let mut renderer = HtmlRenderer::new();
    let html = renderer.render(&doc);
    insta::assert_snapshot!(html);
}

#[test]
fn test_render_table_no_gfm() {
    let allocator = Allocator::new();
    // Default options have tables: false
    let doc = Parser::new(&allocator, "| head |\n| --- |\n| body |").parse().unwrap();
    let mut renderer = HtmlRenderer::new();
    let html = renderer.render(&doc);
    insta::assert_snapshot!(html);
}

#[test]
fn test_render_list_with_bold() {
    let allocator = Allocator::new();
    let doc = Parser::new(&allocator, "- **bold** text").parse().unwrap();
    let mut renderer = HtmlRenderer::new();
    let html = renderer.render(&doc);
    insta::assert_snapshot!(html);
}

#[test]
fn test_render_task_list() {
    let allocator = Allocator::new();
    let parser_options = ox_content_parser::ParserOptions::gfm();
    let doc = Parser::with_options(&allocator, "- [x] task 1\n- [ ] task 2", parser_options)
        .parse()
        .unwrap();
    let mut renderer = HtmlRenderer::new();
    let html = renderer.render(&doc);
    insta::assert_snapshot!(html);
}
