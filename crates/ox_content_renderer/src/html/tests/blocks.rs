use crate::html::{HtmlRenderer, HtmlRendererOptions};
use ox_content_allocator::Allocator;
use ox_content_parser::Parser;

#[test]
fn test_render_paragraph() {
    let allocator = Allocator::new();
    let doc = Parser::new(&allocator, "Hello world").parse().unwrap();
    let mut renderer = HtmlRenderer::new();
    let html = renderer.render(&doc);
    assert_eq!(html, "<p>Hello world</p>\n");
}

#[test]
fn test_render_heading() {
    let allocator = Allocator::new();
    let doc = Parser::new(&allocator, "# Hello").parse().unwrap();
    let mut renderer = HtmlRenderer::new();
    let html = renderer.render(&doc);
    assert_eq!(html, "<h1 id=\"hello\">Hello</h1>\n");
}

#[test]
fn test_render_heading_ids_are_unique_and_unicode() {
    let allocator = Allocator::new();
    let doc = Parser::new(&allocator, "## はじめに\n## はじめに").parse().unwrap();
    let mut renderer = HtmlRenderer::new();
    let html = renderer.render(&doc);
    insta::assert_snapshot!(html);
}

#[test]
fn test_render_heading_id_uses_inline_text() {
    let allocator = Allocator::new();
    let doc = Parser::new(&allocator, "## **API** `Index` [Guide](./guide.md)").parse().unwrap();
    let mut renderer = HtmlRenderer::new();
    let html = renderer.render(&doc);
    insta::assert_snapshot!(html);
}

fn render_with_permalinks(source: &str) -> String {
    let allocator = Allocator::new();
    let doc = Parser::new(&allocator, source).parse().unwrap();
    let mut renderer = HtmlRenderer::with_options(HtmlRendererOptions {
        heading_permalinks: true,
        ..Default::default()
    });
    renderer.render(&doc)
}

#[test]
fn test_heading_permalinks_default_off_keeps_html() {
    let allocator = Allocator::new();
    let doc = Parser::new(&allocator, "# Hello").parse().unwrap();
    let html = HtmlRenderer::new().render(&doc);
    assert_eq!(html, "<h1 id=\"hello\">Hello</h1>\n");
}

#[test]
fn test_heading_permalinks_reuse_generated_id() {
    let html = render_with_permalinks("# Hello World");
    assert_eq!(
        html,
        "<h1 id=\"hello-world\">Hello World<a class=\"header-anchor\" href=\"#hello-world\" aria-label=\"Permalink to &quot;Hello World&quot;\">#</a></h1>\n"
    );
}

#[test]
fn test_heading_permalinks_unicode_and_duplicates() {
    let html = render_with_permalinks("## はじめに\n## はじめに");
    insta::assert_snapshot!(html);
}

#[test]
fn test_heading_permalinks_skip_existing_hash_link() {
    let html = render_with_permalinks("## Hello [#](#hello)");
    assert!(html.contains("<h2 id=\"hello\">"), "{html}");
    assert_eq!(html.matches("href=\"#hello\"").count(), 1, "{html}");
    assert!(!html.contains("class=\"header-anchor\""), "{html}");
}

#[test]
fn test_heading_permalinks_skip_existing_header_anchor_html() {
    let html = render_with_permalinks(
        "## Hello <a class=\"header-anchor\" href=\"#hello\" aria-label=\"Permalink to &quot;Hello&quot;\">#</a>",
    );
    assert_eq!(html.matches("class=\"header-anchor\"").count(), 1, "{html}");
}

#[test]
fn test_heading_permalinks_empty_heading_uses_section_id() {
    let html = render_with_permalinks("#");
    assert!(
        html.contains("<h1 id=\"section\"><a class=\"header-anchor\" href=\"#section\" aria-label=\"Permalink to this section\">#</a></h1>"),
        "{html}"
    );
}

#[test]
fn test_heading_permalinks_are_real_links_without_js() {
    let html = render_with_permalinks("## API");
    assert!(html.contains("<a class=\"header-anchor\" href=\"#api\""), "{html}");
    assert!(!html.contains("onclick="), "{html}");
    assert!(!html.contains("<script"), "{html}");
}

#[test]
fn test_render_inline_toc_directive() {
    let allocator = Allocator::new();
    let doc = Parser::new(&allocator, "# Title\n\n[[toc]]\n\n## Intro\n### API").parse().unwrap();
    let mut renderer = HtmlRenderer::new();
    let html = renderer.render(&doc);

    insta::assert_snapshot!(html);
}

#[test]
fn test_render_inline_toc_uses_unique_and_unicode_ids() {
    let allocator = Allocator::new();
    let doc =
        Parser::new(&allocator, "[[toc]]\n\n## Setup\n## Setup\n## はじめに").parse().unwrap();
    let mut renderer = HtmlRenderer::new();
    let html = renderer.render(&doc);

    insta::assert_snapshot!(html);
}

#[test]
fn test_render_inline_toc_directive_is_case_insensitive() {
    // The directive names itself; a page written `[[TOC]]` meant the same
    // thing and used to ship the literal text instead of the outline.
    for marker in ["[[toc]]", "[[Toc]]", "[[TOC]]", "[[tOc]]"] {
        let allocator = Allocator::new();
        let source = format!("# Title\n\n{marker}\n\n## Intro");
        let doc = Parser::new(&allocator, &source).parse().unwrap();
        let html = HtmlRenderer::new().render(&doc);

        assert!(html.contains(r#"<nav class="ox-toc""#), "{marker}: {html}");
        assert!(!html.contains(marker), "{marker}: {html}");
    }
}

#[test]
fn test_render_inline_toc_requires_standalone_text() {
    let allocator = Allocator::new();
    let doc = Parser::new(&allocator, "See [[toc]] here\n\n`[[toc]]`\n\n## Intro").parse().unwrap();
    let mut renderer = HtmlRenderer::new();
    let html = renderer.render(&doc);

    insta::assert_snapshot!(html);
}

#[test]
fn test_render_inline_toc_marker_is_suppressed_when_no_headings() {
    // When the document contains `[[toc]]` but no headings (so
    // `toc_entries` is empty), the marker paragraph must still be
    // suppressed from output — otherwise the literal `[[toc]]`
    // leaks through as `<p>[[toc]]</p>`. Regression coverage for
    // the lazy-TOC optimization.
    let allocator = Allocator::new();
    let doc = Parser::new(&allocator, "[[toc]]").parse().unwrap();
    let mut renderer = HtmlRenderer::new();
    let html = renderer.render(&doc);

    assert_eq!(html, "");
}

#[test]
fn test_render_inline_toc_marker_is_suppressed_when_filtered_by_depth() {
    // `toc_max_depth: 0` filters every heading out, but the marker
    // paragraph should still be consumed so it doesn't leak.
    let allocator = Allocator::new();
    let doc = Parser::new(&allocator, "[[toc]]\n\n## Intro").parse().unwrap();
    let mut renderer =
        HtmlRenderer::with_options(HtmlRendererOptions { toc_max_depth: 0, ..Default::default() });
    let html = renderer.render(&doc);

    insta::assert_snapshot!(html);
}

#[test]
fn test_render_inline_toc_honors_max_depth() {
    let allocator = Allocator::new();
    let doc = Parser::new(&allocator, "[[toc]]\n\n# Title\n## Intro\n### API").parse().unwrap();
    let mut renderer =
        HtmlRenderer::with_options(HtmlRendererOptions { toc_max_depth: 2, ..Default::default() });
    let html = renderer.render(&doc);

    insta::assert_snapshot!(html);
}

#[test]
fn test_render_block_quote() {
    let allocator = Allocator::new();
    let doc = Parser::new(&allocator, "> Hello world").parse().unwrap();
    let mut renderer = HtmlRenderer::new();
    let html = renderer.render(&doc);
    assert_eq!(html, "<blockquote>\n<p>Hello world</p>\n</blockquote>\n");
}

#[test]
fn test_render_block_quote_with_inline() {
    let allocator = Allocator::new();
    let doc = Parser::new(&allocator, "> **Note:** This is important").parse().unwrap();
    let mut renderer = HtmlRenderer::new();
    let html = renderer.render(&doc);
    insta::assert_snapshot!(html);
}

#[test]
fn test_render_github_style_important_callout() {
    let allocator = Allocator::new();
    let doc = Parser::new(&allocator, "> [!IMPORTANT]\n> This is important.").parse().unwrap();
    let mut renderer = HtmlRenderer::new();
    let html = renderer.render(&doc);

    insta::assert_snapshot!(html);
}

#[test]
fn test_render_github_style_callout_with_inline_content_after_marker() {
    let allocator = Allocator::new();
    let doc = Parser::new(&allocator, "> [!NOTE] Supports **inline** content").parse().unwrap();
    let mut renderer = HtmlRenderer::new();
    let html = renderer.render(&doc);

    insta::assert_snapshot!(html);
}
