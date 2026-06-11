#[path = "support/snapshot.rs"]
mod snapshot_support;

use ox_content_parser::ParserOptions;
use ox_content_renderer::HtmlRendererOptions;
use snapshot_support::check;

#[test]
fn html_table_alignment_variants() {
    check(
        "table_alignment_variants",
        "| a | b | c |\n| :-- | :-: | --: |\n| 1 | 2 | 3 |\n",
        ParserOptions::gfm(),
        HtmlRendererOptions::default(),
    );
}

#[test]
fn html_table_with_inline_formatting() {
    check(
        "table_with_inline_formatting",
        "| name | status |\n| ---- | ------ |\n| **bold** | *it* |\n| `code` | ~~old~~ |\n",
        ParserOptions::gfm(),
        HtmlRendererOptions::default(),
    );
}

// --- HTML block behaviors ---

#[test]
fn html_block_details_preserved_raw() {
    check(
        "html_block_details_preserved_raw",
        "<details id=\"a\">\n<summary>S</summary>\n<p>Body</p>\n</details>\n",
        ParserOptions::default(),
        HtmlRendererOptions::default(),
    );
}

#[test]
fn html_type6_details_resumes_markdown_after_blank() {
    check(
        "html_type6_details_resumes_markdown_after_blank",
        "<details>\n\n<summary>Click</summary>\n\n**bold**\n\n- list\n\n```js\nconsole.log(\"x\");\n```\n\n</details>\n",
        ParserOptions::default(),
        HtmlRendererOptions::default(),
    );
}

#[test]
fn html_inline_raw_html_in_list_item() {
    check(
        "inline_raw_html_in_list_item",
        "- <input type=\"checkbox\"> task\n",
        ParserOptions::default(),
        HtmlRendererOptions::default(),
    );
}

// --- Kitchen sink: end-to-end realistic doc ---
