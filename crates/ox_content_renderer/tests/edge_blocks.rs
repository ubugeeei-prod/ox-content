#[path = "support/edge.rs"]
mod edge_support;

use edge_support::render;
use ox_content_parser::ParserOptions;
use ox_content_renderer::HtmlRendererOptions;
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
fn fenced_code_inside_list_item_renders_as_block_code() {
    let html = render(
        "1. text\n\n   ```ts\n   const a = 1;\n   ```",
        ParserOptions::default(),
        HtmlRendererOptions::default(),
    );

    assert!(html.contains("<ol>"));
    assert!(html.contains("<li><p>text</p>"));
    assert!(html.contains("<pre><code class=\"language-ts\">const a = 1;\n</code></pre>"));
    assert!(!html.contains("<p><code></code><code>ts"));
}

#[test]
fn hard_breaks_render_inside_paragraphs() {
    let html = render("line 1\\\nline 2", ParserOptions::default(), HtmlRendererOptions::default());
    assert_eq!(html, "<p>line 1<br>\nline 2</p>\n");
}

#[test]
fn inline_raw_html_renders_without_extra_newline() {
    let html = render(
        "- <input type=\"checkbox\"> task",
        ParserOptions::default(),
        HtmlRendererOptions::default(),
    );

    assert_eq!(html, "<ul>\n<li><p><input type=\"checkbox\"> task</p>\n</li>\n</ul>\n");
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
