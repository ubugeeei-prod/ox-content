#[path = "support/edge.rs"]
mod edge_support;

use edge_support::render;
use ox_content_parser::ParserOptions;
use ox_content_renderer::HtmlRendererOptions;
#[test]
fn ordered_lists_preserve_start_attribute() {
    let html =
        render("3. third\n4. fourth", ParserOptions::default(), HtmlRendererOptions::default());
    insta::assert_snapshot!(html);
}

#[test]
fn task_list_without_feature_renders_literal_text() {
    let html = render("- [x] done", ParserOptions::default(), HtmlRendererOptions::default());
    insta::assert_snapshot!(html);
}

#[test]
fn aligned_tables_render_align_attributes() {
    let html = render(
        "| a | b | c |\n| :-- | :-: | --: |\n| 1 | 2 | 3 |",
        ParserOptions::gfm(),
        HtmlRendererOptions::default(),
    );

    insta::assert_snapshot!(html);
}

#[test]
fn code_block_meta_does_not_leak_into_class_name() {
    let html = render(
        "```ts file=main.ts\nconsole.log(1)\n```",
        ParserOptions::default(),
        HtmlRendererOptions::default(),
    );

    insta::assert_snapshot!(html);
}

#[test]
fn fenced_code_inside_list_item_renders_as_block_code() {
    let html = render(
        "1. text\n\n   ```ts\n   const a = 1;\n   ```",
        ParserOptions::default(),
        HtmlRendererOptions::default(),
    );

    insta::assert_snapshot!(html);
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

    insta::assert_snapshot!(html);
}
