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
    insta::assert_snapshot!(html);
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
    insta::assert_snapshot!(html);
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
    insta::assert_snapshot!(html);
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
    insta::assert_snapshot!(html);
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
    insta::assert_snapshot!(html);
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
    insta::assert_snapshot!(html);
}

#[test]
fn test_convert_md_link_to_child_index_file() {
    // A link to a directory's index page (./lib/index.md) names the directory
    // page itself — it must become ./lib/index.html, never ./lib/index/index.html
    // (a page that does not exist in the output tree). Same for absolute links.
    let allocator = Allocator::new();
    let doc = Parser::new(&allocator, "[Lib](./lib/index.md) [Abs](/lib/index.md)").parse().unwrap();
    let mut renderer = HtmlRenderer::with_options(HtmlRendererOptions {
        convert_md_links: true,
        base_url: "/".to_string(),
        source_path: "api/index.md".to_string(),
        ..Default::default()
    });
    let html = renderer.render(&doc);
    insta::assert_snapshot!(html);
}

#[test]
fn test_convert_md_link_to_sibling_dir_index_from_non_index() {
    // From a non-index page, ./lib/index.md resolves one level up like every
    // other ./ link, then collapses the index segment.
    let allocator = Allocator::new();
    let doc = Parser::new(&allocator, "[Lib](./lib/index.md)").parse().unwrap();
    let mut renderer = HtmlRenderer::with_options(HtmlRendererOptions {
        convert_md_links: true,
        base_url: "/".to_string(),
        source_path: "api/types.md".to_string(),
        ..Default::default()
    });
    let html = renderer.render(&doc);
    insta::assert_snapshot!(html);
}

#[test]
fn test_convert_md_href_inside_raw_html_anchor() {
    // The docs generator emits raw <a href="X.md"> anchors alongside Markdown
    // links; both must be converted or the raw ones 404 in the output tree.
    let allocator = Allocator::new();
    let doc = Parser::new(
        &allocator,
        "<a class=\"x\" href=\"../type-aliases/CounterOptions.md\">CounterOptions</a>",
    )
    .parse()
    .unwrap();
    let mut renderer = HtmlRenderer::with_options(HtmlRendererOptions {
        convert_md_links: true,
        base_url: "/".to_string(),
        source_path: "lib/functions/createCounter.md".to_string(),
        ..Default::default()
    });
    let html = renderer.render(&doc);
    insta::assert_snapshot!(html);
}
