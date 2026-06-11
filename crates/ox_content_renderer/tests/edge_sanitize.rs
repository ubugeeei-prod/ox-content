#[path = "support/edge.rs"]
mod edge_support;

use edge_support::render;
use ox_content_allocator::Allocator;
use ox_content_parser::ParserOptions;
use ox_content_renderer::HtmlRenderer;
use ox_content_renderer::HtmlRendererOptions;

#[test]
fn html_blocks_are_escaped_when_sanitize_is_enabled() {
    let allocator = Allocator::new();
    let mut children = allocator.new_vec();
    children.push(ox_content_ast::Node::Html(ox_content_ast::Html {
        value: "<script>alert(1)</script>",
        span: ox_content_ast::Span::new(0, 25),
    }));
    let doc = ox_content_ast::Document { children, span: ox_content_ast::Span::new(0, 25) };

    let mut renderer =
        HtmlRenderer::with_options(HtmlRendererOptions { sanitize: true, ..Default::default() });
    let html = renderer.render(&doc);

    assert_eq!(html, "&lt;script&gt;alert(1)&lt;/script&gt;\n");
}

#[test]
fn unsafe_link_urls_are_neutralized_when_sanitize_is_enabled() {
    let html = render(
        "[run](javascript:alert(1))",
        ParserOptions::default(),
        HtmlRendererOptions { sanitize: true, ..Default::default() },
    );

    assert_eq!(html, "<p><a href=\"#\">run</a></p>\n");
}

#[test]
fn obfuscated_unsafe_link_schemes_are_neutralized() {
    let html = render(
        "[run](  JaVa ScRiPt:alert(1))",
        ParserOptions::default(),
        HtmlRendererOptions { sanitize: true, ..Default::default() },
    );

    assert_eq!(html, "<p><a href=\"#\">run</a></p>\n");
}

#[test]
fn unsafe_image_urls_are_cleared_when_sanitize_is_enabled() {
    let html = render(
        "![x](data:text/html,<script>alert(1)</script>)",
        ParserOptions::default(),
        HtmlRendererOptions { sanitize: true, ..Default::default() },
    );

    assert_eq!(html, "<p><img src=\"\" alt=\"x\"></p>\n");
}

#[test]
fn sanitize_keeps_relative_and_allowed_url_schemes() {
    let html = render(
        "[guide](./guide.md) [mail](mailto:hi@example.com) [phone](tel:+123)",
        ParserOptions::default(),
        HtmlRendererOptions { sanitize: true, ..Default::default() },
    );

    assert!(html.contains("href=\"./guide.md\""));
    assert!(html.contains("href=\"mailto:hi@example.com\""));
    assert!(html.contains("href=\"tel:+123\""));
}

#[test]
fn inline_raw_html_is_escaped_when_sanitize_is_enabled() {
    let html = render(
        "<span>ok</span>",
        ParserOptions::default(),
        HtmlRendererOptions { sanitize: true, ..Default::default() },
    );

    assert_eq!(html, "<p>&lt;span&gt;ok&lt;/span&gt;</p>\n");
}
