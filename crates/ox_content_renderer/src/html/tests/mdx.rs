use ox_content_allocator::Allocator;
use ox_content_parser::{Parser, ParserOptions};

use crate::html::HtmlRenderer;

fn render_mdx(source: &str) -> String {
    let allocator = Allocator::new();
    let doc = Parser::with_options(&allocator, source, ParserOptions::mdx())
        .parse()
        .expect("parser should not fail on MDX render fixtures");
    HtmlRenderer::new().render(&doc)
}

#[test]
fn prose_expressions_do_not_emit_javascript() {
    let html = render_mdx("Hello {name}.\n\n{count + 1}\n");
    assert!(!html.contains("<script"), "pages without islands stay JS-free:\n{html}");
    assert!(!html.contains("data-ox-island"), "expressions are not islands:\n{html}");
    assert!(!html.contains("{name}"), "expression source is not leaked as text:\n{html}");
    assert!(!html.contains("{count + 1}"), "flow source is not leaked as text:\n{html}");
}

#[test]
fn hostile_expression_does_not_emit_raw_markup() {
    let html = render_mdx("{ \"<script>alert(1)</script>\" }\n");
    assert!(!html.contains("<script"), "hostile source must not render as HTML:\n{html}");
    assert!(!html.contains("alert(1)"), "expression source is not evaluated or emitted:\n{html}");
}
