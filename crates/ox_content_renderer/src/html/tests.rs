use super::*;
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
    assert!(html.contains("<h2 id=\"はじめに\">はじめに</h2>"));
    assert!(html.contains("<h2 id=\"はじめに-1\">はじめに</h2>"));
}

#[test]
fn test_render_heading_id_uses_inline_text() {
    let allocator = Allocator::new();
    let doc = Parser::new(&allocator, "## **API** `Index` [Guide](./guide.md)").parse().unwrap();
    let mut renderer = HtmlRenderer::new();
    let html = renderer.render(&doc);
    assert!(html.starts_with("<h2 id=\"api-index-guide\">"));
}

#[test]
fn test_render_inline_toc_directive() {
    let allocator = Allocator::new();
    let doc = Parser::new(&allocator, "# Title\n\n[[toc]]\n\n## Intro\n### API").parse().unwrap();
    let mut renderer = HtmlRenderer::new();
    let html = renderer.render(&doc);

    assert!(html.contains("<nav class=\"ox-toc\" aria-label=\"Table of contents\">"));
    assert!(html.contains("<a href=\"#title\">Title</a>"));
    assert!(html.contains("<a href=\"#intro\">Intro</a>"));
    assert!(html.contains("<a href=\"#api\">API</a>"));
    assert!(!html.contains("<p>[[toc]]</p>"));
}

#[test]
fn test_render_inline_toc_uses_unique_and_unicode_ids() {
    let allocator = Allocator::new();
    let doc =
        Parser::new(&allocator, "[[toc]]\n\n## Setup\n## Setup\n## はじめに").parse().unwrap();
    let mut renderer = HtmlRenderer::new();
    let html = renderer.render(&doc);

    assert!(html.contains("href=\"#setup\""));
    assert!(html.contains("href=\"#setup-1\""));
    assert!(html.contains("href=\"#はじめに\""));
}

#[test]
fn test_render_inline_toc_requires_standalone_text() {
    let allocator = Allocator::new();
    let doc = Parser::new(&allocator, "See [[toc]] here\n\n`[[toc]]`\n\n## Intro").parse().unwrap();
    let mut renderer = HtmlRenderer::new();
    let html = renderer.render(&doc);

    assert!(html.contains("<p>See [[toc]] here</p>"));
    assert!(html.contains("<p><code>[[toc]]</code></p>"));
    assert!(!html.contains("ox-toc"));
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

    assert!(!html.contains("[[toc]]"), "marker leaked into output: {html}");
    assert!(!html.contains("<p>"), "expected no paragraph wrapper: {html}");
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

    assert!(!html.contains("[[toc]]"), "marker leaked: {html}");
    // The heading should still render as a heading (not as a TOC entry).
    assert!(html.contains("<h2"), "heading missing: {html}");
}

#[test]
fn test_render_inline_toc_honors_max_depth() {
    let allocator = Allocator::new();
    let doc = Parser::new(&allocator, "[[toc]]\n\n# Title\n## Intro\n### API").parse().unwrap();
    let mut renderer =
        HtmlRenderer::with_options(HtmlRendererOptions { toc_max_depth: 2, ..Default::default() });
    let html = renderer.render(&doc);

    assert!(html.contains("href=\"#title\""));
    assert!(html.contains("href=\"#intro\""));
    assert!(!html.contains("href=\"#api\""));
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
    assert!(html.contains("<blockquote>"));
    assert!(html.contains("<strong>Note:</strong>"));
    assert!(html.contains("</blockquote>"));
}

#[test]
fn test_render_github_style_important_callout() {
    let allocator = Allocator::new();
    let doc = Parser::new(&allocator, "> [!IMPORTANT]\n> This is important.").parse().unwrap();
    let mut renderer = HtmlRenderer::new();
    let html = renderer.render(&doc);

    assert!(html.contains("<blockquote class=\"ox-callout ox-callout--important\">"));
    assert!(html.contains("<p class=\"ox-callout-title\">Important</p>"));
    assert!(html.contains("<p>This is important.</p>"));
    assert!(!html.contains("[!IMPORTANT]"));
}

#[test]
fn test_render_github_style_callout_with_inline_content_after_marker() {
    let allocator = Allocator::new();
    let doc = Parser::new(&allocator, "> [!NOTE] Supports **inline** content").parse().unwrap();
    let mut renderer = HtmlRenderer::new();
    let html = renderer.render(&doc);

    assert!(html.contains("<blockquote class=\"ox-callout ox-callout--note\">"));
    assert!(html.contains("<p class=\"ox-callout-title\">Note</p>"));
    assert!(html.contains("<p>Supports <strong>inline</strong> content</p>"));
    assert!(!html.contains("[!NOTE]"));
}

#[test]
fn test_render_code_block() {
    let allocator = Allocator::new();
    let doc = Parser::new(&allocator, "```rust\nfn main() {}\n```").parse().unwrap();
    let mut renderer = HtmlRenderer::new();
    let html = renderer.render(&doc);
    assert!(html.contains("<pre><code class=\"language-rust\">"));
}

#[test]
fn test_render_code_block_with_annotations() {
    let allocator = Allocator::new();
    let doc = Parser::new(
            &allocator,
            "```ts file=main.ts annotate=\"highlight:1;warning:2;error:3\"\nconst ok = true;\nconst maybe = false;\nthrow new Error('boom');\n```",
        )
        .parse()
        .unwrap();
    let mut renderer = HtmlRenderer::with_options(HtmlRendererOptions {
        code_annotations: true,
        ..Default::default()
    });
    let html = renderer.render(&doc);

    assert!(html.contains("class=\"ox-code-block ox-code-block--annotated has-highlighted\""));
    assert!(html.contains(
        "class=\"line ox-code-line ox-code-line--highlight highlighted\" data-line=\"1\""
    ));
    assert!(html.contains(
        "class=\"line ox-code-line ox-code-line--warning highlighted warning\" data-line=\"2\""
    ));
    assert!(html.contains(
        "class=\"line ox-code-line ox-code-line--error highlighted error\" data-line=\"3\""
    ));
    assert!(!html.contains("file=main.ts"));
}

#[test]
fn test_render_code_block_with_custom_annotation_meta_key() {
    let allocator = Allocator::new();
    let doc = Parser::new(
        &allocator,
        "```ts markers=\"warning:2\"\nconst ok = true;\nconst maybe = false;\n```",
    )
    .parse()
    .unwrap();
    let mut renderer = HtmlRenderer::with_options(HtmlRendererOptions {
        code_annotations: true,
        code_annotation_meta_key: "markers".to_string(),
        ..Default::default()
    });
    let html = renderer.render(&doc);

    assert!(html.contains("ox-code-block--annotated"));
    assert!(html.contains("ox-code-line--warning"));
}

#[test]
fn test_render_code_block_with_vitepress_meta() {
    let allocator = Allocator::new();
    let doc = Parser::new(
            &allocator,
            "```ts:line-numbers=2 {1,3} [config.ts]\nconst first = true;\nconst second = false;\nconst third = true;\n```",
        )
        .parse()
        .unwrap();
    let mut renderer = HtmlRenderer::with_options(HtmlRendererOptions {
        code_annotations: true,
        code_annotation_syntax: CodeAnnotationSyntax::VitePress,
        ..Default::default()
    });
    let html = renderer.render(&doc);

    assert!(html.contains("ox-code-block--annotated"));
    assert!(html.contains("ox-code-block--line-numbers"));
    assert!(html.contains("ox-code-block--with-title"));
    assert!(html.contains("line-numbers-mode"));
    assert!(html.contains("has-highlighted"));
    assert!(html.contains("data-code-title=\"config.ts\""));
    assert!(html.contains("data-line-number-start=\"2\""));
    assert!(html.contains("class=\"language-ts\""));
    assert!(html.contains("data-line-number=\"2\""));
    assert!(html.contains("data-line-number=\"4\""));
    assert!(html.contains("ox-code-line--highlight"));
}

#[test]
fn test_render_code_block_with_vitepress_inline_directives() {
    let allocator = Allocator::new();
    let doc = Parser::new(
            &allocator,
            "```ts\n// [!code focus:2]\nconst first = true;\nconst second = false;\nconsole.log('old value') // [!code --]\nconsole.log('new value') // [!code ++]\nconsole.warn('careful') // [!code warning]\nthrow new Error('boom') // [!code error]\n```",
        )
        .parse()
        .unwrap();
    let mut renderer = HtmlRenderer::with_options(HtmlRendererOptions {
        code_annotations: true,
        code_annotation_syntax: CodeAnnotationSyntax::VitePress,
        ..Default::default()
    });
    let html = renderer.render(&doc);

    assert!(!html.contains("[!code"));
    assert!(html.contains("has-focused"));
    assert!(html.contains("has-diff"));
    assert!(html.contains("ox-code-line--focus"));
    assert!(html.contains("ox-code-line--dimmed"));
    assert!(html.contains("ox-code-line--remove"));
    assert!(html.contains("ox-code-line--add"));
    assert!(html.contains("ox-code-line--warning"));
    assert!(html.contains("ox-code-line--error"));
    assert!(html.contains("console.log(&#39;old value&#39;)"));
    assert!(html.contains("console.log(&#39;new value&#39;)"));
}

#[test]
fn test_render_code_block_with_vitepress_escape_next_line() {
    let allocator = Allocator::new();
    let doc = Parser::new(
            &allocator,
            "```ts\n// [!code escape]\nconsole.warn('literal') // [!code warning]\nconsole.warn('annotated') // [!code warning]\n```",
        )
        .parse()
        .unwrap();
    let mut renderer = HtmlRenderer::with_options(HtmlRendererOptions {
        code_annotations: true,
        code_annotation_syntax: CodeAnnotationSyntax::VitePress,
        ..Default::default()
    });
    let html = renderer.render(&doc);

    assert!(!html.contains("[!code escape]"));
    assert!(html.contains("console.warn(&#39;literal&#39;) // [!code warning]"));
    assert!(html.contains("console.warn(&#39;annotated&#39;)"));
    assert_eq!(html.matches("ox-code-line--warning").count(), 1);
}

#[test]
fn test_render_nested_list() {
    let allocator = Allocator::new();
    // Indent with 2 spaces for nesting
    let doc = Parser::new(&allocator, "- item 1\n  - sub 1\n- item 2").parse().unwrap();
    let mut renderer = HtmlRenderer::new();
    let html = renderer.render(&doc);

    // Normalize newlines for comparison
    let normalized = html.replace('\n', "");
    // We expect:
    // <ul>
    //   <li>
    //     <p>item 1</p>
    //     <ul>
    //       <li><p>sub 1</p></li>
    //     </ul>
    //   </li>
    //   <li><p>item 2</p></li>
    // </ul>
    // Note: The exact placement of <p> tags depends on how we handle list content.
    // Assuming tight list items might not have <p> if we implement loose/tight lists,
    // but currently everything is wrapped in <p> in parse_list implementation (wrapped in Paragraph).

    // Let's just check for the structure <li>...<ul>...</ul>...</li>
    assert!(normalized.contains("<li><p>item 1</p><ul><li><p>sub 1</p></li></ul></li>"));
    assert!(normalized.contains("<li><p>item 2</p></li>"));
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
    assert!(html.contains("<table>"));
    assert!(html.contains("<thead>"));
    assert!(html.contains("<th>head</th>"));
    assert!(html.contains("<tbody>"));
    assert!(html.contains("<td>body</td>"));
}

#[test]
fn test_render_table_no_gfm() {
    let allocator = Allocator::new();
    // Default options have tables: false
    let doc = Parser::new(&allocator, "| head |\n| --- |\n| body |").parse().unwrap();
    let mut renderer = HtmlRenderer::new();
    let html = renderer.render(&doc);
    assert!(!html.contains("<table>"));
    assert!(html.contains("| head |"));
}

#[test]
fn test_render_heading_with_link() {
    let allocator = Allocator::new();
    let doc = Parser::new(&allocator, "### [index](./index-module.md)").parse().unwrap();
    let mut renderer = HtmlRenderer::new();
    let html = renderer.render(&doc);
    assert_eq!(html, "<h3 id=\"index\"><a href=\"./index-module.md\">index</a></h3>\n");
}

#[test]
fn test_render_list_with_bold() {
    let allocator = Allocator::new();
    let doc = Parser::new(&allocator, "- **bold** text").parse().unwrap();
    let mut renderer = HtmlRenderer::new();
    let html = renderer.render(&doc);
    assert!(html.contains("<strong>bold</strong>"));
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
    assert!(html.contains("<input type=\"checkbox\" checked disabled> <p>task 1</p>"));
    assert!(html.contains("<input type=\"checkbox\" disabled> <p>task 2</p>"));
}

#[test]
fn test_render_strikethrough() {
    let allocator = Allocator::new();
    let doc = Parser::with_options(&allocator, "~~done~~", ox_content_parser::ParserOptions::gfm())
        .parse()
        .unwrap();
    let mut renderer = HtmlRenderer::new();
    let html = renderer.render(&doc);
    assert_eq!(html, "<p><del>done</del></p>\n");
}

#[test]
fn test_render_hard_break() {
    let allocator = Allocator::new();
    let doc = Parser::new(&allocator, "line 1\\\nline 2").parse().unwrap();
    let mut renderer = HtmlRenderer::new();
    let html = renderer.render(&doc);
    assert_eq!(html, "<p>line 1<br>\nline 2</p>\n");
}

#[test]
fn test_render_image() {
    let allocator = Allocator::new();
    let doc = Parser::new(&allocator, "![Alt text](/path/to/image.png)").parse().unwrap();
    let mut renderer = HtmlRenderer::new();
    let html = renderer.render(&doc);
    assert!(html.contains("<img src=\"/path/to/image.png\" alt=\"Alt text\">"));
}

#[test]
fn test_render_image_xhtml() {
    let allocator = Allocator::new();
    let doc = Parser::new(&allocator, "![Logo](/logo.svg)").parse().unwrap();
    let mut renderer =
        HtmlRenderer::with_options(HtmlRendererOptions { xhtml: true, ..Default::default() });
    let html = renderer.render(&doc);
    assert!(html.contains("<img src=\"/logo.svg\" alt=\"Logo\" />"));
}

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

#[test]
fn test_autolink_disabled_by_default() {
    let allocator = Allocator::new();
    let doc = Parser::new(&allocator, "see http://example.com here").parse().unwrap();
    let mut renderer = HtmlRenderer::new();
    let html = renderer.render(&doc);
    // No <a> tag is emitted unless the flag is on.
    assert!(!html.contains("<a "), "unexpected autolink in: {html}");
    assert!(html.contains("http://example.com"));
}

#[test]
fn test_autolink_basic_http_and_https() {
    let allocator = Allocator::new();
    let doc =
        Parser::new(&allocator, "see http://example.com and https://example.org").parse().unwrap();
    let mut renderer = HtmlRenderer::with_options(HtmlRendererOptions {
        autolink_urls: true,
        ..Default::default()
    });
    let html = renderer.render(&doc);
    assert!(
            html.contains(
                "<a href=\"http://example.com\" target=\"_blank\" rel=\"noopener noreferrer\">http://example.com</a>"
            ),
            "missing http autolink in: {html}"
        );
    assert!(
            html.contains(
                "<a href=\"https://example.org\" target=\"_blank\" rel=\"noopener noreferrer\">https://example.org</a>"
            ),
            "missing https autolink in: {html}"
        );
}

#[test]
fn test_autolink_target_blank_can_be_disabled() {
    let allocator = Allocator::new();
    let doc = Parser::new(&allocator, "go to https://example.com now").parse().unwrap();
    let mut renderer = HtmlRenderer::with_options(HtmlRendererOptions {
        autolink_urls: true,
        autolink_target_blank: false,
        ..Default::default()
    });
    let html = renderer.render(&doc);
    assert!(
        html.contains("<a href=\"https://example.com\">https://example.com</a>"),
        "expected bare anchor in: {html}"
    );
    assert!(!html.contains("target=\"_blank\""), "blank attr leaked: {html}");
}

#[test]
fn test_autolink_strips_trailing_punctuation() {
    let allocator = Allocator::new();
    let doc =
        Parser::new(&allocator, "find it at https://example.com. or (https://example.org) maybe")
            .parse()
            .unwrap();
    let mut renderer = HtmlRenderer::with_options(HtmlRendererOptions {
        autolink_urls: true,
        ..Default::default()
    });
    let html = renderer.render(&doc);
    assert!(html.contains(">https://example.com</a>."), "period leaked: {html}");
    assert!(html.contains("(<a href=\"https://example.org\""), "open paren lost: {html}");
    assert!(html.contains(">https://example.org</a>)"), "close paren lost: {html}");
}

#[test]
fn test_autolink_word_boundary_required() {
    let allocator = Allocator::new();
    // "shttp://x" must not match — the prefix is glued to a word char.
    let doc = Parser::new(&allocator, "shttp://x and http://y").parse().unwrap();
    let mut renderer = HtmlRenderer::with_options(HtmlRendererOptions {
        autolink_urls: true,
        ..Default::default()
    });
    let html = renderer.render(&doc);
    assert!(!html.contains("href=\"http://x\""), "unexpected glued autolink: {html}");
    assert!(html.contains("href=\"http://y\""), "missing real autolink: {html}");
}

#[test]
fn test_autolink_custom_pattern_registration() {
    let allocator = Allocator::new();
    let doc = Parser::new(&allocator, "email mailto:foo@example.com please").parse().unwrap();
    let mut renderer = HtmlRenderer::with_options(HtmlRendererOptions {
        autolink_urls: true,
        autolink_patterns: vec!["mailto:".to_string()],
        ..Default::default()
    });
    let html = renderer.render(&doc);
    assert!(
        html.contains("<a href=\"mailto:foo@example.com\""),
        "missing custom-pattern autolink: {html}"
    );
}

#[test]
fn test_autolink_many_patterns_uses_table_fallback() {
    // Five patterns with five distinct leading letters exceed the
    // three-needle SIMD fast path, exercising the `FirstByteIndex`
    // lookup-table fallback. All schemes must still autolink.
    let allocator = Allocator::new();
    let doc = Parser::new(
        &allocator,
        "a http://h.test b ftp://f.test c mailto:m@x d tel:123 e ssh://s.test f",
    )
    .parse()
    .unwrap();
    let mut renderer = HtmlRenderer::with_options(HtmlRendererOptions {
        autolink_urls: true,
        autolink_patterns: vec![
            "http://".to_string(),
            "ftp://".to_string(),
            "mailto:".to_string(),
            "tel:".to_string(),
            "ssh://".to_string(),
        ],
        ..Default::default()
    });
    let html = renderer.render(&doc);
    for href in ["http://h.test", "ftp://f.test", "mailto:m@x", "tel:123", "ssh://s.test"] {
        let mut needle = String::with_capacity(href.len() + 9);
        needle.push_str("<a href=\"");
        needle.push_str(href);
        needle.push('"');
        assert!(html.contains(&needle), "missing {href} in: {html}");
    }
}

#[test]
fn test_autolink_does_not_nest_inside_existing_link() {
    let allocator = Allocator::new();
    // The text inside the explicit markdown link contains a URL — the
    // builtin must not wrap that URL in a second <a>.
    let doc = Parser::new(&allocator, "[visit https://example.com here](/page)").parse().unwrap();
    let mut renderer = HtmlRenderer::with_options(HtmlRendererOptions {
        autolink_urls: true,
        ..Default::default()
    });
    let html = renderer.render(&doc);
    assert_eq!(html.matches("<a ").count(), 1, "nested anchor in: {html}");
    assert!(html.contains("href=\"/page\""), "outer link lost: {html}");
    assert!(html.contains("visit https://example.com here"), "inner text lost: {html}");
}

#[test]
fn test_autolink_escapes_query_string_safely() {
    let allocator = Allocator::new();
    // `&` inside the URL must be escaped both as href and as visible
    // text — otherwise the output would be parser-ambiguous HTML.
    let doc = Parser::new(&allocator, "see http://a.test/?q=foo&r=bar now").parse().unwrap();
    let mut renderer = HtmlRenderer::with_options(HtmlRendererOptions {
        autolink_urls: true,
        ..Default::default()
    });
    let html = renderer.render(&doc);
    assert!(html.contains("href=\"http://a.test/?q=foo&amp;r=bar\""), "href not escaped: {html}");
    assert!(
        html.contains(">http://a.test/?q=foo&amp;r=bar</a>"),
        "visible text not escaped: {html}"
    );
}
