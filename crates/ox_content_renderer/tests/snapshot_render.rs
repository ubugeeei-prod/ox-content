//! Exact, full-document HTML snapshot tests for the renderer.
//!
//! Captures the entire HTML output via `insta::assert_snapshot!` rather than
//! checking for individual substrings. Any drift in whitespace, attribute
//! order, escaping, sanitization, or block wrapping shows up immediately in
//! the diff.

use ox_content_allocator::Allocator;
use ox_content_parser::{Parser, ParserOptions};
use ox_content_renderer::{HtmlRenderer, HtmlRendererOptions};

fn render(
    source: &str,
    parser_options: ParserOptions,
    renderer_options: HtmlRendererOptions,
) -> String {
    let allocator = Allocator::new();
    let doc = Parser::with_options(&allocator, source, parser_options)
        .parse()
        .expect("parser should not fail on snapshot fixtures");
    let mut renderer = HtmlRenderer::with_options(renderer_options);
    renderer.render(&doc)
}

fn check(
    name: &str,
    source: &str,
    parser_options: ParserOptions,
    renderer_options: HtmlRendererOptions,
) {
    let html = render(source, parser_options, renderer_options);
    insta::with_settings!({
        snapshot_path => "snapshots/renderer",
        prepend_module_to_snapshot => false,
        description => source.to_string(),
        omit_expression => true,
    }, {
        insta::assert_snapshot!(name, html);
    });
}

// --- Plain blocks ---

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

#[test]
fn html_inline_emphasis_strong_combined() {
    check(
        "inline_emphasis_strong_combined",
        "*it* **bold** ***both*** `code`\n",
        ParserOptions::default(),
        HtmlRendererOptions::default(),
    );
}

#[test]
fn html_inline_strikethrough_gfm() {
    check(
        "inline_strikethrough_gfm",
        "~~gone~~ kept\n",
        ParserOptions::gfm(),
        HtmlRendererOptions::default(),
    );
}

#[test]
fn html_inline_hard_break_backslash() {
    check(
        "inline_hard_break_backslash",
        "line 1\\\nline 2\n",
        ParserOptions::default(),
        HtmlRendererOptions::default(),
    );
}

#[test]
fn html_inline_escaped_punctuation() {
    check(
        "inline_escaped_punctuation",
        "\\*not italic\\*\n",
        ParserOptions::default(),
        HtmlRendererOptions::default(),
    );
}

#[test]
fn html_inline_special_chars_are_escaped() {
    check(
        "inline_special_chars_are_escaped",
        "5 < 6 & 7 > 4, quote: \"hi\" 'bye'\n",
        ParserOptions::default(),
        HtmlRendererOptions::default(),
    );
}

// --- Links and images ---

#[test]
fn html_external_link_adds_security_attrs() {
    check(
        "external_link_adds_security_attrs",
        "[site](https://example.com)\n",
        ParserOptions::default(),
        HtmlRendererOptions::default(),
    );
}

#[test]
fn html_relative_link_has_no_external_attrs() {
    check(
        "relative_link_has_no_external_attrs",
        "[guide](./guide.md)\n",
        ParserOptions::default(),
        HtmlRendererOptions::default(),
    );
}

#[test]
fn html_link_with_title() {
    check(
        "link_with_title",
        "[home](https://example.com \"Title\")\n",
        ParserOptions::default(),
        HtmlRendererOptions::default(),
    );
}

#[test]
fn html_image_basic() {
    check(
        "image_basic",
        "![alt](./logo.png)\n",
        ParserOptions::default(),
        HtmlRendererOptions::default(),
    );
}

#[test]
fn html_image_xhtml_self_closes() {
    check(
        "image_xhtml_self_closes",
        "![logo](/logo.svg)\n",
        ParserOptions::default(),
        HtmlRendererOptions { xhtml: true, ..HtmlRendererOptions::default() },
    );
}

// --- Sanitization ---

#[test]
fn html_sanitize_escapes_html_block() {
    check(
        "sanitize_escapes_html_block",
        "<div><script>alert(1)</script></div>\n",
        ParserOptions::default(),
        HtmlRendererOptions { sanitize: true, ..HtmlRendererOptions::default() },
    );
}

#[test]
fn html_sanitize_escapes_inline_raw_html() {
    check(
        "sanitize_escapes_inline_raw_html",
        "<span>ok</span>\n",
        ParserOptions::default(),
        HtmlRendererOptions { sanitize: true, ..HtmlRendererOptions::default() },
    );
}

#[test]
fn html_sanitize_neutralizes_javascript_link() {
    check(
        "sanitize_neutralizes_javascript_link",
        "[run](javascript:alert(1))\n",
        ParserOptions::default(),
        HtmlRendererOptions { sanitize: true, ..HtmlRendererOptions::default() },
    );
}

#[test]
fn html_sanitize_neutralizes_obfuscated_javascript_link() {
    check(
        "sanitize_neutralizes_obfuscated_javascript_link",
        "[run](  JaVa ScRiPt:alert(1))\n",
        ParserOptions::default(),
        HtmlRendererOptions { sanitize: true, ..HtmlRendererOptions::default() },
    );
}

#[test]
fn html_sanitize_clears_unsafe_image_data_url() {
    check(
        "sanitize_clears_unsafe_image_data_url",
        "![x](data:text/html,<script>alert(1)</script>)\n",
        ParserOptions::default(),
        HtmlRendererOptions { sanitize: true, ..HtmlRendererOptions::default() },
    );
}

#[test]
fn html_sanitize_keeps_allowed_schemes() {
    check(
        "sanitize_keeps_allowed_schemes",
        "[guide](./guide.md) [mail](mailto:hi@example.com) [phone](tel:+123)\n",
        ParserOptions::default(),
        HtmlRendererOptions { sanitize: true, ..HtmlRendererOptions::default() },
    );
}

// --- Base URL / md-link conversion ---

#[test]
fn html_base_url_prefixes_root_absolute_links() {
    check(
        "base_url_prefixes_root_absolute_links",
        "[guide](/guide) [dir](/guide/) [md](/api.md#types)\n",
        ParserOptions::default(),
        HtmlRendererOptions {
            convert_md_links: true,
            base_url: "/docs/".to_string(),
            ..HtmlRendererOptions::default()
        },
    );
}

#[test]
fn html_base_url_prefixes_root_absolute_images() {
    check(
        "base_url_prefixes_root_absolute_images",
        "![logo](/img/logo.png)\n",
        ParserOptions::default(),
        HtmlRendererOptions {
            convert_md_links: true,
            base_url: "/docs/".to_string(),
            ..HtmlRendererOptions::default()
        },
    );
}

#[test]
fn html_base_url_prefixes_raw_html_attrs() {
    check(
        "base_url_prefixes_raw_html_attrs",
        "<div>\n<a href=\"/guide\">Guide</a>\n<img src='/img/logo.png'>\n</div>\n",
        ParserOptions::default(),
        HtmlRendererOptions {
            convert_md_links: true,
            base_url: "/docs/".to_string(),
            ..HtmlRendererOptions::default()
        },
    );
}

#[test]
fn html_base_url_leaves_protocol_relative_unchanged() {
    check(
        "base_url_leaves_protocol_relative_unchanged",
        "<script src=\"//cdn.example/app.js\"></script>\n",
        ParserOptions::default(),
        HtmlRendererOptions {
            convert_md_links: true,
            base_url: "/docs/".to_string(),
            ..HtmlRendererOptions::default()
        },
    );
}

// --- Tables (GFM) ---

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

#[test]
fn html_kitchen_sink_document() {
    check(
        "kitchen_sink_document",
        concat!(
            "# Project\n",
            "\n",
            "An intro with **bold**, *it*, `code`, ~~old~~, [link](https://example.com), and a hard\\\nbreak.\n",
            "\n",
            "## Install\n",
            "\n",
            "```bash\n",
            "npm install ox-content\n",
            "```\n",
            "\n",
            "## Steps\n",
            "\n",
            "1. one\n",
            "2. two\n",
            "   - nested\n",
            "3. three\n",
            "\n",
            "> Quoted tip.\n",
            "\n",
            "| col | val |\n",
            "| :-- | --: |\n",
            "| a   | 1   |\n",
            "| b   | 2   |\n",
            "\n",
            "![diagram](./img.png \"Title\")\n",
        ),
        ParserOptions::gfm(),
        HtmlRendererOptions::default(),
    );
}
