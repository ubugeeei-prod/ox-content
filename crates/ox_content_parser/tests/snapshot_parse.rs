//! Exact, full-document snapshot tests for the parser.
//!
//! Unlike `edge_cases.rs`, these tests do not poke at individual fields with
//! `assert!`/`contains` — every case captures the entire AST as a
//! deterministic, indented tree string and pins it with `insta::assert_snapshot!`.
//! Spans are included so any structural drift is visible in the diff.

use ox_content_allocator::Allocator;
use ox_content_ast::Node;
use ox_content_parser::{Parser, ParserOptions};

mod pretty;

fn parse_to_snapshot(source: &str, options: ParserOptions) -> String {
    let allocator = Allocator::new();
    let doc = Parser::with_options(&allocator, source, options)
        .parse()
        .expect("parser should not fail on snapshot fixtures");
    let mut out = String::new();
    pretty::format_document(&doc, source, &mut out);
    out
}

fn check(name: &str, source: &str, options: ParserOptions) {
    let snapshot = parse_to_snapshot(source, options);
    insta::with_settings!({
        snapshot_path => "snapshots/parser",
        prepend_module_to_snapshot => false,
        description => source.to_string(),
        omit_expression => true,
    }, {
        insta::assert_snapshot!(name, snapshot);
    });
}

// --- Whitespace, blank input, trivial documents ---

#[test]
fn snapshot_empty_document() {
    check("empty_document", "", ParserOptions::default());
}

#[test]
fn snapshot_whitespace_only_document() {
    check("whitespace_only_document", "\n  \n\t\n   \n", ParserOptions::default());
}

#[test]
fn snapshot_single_paragraph() {
    check("single_paragraph", "Just one paragraph.", ParserOptions::default());
}

// --- Headings ---

#[test]
fn snapshot_atx_headings_all_depths() {
    check(
        "atx_headings_all_depths",
        "# h1\n## h2\n### h3\n#### h4\n##### h5\n###### h6\n",
        ParserOptions::default(),
    );
}

#[test]
fn snapshot_atx_heading_with_trailing_hashes() {
    check(
        "atx_heading_with_trailing_hashes",
        "## Title ###\n## Edge  ####  \n",
        ParserOptions::default(),
    );
}

#[test]
fn snapshot_atx_heading_requires_space() {
    check("atx_heading_requires_space", "#NoSpace\n## With space\n", ParserOptions::default());
}

#[test]
fn snapshot_atx_heading_too_many_hashes() {
    check("atx_heading_too_many_hashes", "####### too many\n", ParserOptions::default());
}

// --- Thematic breaks ---

#[test]
fn snapshot_thematic_break_variants() {
    check(
        "thematic_break_variants",
        "---\n\n___\n\n***\n\n  ---  \n\n - - -\n\n* * *\n",
        ParserOptions::default(),
    );
}

#[test]
fn snapshot_thematic_break_invalid_mixed_markers() {
    check("thematic_break_invalid_mixed_markers", "- * -\n", ParserOptions::default());
}

// --- Code blocks ---

#[test]
fn snapshot_fenced_code_with_lang_and_meta() {
    check(
        "fenced_code_with_lang_and_meta",
        "```ts filename=main.ts\nconst answer = 42;\n```\n",
        ParserOptions::default(),
    );
}

#[test]
fn snapshot_fenced_code_tildes_with_meta() {
    check(
        "fenced_code_tildes_with_meta",
        "~~~rust title=lib.rs\nfn main() {}\n~~~\n",
        ParserOptions::default(),
    );
}

#[test]
fn snapshot_fenced_code_unclosed_until_eof() {
    check("fenced_code_unclosed_until_eof", "```rs\nfn main() {}\n", ParserOptions::default());
}

#[test]
fn snapshot_fenced_code_indented_inside_list_item() {
    check(
        "fenced_code_indented_inside_list_item",
        "1. text\n\n   ```ts\n   const a = 1;\n   ```\n",
        ParserOptions::default(),
    );
}

#[test]
fn snapshot_indented_code_block_preserves_blank_lines() {
    check(
        "indented_code_block_preserves_blank_lines",
        "    line a\n\n    line b\n    line c\n",
        ParserOptions::default(),
    );
}

// --- Block quotes ---

#[test]
fn snapshot_blockquote_single_line() {
    check("blockquote_single_line", "> hello\n", ParserOptions::default());
}

#[test]
fn snapshot_blockquote_multi_paragraph() {
    check("blockquote_multi_paragraph", "> first\n>\n> second\n", ParserOptions::default());
}

#[test]
fn snapshot_blockquote_nested() {
    check(
        "blockquote_nested",
        "> outer\n>\n> > inner\n>\n> outer again\n",
        ParserOptions::default(),
    );
}

#[test]
fn snapshot_blockquote_with_nested_list_and_code() {
    check(
        "blockquote_with_nested_list_and_code",
        "> - item one\n> - item two\n>\n> ```rs\n> fn f() {}\n> ```\n",
        ParserOptions::default(),
    );
}

// --- Lists ---

#[test]
fn snapshot_unordered_list_dash_marker() {
    check("unordered_list_dash_marker", "- a\n- b\n- c\n", ParserOptions::default());
}

#[test]
fn snapshot_wrapped_list_item_continuation() {
    check(
        "wrapped_list_item_continuation",
        "- [Blacksmith](https://www.blacksmith.sh/) for sponsoring CI and\n  Testbox infrastructure across projects.\n- [Mates Inc.](https://eng.mates.education/) for supporting OSS and\n  adopting Vize in production.\n",
        ParserOptions::default(),
    );
}

#[test]
fn snapshot_ordered_list_with_start() {
    check("ordered_list_with_start", "3. third\n4. fourth\n", ParserOptions::default());
}

#[test]
fn snapshot_ordered_list_parenthesis_marker() {
    check("ordered_list_parenthesis_marker", "3) third\n4) fourth\n", ParserOptions::default());
}

#[test]
fn snapshot_nested_lists_mixed_markers() {
    check(
        "nested_lists_mixed_markers",
        "- parent\n  - child\n  - second child\n    1. deep one\n    2. deep two\n- sibling\n",
        ParserOptions::default(),
    );
}

#[test]
fn snapshot_task_list_literal_without_gfm() {
    check("task_list_literal_without_gfm", "- [x] done\n- [ ] todo\n", ParserOptions::default());
}

#[test]
fn snapshot_task_list_with_gfm() {
    check("task_list_with_gfm", "- [ ] todo\n- [x] done\n", ParserOptions::gfm());
}

#[test]
fn snapshot_loose_vs_tight_list() {
    check(
        "loose_vs_tight_list",
        "- tight a\n- tight b\n\n- loose a\n\n- loose b\n",
        ParserOptions::default(),
    );
}

// --- Inline content ---

#[test]
fn snapshot_inline_emphasis_strong_combined() {
    check(
        "inline_emphasis_strong_combined",
        "An *italic* word and a **bold** one and a ***both*** one.\n",
        ParserOptions::default(),
    );
}

#[test]
fn snapshot_inline_underscore_emphasis() {
    check(
        "inline_underscore_emphasis",
        "_italic_ and __bold__ and ___both___.\n",
        ParserOptions::default(),
    );
}

#[test]
fn snapshot_inline_strikethrough_gfm() {
    check("inline_strikethrough_gfm", "~~gone~~ but kept.\n", ParserOptions::gfm());
}

#[test]
fn snapshot_inline_strikethrough_unmatched() {
    check("inline_strikethrough_unmatched", "~~open\n", ParserOptions::gfm());
}

#[test]
fn snapshot_inline_code_basic() {
    check("inline_code_basic", "Use `let x = 1;` to declare a value.\n", ParserOptions::default());
}

#[test]
fn snapshot_inline_code_with_html_literal() {
    check(
        "inline_code_with_html_literal",
        "Show `<input type=\"checkbox\">` literally.\n",
        ParserOptions::default(),
    );
}

#[test]
fn snapshot_inline_hard_break_backslash() {
    check("inline_hard_break_backslash", "line 1\\\nline 2\n", ParserOptions::default());
}

#[test]
fn snapshot_inline_escaped_punctuation() {
    check(
        "inline_escaped_punctuation",
        "\\*not italic\\* and \\_not underscore\\_\n",
        ParserOptions::default(),
    );
}

// --- Links and images ---

#[test]
fn snapshot_inline_link_simple() {
    check("inline_link_simple", "See [the site](https://example.com).\n", ParserOptions::default());
}

#[test]
fn snapshot_inline_link_with_title() {
    check(
        "inline_link_with_title",
        "See [home](https://example.com \"Example Home\").\n",
        ParserOptions::default(),
    );
}

#[test]
fn snapshot_inline_link_nested_parentheses() {
    check(
        "inline_link_nested_parentheses",
        "[docs](https://example.com/a(b)c)\n",
        ParserOptions::default(),
    );
}

#[test]
fn snapshot_image_simple() {
    check("image_simple", "![alt](./logo.png)\n", ParserOptions::default());
}

#[test]
fn snapshot_image_with_title() {
    check("image_with_title", "![alt](./logo.png \"A logo\")\n", ParserOptions::default());
}

#[test]
fn snapshot_image_nested_parentheses() {
    check("image_nested_parentheses", "![diagram](./img(test).png)\n", ParserOptions::default());
}

// --- Raw HTML ---

#[test]
fn snapshot_html_block_div() {
    check("html_block_div", "<div>\nraw html line\n</div>\n\nAfter\n", ParserOptions::default());
}

#[test]
fn snapshot_html_block_details() {
    check(
        "html_block_details",
        "<details id=\"a\">\n<summary>S</summary>\n<p>Body</p>\n</details>\n\nAfter\n",
        ParserOptions::default(),
    );
}

#[test]
fn snapshot_html_type6_details_resumes_markdown_after_blank() {
    check(
        "html_type6_details_resumes_markdown_after_blank",
        "<details>\n\n<summary>Click</summary>\n\n**bold**\n\n- list\n\n</details>\n",
        ParserOptions::default(),
    );
}

#[test]
fn snapshot_inline_raw_html_in_paragraph() {
    check(
        "inline_raw_html_in_paragraph",
        "before <span class=\"x\">middle</span> after\n",
        ParserOptions::default(),
    );
}

// --- Tables (GFM) ---

#[test]
fn snapshot_table_alignment_variants() {
    check(
        "table_alignment_variants",
        "| a | b | c |\n| :-- | :-: | --: |\n| 1 | 2 | 3 |\n",
        ParserOptions::gfm(),
    );
}

#[test]
fn snapshot_table_with_inline_formatting() {
    check(
        "table_with_inline_formatting",
        "| name | status |\n| ---- | ------ |\n| **bold** | *italic* |\n| `code` | ~~old~~ |\n",
        ParserOptions::gfm(),
    );
}

// --- Definitions / footnotes ---

#[test]
fn snapshot_reference_link_definition() {
    check(
        "reference_link_definition",
        "[link][ref]\n\n[ref]: https://example.com \"Title\"\n",
        ParserOptions::default(),
    );
}

#[test]
fn snapshot_footnote_reference_and_definition() {
    let mut options = ParserOptions::gfm();
    options.footnotes = true;
    check("footnote_reference_and_definition", "See[^1].\n\n[^1]: The footnote body.\n", options);
}

// --- Mixed / regression ---

#[test]
fn snapshot_kitchen_sink_document() {
    check(
        "kitchen_sink_document",
        concat!(
            "# Project README\n",
            "\n",
            "A short blurb with **bold**, *italic*, and `code`.\n",
            "\n",
            "## Install\n",
            "\n",
            "```bash\n",
            "npm install ox-content\n",
            "```\n",
            "\n",
            "## Usage\n",
            "\n",
            "- Step one\n",
            "- Step two\n",
            "  - Sub-step a\n",
            "  - Sub-step b\n",
            "\n",
            "> Tip: read the docs.\n",
            "\n",
            "| col | val |\n",
            "| --- | --- |\n",
            "| a   | 1   |\n",
            "| b   | 2   |\n",
            "\n",
            "See the [website](https://example.com).\n",
        ),
        ParserOptions::gfm(),
    );
}

// --- Sanity guard: pretty-printer never emits an empty document for input that contains
//     at least one block-level construct. This is a structural invariant we want to defend
//     against silent regressions where the parser swallows valid content. ---

#[test]
fn snapshot_first_block_node_kinds_for_basic_constructs() {
    let cases: &[(&str, &str)] = &[
        ("para_kind", "hello"),
        ("heading_kind", "# h"),
        ("list_kind", "- a"),
        ("blockquote_kind", "> q"),
        ("code_kind", "```\nfn\n```"),
        ("hr_kind", "---"),
        ("table_kind", "| a |\n| - |\n| 1 |"),
    ];

    let mut report = String::new();
    let allocator = Allocator::new();
    for (label, source) in cases {
        let doc = Parser::with_options(&allocator, source, ParserOptions::gfm())
            .parse()
            .expect("parser should not fail");
        report.push_str(label);
        report.push_str(": ");
        report.push_str(node_kind(doc.children.first()));
        report.push('\n');
    }

    insta::with_settings!({
        snapshot_path => "snapshots/parser",
        prepend_module_to_snapshot => false,
        omit_expression => true,
    }, {
        insta::assert_snapshot!("first_block_node_kinds_for_basic_constructs", report);
    });
}

fn node_kind(node: Option<&Node<'_>>) -> &'static str {
    match node {
        None => "<none>",
        Some(Node::Paragraph(_)) => "Paragraph",
        Some(Node::Heading(_)) => "Heading",
        Some(Node::ThematicBreak(_)) => "ThematicBreak",
        Some(Node::BlockQuote(_)) => "BlockQuote",
        Some(Node::List(_)) => "List",
        Some(Node::ListItem(_)) => "ListItem",
        Some(Node::CodeBlock(_)) => "CodeBlock",
        Some(Node::Html(_)) => "Html",
        Some(Node::Table(_)) => "Table",
        Some(Node::Text(_)) => "Text",
        Some(Node::Emphasis(_)) => "Emphasis",
        Some(Node::Strong(_)) => "Strong",
        Some(Node::InlineCode(_)) => "InlineCode",
        Some(Node::Break(_)) => "Break",
        Some(Node::Link(_)) => "Link",
        Some(Node::Image(_)) => "Image",
        Some(Node::Delete(_)) => "Delete",
        Some(Node::FootnoteReference(_)) => "FootnoteReference",
        Some(Node::Definition(_)) => "Definition",
        Some(Node::FootnoteDefinition(_)) => "FootnoteDefinition",
    }
}
