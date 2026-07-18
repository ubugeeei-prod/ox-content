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
        "[run](JaVaScRiPt:alert(1))",
        ParserOptions::default(),
        HtmlRendererOptions { sanitize: true, ..Default::default() },
    );

    assert_eq!(html, "<p><a href=\"#\">run</a></p>\n");
}

#[test]
fn whitespace_destinations_do_not_become_links_at_all() {
    // Per CommonMark, whitespace in a bare destination means the bracketed
    // run is not a link. The former scheme obfuscation via embedded spaces
    // now yields plain text — there is no href left to sanitize.
    let html = render(
        "[run](  JaVa ScRiPt:alert(1))",
        ParserOptions::default(),
        HtmlRendererOptions { sanitize: true, ..Default::default() },
    );

    assert!(!html.contains("<a"), "no anchor should be emitted: {html}");
    assert_eq!(html, "<p>[run](  JaVa ScRiPt:alert(1))</p>\n");
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

    insta::assert_snapshot!(html);
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
