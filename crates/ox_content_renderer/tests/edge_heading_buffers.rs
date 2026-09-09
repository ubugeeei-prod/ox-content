use ox_content_allocator::Allocator;
use ox_content_ast::Node;
use ox_content_parser::Parser;
use ox_content_renderer::{HtmlRenderer, HtmlRendererOptions};

#[test]
fn explicit_heading_ids_escape_identically_in_ids_and_permalinks() {
    let allocator = Allocator::new();
    let mut document = Parser::new(&allocator, "## Title").parse().unwrap();
    let Node::Heading(heading) = &mut document.children[0] else {
        panic!("expected heading");
    };
    heading.id = Some("日本語<&>\"'\r\n");
    let mut renderer = HtmlRenderer::with_options(HtmlRendererOptions {
        heading_permalinks: true,
        ..Default::default()
    });
    let expected = concat!(
        "<h2 id=\"日本語&lt;&amp;&gt;&quot;&#39;&#13;&#10;\">Title",
        "<a class=\"header-anchor\" href=\"#日本語&lt;&amp;&gt;&quot;&#39;&#13;&#10;\" ",
        "aria-label=\"Permalink to &quot;Title&quot;\">#</a></h2>\n"
    );
    assert_eq!(renderer.render_borrowed(&document), expected);
    assert_eq!(renderer.render(&document), expected);
}

#[test]
fn heading_buffers_do_not_leak_long_ids_or_duplicate_counts_across_renders() {
    let allocator = Allocator::new();
    let long_title = "日本語の長い見出し".repeat(64);
    let long_source = format!("## {long_title}\n\n## {long_title}");
    let long_document = Parser::new(&allocator, &long_source).parse().unwrap();
    let short_document = Parser::new(&allocator, "## A\n\n## A").parse().unwrap();
    let empty_document = Parser::new(&allocator, "").parse().unwrap();
    let options = HtmlRendererOptions { heading_permalinks: true, ..Default::default() };
    let mut renderer = HtmlRenderer::with_options(options.clone());
    let expected = concat!(
        "<h2 id=\"a\">A<a class=\"header-anchor\" href=\"#a\" ",
        "aria-label=\"Permalink to &quot;A&quot;\">#</a></h2>\n",
        "<h2 id=\"a-1\">A<a class=\"header-anchor\" href=\"#a-1\" ",
        "aria-label=\"Permalink to &quot;A&quot;\">#</a></h2>\n"
    );
    for _ in 0..3 {
        assert_eq!(
            renderer.render_borrowed(&long_document),
            HtmlRenderer::with_options(options.clone()).render(&long_document)
        );
        assert_eq!(renderer.render_borrowed(&short_document), expected);
        assert_eq!(renderer.render_borrowed(&empty_document), "");
        assert_eq!(renderer.render(&short_document), expected);
    }
}
