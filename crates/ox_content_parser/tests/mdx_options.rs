//! Strict tests for `ParserOptions.mdx`.
//!
//! Enabling the flag must not change CommonMark or GFM parse output for
//! non-JSX, non-ESM Markdown. PascalCase JSX is covered in `mdx_jsx.rs`.
//! Module-level `import` / `export` is covered in `mdx_esm.rs`.

use ox_content_allocator::Allocator;
use ox_content_parser::{Parser, ParserOptions};

#[path = "support/pretty.rs"]
mod pretty;

fn pretty_ast(source: &str, options: ParserOptions) -> String {
    let allocator = Allocator::new();
    let doc = Parser::with_options(&allocator, source, options)
        .parse()
        .expect("parser should not fail on MDX option fixtures");
    let mut out = String::new();
    pretty::format_document(&doc, source, &mut out);
    out
}

fn assert_mdx_flag_is_noop(source: &str, mut options: ParserOptions) {
    options.mdx = false;
    let off = pretty_ast(source, options.clone());
    options.mdx = true;
    let on = pretty_ast(source, options);
    assert_eq!(
        off, on,
        "ParserOptions.mdx must not change parse output for non-JSX Markdown\n--- mdx=false ---\n{off}\n--- mdx=true ---\n{on}"
    );
}

#[test]
fn default_options_disable_mdx() {
    let options = ParserOptions::default();
    assert!(!options.mdx, "MDX must stay opt-in until the default-enable PR");
    assert!(!options.gfm);
    assert!(!options.footnotes);
    assert!(!options.task_lists);
    assert!(!options.tables);
    assert!(!options.strikethrough);
    assert!(!options.autolinks);
}

#[test]
fn gfm_helper_does_not_enable_mdx() {
    let options = ParserOptions::gfm();
    assert!(!options.mdx);
    assert!(options.gfm);
    assert!(options.footnotes);
    assert!(options.task_lists);
    assert!(options.tables);
    assert!(options.strikethrough);
    assert!(options.autolinks);
    assert_eq!(options.max_nesting_depth, 100);
}

#[test]
fn mdx_helper_enables_mdx_without_gfm() {
    let options = ParserOptions::mdx();
    assert!(options.mdx);
    assert!(!options.gfm);
    assert!(!options.footnotes);
    assert!(!options.task_lists);
    assert!(!options.tables);
    assert!(!options.strikethrough);
    assert!(!options.autolinks);
    assert_eq!(options.max_nesting_depth, 100);
}

#[test]
fn mdx_can_be_combined_with_gfm() {
    let mut options = ParserOptions::gfm();
    options.mdx = true;
    assert!(options.mdx);
    assert!(options.gfm);
    assert!(options.tables);
    assert_eq!(options.max_nesting_depth, 100);
}

#[test]
fn clone_preserves_mdx_flag() {
    let enabled = ParserOptions::mdx();
    let cloned = enabled.clone();
    assert!(cloned.mdx);
    assert_eq!(format!("{enabled:?}"), format!("{cloned:?}"));
}

#[test]
fn debug_mentions_mdx_field() {
    let debug = format!("{:?}", ParserOptions::default());
    assert!(debug.contains("mdx: false"), "Debug output must include the mdx field: {debug}");
    let debug = format!("{:?}", ParserOptions::mdx());
    assert!(debug.contains("mdx: true"), "Debug output must include the mdx field: {debug}");
}

#[test]
fn mdx_flag_is_noop_for_plain_paragraph() {
    assert_mdx_flag_is_noop("Just a paragraph.\n", ParserOptions::default());
}

#[test]
fn mdx_flag_is_noop_for_headings_and_lists() {
    assert_mdx_flag_is_noop("# Title\n\n- one\n- two\n", ParserOptions::default());
}

#[test]
fn mdx_flag_is_noop_for_fenced_code() {
    assert_mdx_flag_is_noop("```js\nconst x = <div />;\n```\n", ParserOptions::default());
}

#[test]
fn mdx_flag_is_noop_for_inline_code() {
    assert_mdx_flag_is_noop("Use `<Alert />` in MDX later.\n", ParserOptions::default());
}

#[test]
fn mdx_flag_is_noop_for_html_block() {
    assert_mdx_flag_is_noop("<div>\nraw\n</div>\n\nAfter\n", ParserOptions::default());
}

#[test]
fn mdx_flag_is_noop_for_inline_html() {
    assert_mdx_flag_is_noop("Hello <span class=\"x\">there</span>.\n", ParserOptions::default());
}

#[test]
fn mdx_flag_is_noop_for_gfm_table() {
    assert_mdx_flag_is_noop("| a | b |\n| - | - |\n| 1 | 2 |\n", ParserOptions::gfm());
}

#[test]
fn mdx_flag_is_noop_for_gfm_task_list() {
    assert_mdx_flag_is_noop("- [ ] todo\n- [x] done\n", ParserOptions::gfm());
}

#[test]
fn mdx_flag_is_noop_for_brace_expression() {
    assert_mdx_flag_is_noop("Hello {name}.\n", ParserOptions::default());
}

#[test]
fn mdx_flag_is_noop_for_nested_blockquote() {
    assert_mdx_flag_is_noop("> quote\n>\n> still quote\n", ParserOptions::default());
}

#[test]
fn mdx_false_does_not_emit_mdx_nodes_for_jsx() {
    let tree = pretty_ast("<Counter count={1} />\n", ParserOptions::default());
    assert!(!tree.contains("MdxJsx"), "mdx=false must not emit MDX JSX nodes:\n{tree}");
}
