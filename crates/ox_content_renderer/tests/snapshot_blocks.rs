#[path = "support/snapshot.rs"]
mod snapshot_support;

use ox_content_parser::ParserOptions;
use ox_content_renderer::HtmlRendererOptions;
use snapshot_support::check;

#[test]
fn html_empty_document() {
    check("empty_document", "", ParserOptions::default(), HtmlRendererOptions::default());
}

#[test]
fn html_single_paragraph() {
    check(
        "single_paragraph",
        "Just a paragraph.\n",
        ParserOptions::default(),
        HtmlRendererOptions::default(),
    );
}

#[test]
fn html_atx_headings_all_depths() {
    check(
        "atx_headings_all_depths",
        "# h1\n## h2\n### h3\n#### h4\n##### h5\n###### h6\n",
        ParserOptions::default(),
        HtmlRendererOptions::default(),
    );
}

#[test]
fn html_thematic_break() {
    check(
        "thematic_break_variants",
        "---\n\n***\n\n___\n",
        ParserOptions::default(),
        HtmlRendererOptions::default(),
    );
}

#[test]
fn html_blockquote_multi_paragraph() {
    check(
        "blockquote_multi_paragraph",
        "> first\n>\n> second\n",
        ParserOptions::default(),
        HtmlRendererOptions::default(),
    );
}

#[test]
fn html_blockquote_nested() {
    check(
        "blockquote_nested",
        "> outer\n>\n> > inner\n",
        ParserOptions::default(),
        HtmlRendererOptions::default(),
    );
}

// --- Lists ---

#[test]
fn html_unordered_list_basic() {
    check(
        "unordered_list_basic",
        "- a\n- b\n- c\n",
        ParserOptions::default(),
        HtmlRendererOptions::default(),
    );
}

#[test]
fn html_wrapped_list_item_continuation() {
    check(
        "wrapped_list_item_continuation",
        "- [Blacksmith](https://www.blacksmith.sh/) for sponsoring CI and\n  Testbox infrastructure across projects.\n- [Mates Inc.](https://eng.mates.education/) for supporting OSS and\n  adopting Vize in production.\n",
        ParserOptions::default(),
        HtmlRendererOptions::default(),
    );
}

#[test]
fn html_ordered_list_with_start() {
    check(
        "ordered_list_with_start",
        "3. third\n4. fourth\n",
        ParserOptions::default(),
        HtmlRendererOptions::default(),
    );
}

#[test]
fn html_nested_lists_mixed_markers() {
    check(
        "nested_lists_mixed_markers",
        "- parent\n  - child a\n  - child b\n    1. deep one\n- sibling\n",
        ParserOptions::default(),
        HtmlRendererOptions::default(),
    );
}

#[test]
fn html_task_list_gfm() {
    check(
        "task_list_gfm",
        "- [x] done\n- [ ] todo\n",
        ParserOptions::gfm(),
        HtmlRendererOptions::default(),
    );
}

#[test]
fn html_task_list_without_gfm_renders_literal() {
    check(
        "task_list_without_gfm_renders_literal",
        "- [x] done\n",
        ParserOptions::default(),
        HtmlRendererOptions::default(),
    );
}

#[test]
fn html_fenced_code_with_lang() {
    check(
        "fenced_code_with_lang",
        "```ts\nconst x = 1;\n```\n",
        ParserOptions::default(),
        HtmlRendererOptions::default(),
    );
}

#[test]
fn html_fenced_code_with_meta_strips_meta_from_class() {
    check(
        "fenced_code_with_meta_strips_meta_from_class",
        "```ts file=main.ts\nconsole.log(1)\n```\n",
        ParserOptions::default(),
        HtmlRendererOptions::default(),
    );
}

#[test]
fn html_fenced_code_inside_list_item() {
    check(
        "fenced_code_inside_list_item",
        "1. text\n\n   ```ts\n   const a = 1;\n   ```\n",
        ParserOptions::default(),
        HtmlRendererOptions::default(),
    );
}

// --- Inline ---
