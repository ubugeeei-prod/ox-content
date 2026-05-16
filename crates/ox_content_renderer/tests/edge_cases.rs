use ox_content_allocator::Allocator;
use ox_content_parser::{Parser, ParserOptions};
use ox_content_renderer::{HtmlRenderer, HtmlRendererOptions};

fn render(
    source: &str,
    parser_options: ParserOptions,
    renderer_options: HtmlRendererOptions,
) -> String {
    let allocator = Allocator::new();
    let doc = Parser::with_options(&allocator, source, parser_options).parse().unwrap();
    let mut renderer = HtmlRenderer::with_options(renderer_options);
    renderer.render(&doc)
}

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
fn ordered_lists_preserve_start_attribute() {
    let html =
        render("3. third\n4. fourth", ParserOptions::default(), HtmlRendererOptions::default());
    assert!(html.starts_with("<ol start=\"3\">"));
}

#[test]
fn task_list_without_feature_renders_literal_text() {
    let html = render("- [x] done", ParserOptions::default(), HtmlRendererOptions::default());
    assert!(!html.contains("type=\"checkbox\""));
    assert!(html.contains("[x] done"));
}

#[test]
fn aligned_tables_render_align_attributes() {
    let html = render(
        "| a | b | c |\n| :-- | :-: | --: |\n| 1 | 2 | 3 |",
        ParserOptions::gfm(),
        HtmlRendererOptions::default(),
    );

    assert!(html.contains("<th align=\"left\">a</th>"));
    assert!(html.contains("<th align=\"center\">b</th>"));
    assert!(html.contains("<th align=\"right\">c</th>"));
    assert!(html.contains("<td align=\"left\">1</td>"));
    assert!(html.contains("<td align=\"center\">2</td>"));
    assert!(html.contains("<td align=\"right\">3</td>"));
}

#[test]
fn code_block_meta_does_not_leak_into_class_name() {
    let html = render(
        "```ts file=main.ts\nconsole.log(1)\n```",
        ParserOptions::default(),
        HtmlRendererOptions::default(),
    );

    assert!(html.contains("<pre><code class=\"language-ts\">"));
    assert!(!html.contains("file=main.ts"));
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

#[test]
fn hard_breaks_render_inside_paragraphs() {
    let html = render("line 1\\\nline 2", ParserOptions::default(), HtmlRendererOptions::default());
    assert_eq!(html, "<p>line 1<br>\nline 2</p>\n");
}

#[test]
fn html_type6_details_allows_markdown_after_blank_line() {
    let html = render(
        "<details>\n\n<summary>Click to expand</summary>\n\n**bold should be markdown**\n\n- list\n\n```js\nconsole.log(\"code\");\n```\n\n</details>",
        ParserOptions::default(),
        HtmlRendererOptions::default(),
    );

    assert!(html.contains("<details>"));
    assert!(html.contains("<summary>Click to expand</summary>"));
    assert!(html.contains("<strong>bold should be markdown</strong>"));
    assert!(html.contains("<ul>"));
    assert!(html.contains("<pre><code class=\"language-js\">"));
}
