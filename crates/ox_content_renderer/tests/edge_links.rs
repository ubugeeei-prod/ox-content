#[path = "support/edge.rs"]
mod edge_support;

use edge_support::render;
use ox_content_parser::ParserOptions;
use ox_content_renderer::HtmlRendererOptions;
#[test]
fn external_links_get_security_attributes() {
    let html = render(
        "[site](https://example.com)",
        ParserOptions::default(),
        HtmlRendererOptions::default(),
    );

    assert!(html.contains("target=\"_blank\""));
    assert!(html.contains("rel=\"noopener noreferrer\""));
}

#[test]
fn relative_links_do_not_get_external_attributes() {
    let html =
        render("[guide](./guide.md)", ParserOptions::default(), HtmlRendererOptions::default());

    assert!(!html.contains("target=\"_blank\""));
    assert!(!html.contains("rel=\"noopener noreferrer\""));
}

#[test]
fn base_prefixes_root_absolute_markdown_links() {
    let html = render(
        "[Guide](/guide) [Dir](/guide/) [Markdown](/api.md#types)",
        ParserOptions::default(),
        HtmlRendererOptions {
            convert_md_links: true,
            base_url: "/docs/".to_string(),
            ..Default::default()
        },
    );

    assert!(html.contains("href=\"/docs/guide\""));
    assert!(html.contains("href=\"/docs/guide/\""));
    assert!(html.contains("href=\"/docs/api/index.html#types\""));
}

#[test]
fn base_prefixes_root_absolute_markdown_images() {
    let html = render(
        "![logo](/img/logo.png)",
        ParserOptions::default(),
        HtmlRendererOptions {
            convert_md_links: true,
            base_url: "/docs/".to_string(),
            ..Default::default()
        },
    );

    assert_eq!(html, "<p><img src=\"/docs/img/logo.png\" alt=\"logo\"></p>\n");
}

#[test]
fn base_prefixes_root_absolute_raw_html_attrs() {
    let html = render(
        "<div>\n<a href=\"/guide\">Guide</a>\n<img src='/img/logo.png'>\n<script src=\"//cdn.example/app.js\"></script>\n</div>",
        ParserOptions::default(),
        HtmlRendererOptions {
            convert_md_links: true,
            base_url: "/docs/".to_string(),
            ..Default::default()
        },
    );

    assert!(html.contains("href=\"/docs/guide\""), "{html}");
    assert!(html.contains("src='/docs/img/logo.png'"), "{html}");
    assert!(html.contains("src=\"//cdn.example/app.js\""), "{html}");
}

#[test]
fn nested_parentheses_in_links_are_preserved_in_output() {
    let html = render(
        "[docs](https://example.com/a(b)c)",
        ParserOptions::default(),
        HtmlRendererOptions::default(),
    );
    assert!(html.contains("href=\"https://example.com/a(b)c\""));
}

#[test]
fn xhtml_images_self_close() {
    let html = render(
        "![logo](/logo.svg)",
        ParserOptions::default(),
        HtmlRendererOptions { xhtml: true, ..Default::default() },
    );

    assert!(html.contains("<img src=\"/logo.svg\" alt=\"logo\" />"));
}
