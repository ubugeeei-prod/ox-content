use crate::html::{HtmlRenderer, HtmlRendererOptions};
use ox_content_allocator::Allocator;
use ox_content_parser::Parser;

#[test]
fn test_convert_md_link_from_index_file() {
    // When the source is an index file (api/index.md), relative links like ./docs.md
    // should become ./docs/index.html (not ../docs/index.html)
    let allocator = Allocator::new();
    let doc = Parser::new(&allocator, "[Docs](./docs.md)").parse().unwrap();
    let mut renderer = HtmlRenderer::with_options(HtmlRendererOptions {
        convert_md_links: true,
        base_url: "/".to_string(),
        source_path: "api/index.md".to_string(),
        ..Default::default()
    });
    let html = renderer.render(&doc);
    assert!(
        html.contains("href=\"./docs/index.html\""),
        "Expected ./docs/index.html but got: {html}"
    );
}

#[test]
fn test_convert_md_link_from_non_index_file() {
    // When the source is NOT an index file (api/types.md -> becomes types/index.html),
    // relative links like ./docs.md should become ../docs/index.html
    let allocator = Allocator::new();
    let doc = Parser::new(&allocator, "[Docs](./docs.md)").parse().unwrap();
    let mut renderer = HtmlRenderer::with_options(HtmlRendererOptions {
        convert_md_links: true,
        base_url: "/".to_string(),
        source_path: "api/types.md".to_string(),
        ..Default::default()
    });
    let html = renderer.render(&doc);
    assert!(
        html.contains("href=\"../docs/index.html\""),
        "Expected ../docs/index.html but got: {html}"
    );
}

#[test]
fn test_convert_md_link_plain_relative_from_index() {
    // Plain relative links (no ./) from index file
    let allocator = Allocator::new();
    let doc = Parser::new(&allocator, "[Types](types.md)").parse().unwrap();
    let mut renderer = HtmlRenderer::with_options(HtmlRendererOptions {
        convert_md_links: true,
        base_url: "/".to_string(),
        source_path: "api/index.md".to_string(),
        ..Default::default()
    });
    let html = renderer.render(&doc);
    assert!(
        html.contains("href=\"./types/index.html\""),
        "Expected ./types/index.html but got: {html}"
    );
}

#[test]
fn test_convert_mdx_and_markdown_links() {
    let allocator = Allocator::new();
    let doc = Parser::new(&allocator, "[Component](./component.mdx) [Guide](guide.markdown)")
        .parse()
        .unwrap();
    let mut renderer = HtmlRenderer::with_options(HtmlRendererOptions {
        convert_md_links: true,
        base_url: "/".to_string(),
        source_path: "api/index.mdx".to_string(),
        ..Default::default()
    });
    let html = renderer.render(&doc);
    assert!(html.contains("href=\"./component/index.html\""), "Got: {html}");
    assert!(html.contains("href=\"./guide/index.html\""), "Got: {html}");
}

#[test]
fn test_convert_md_link_parent_relative_from_index() {
    // Parent-relative links from index file
    let allocator = Allocator::new();
    let doc = Parser::new(&allocator, "[Guide](../guide.md)").parse().unwrap();
    let mut renderer = HtmlRenderer::with_options(HtmlRendererOptions {
        convert_md_links: true,
        base_url: "/".to_string(),
        source_path: "api/index.md".to_string(),
        ..Default::default()
    });
    let html = renderer.render(&doc);
    assert!(
        html.contains("href=\"../guide/index.html\""),
        "Expected ../guide/index.html but got: {html}"
    );
}

#[test]
fn test_convert_md_link_parent_relative_from_non_index() {
    // Parent-relative links from non-index file need extra ../
    let allocator = Allocator::new();
    let doc = Parser::new(&allocator, "[Guide](../guide.md)").parse().unwrap();
    let mut renderer = HtmlRenderer::with_options(HtmlRendererOptions {
        convert_md_links: true,
        base_url: "/".to_string(),
        source_path: "api/types.md".to_string(),
        ..Default::default()
    });
    let html = renderer.render(&doc);
    assert!(
        html.contains("href=\"../../guide/index.html\""),
        "Expected ../../guide/index.html but got: {html}"
    );
}
