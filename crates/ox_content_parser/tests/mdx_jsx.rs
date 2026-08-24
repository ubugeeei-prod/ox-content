//! JSX element parse when `ParserOptions.mdx` is true.
//!
//! Covers PascalCase elements (flow + text), literal / boolean / `{expr}`
//! attributes, self-closing tags, and simple open/close children. Lowercase
//! HTML stays `Html`. Fragments, spreads, comments, member names, and
//! children expressions live in `mdx_jsx_remainder.rs`. Document-level
//! `{expression}` is covered in `mdx_expressions.rs`. Module-level
//! `import` / `export` is covered in `mdx_esm.rs`.

use ox_content_allocator::Allocator;
use ox_content_parser::{Parser, ParserOptions};

#[path = "support/pretty.rs"]
mod pretty;

fn pretty_ast(source: &str, options: ParserOptions) -> String {
    let allocator = Allocator::new();
    let doc = Parser::with_options(&allocator, source, options)
        .parse()
        .expect("parser should not fail on MDX JSX fixtures");
    let mut out = String::new();
    pretty::format_document(&doc, source, &mut out);
    out
}

fn mdx_tree(source: &str) -> String {
    pretty_ast(source, ParserOptions::mdx())
}

#[test]
fn mdx_false_does_not_emit_jsx_for_pascal_case() {
    let tree = pretty_ast("<Alert title=\"hi\" />\n", ParserOptions::default());
    assert!(!tree.contains("MdxJsx"), "mdx=false must not emit JSX:\n{tree}");
}

#[test]
fn disabled_mdx_does_not_parse_jsx_children() {
    let source = "<Callout>\n\n# Title\n\nHello **world**.\n\n</Callout>\n";
    let tree = pretty_ast(source, ParserOptions::default());
    assert!(!tree.contains("MdxJsx"), "mdx=false must not wrap children in JSX:\n{tree}");
    assert!(tree.contains("Heading"), "markdown after the opener still parses:\n{tree}");
    assert!(tree.contains("Strong"), "phrasing is not swallowed:\n{tree}");
}

#[test]
fn flow_self_closing_literal_attr() {
    let tree = mdx_tree("<Alert title=\"hi\" />\n");
    assert!(
        tree.contains("MdxJsxFlowElement name=Some(\"Alert\") self_closing=true"),
        "expected flow Alert:\n{tree}"
    );
    assert!(
        tree.contains("Attr name=\"title\" value=literal(\"hi\")"),
        "expected literal title:\n{tree}"
    );
    assert!(!tree.contains("Paragraph"), "standalone tag must not wrap in a paragraph:\n{tree}");
}

#[test]
fn text_self_closing_inside_paragraph() {
    let tree = mdx_tree("Hello <Badge /> world.\n");
    assert!(tree.contains("Paragraph"), "expected a paragraph:\n{tree}");
    assert!(
        tree.contains("MdxJsxTextElement name=Some(\"Badge\") self_closing=true"),
        "expected text Badge:\n{tree}"
    );
    assert!(tree.contains("Text \"Hello \""), "expected leading text:\n{tree}");
    assert!(tree.contains("Text \" world.\""), "expected trailing text:\n{tree}");
}

#[test]
fn boolean_and_expression_attrs() {
    let tree = mdx_tree("<Btn disabled count={1+1} />\n");
    assert!(
        tree.contains("MdxJsxFlowElement name=Some(\"Btn\") self_closing=true"),
        "expected flow Btn:\n{tree}"
    );
    assert!(tree.contains("Attr name=\"disabled\" value=boolean"), "expected boolean:\n{tree}");
    assert!(
        tree.contains("Attr name=\"count\" value=expression(\"1+1\")"),
        "expression attrs store source, not a evaluated result:\n{tree}"
    );
}

#[test]
fn hostile_quoted_attr_is_literal_source() {
    let tree = mdx_tree("<Alert title=\"<script>\" />\n");
    assert!(
        tree.contains("Attr name=\"title\" value=literal(\"<script>\")"),
        "quoted attr must keep source text:\n{tree}"
    );
}

#[test]
fn unclosed_tag_does_not_panic_or_emit_jsx() {
    let tree = mdx_tree("<Alert\n");
    assert!(!tree.contains("MdxJsx"), "unclosed opener stays non-JSX:\n{tree}");
}

#[test]
fn unclosed_component_does_not_swallow_file() {
    let tree = mdx_tree("<Callout>\n\n# Still here\n\nAfter the unclosed tag.\n");
    assert!(!tree.contains("MdxJsx"), "unclosed opener is not JSX:\n{tree}");
    assert!(tree.contains("Heading"), "heading after the opener still parses:\n{tree}");
    assert!(tree.contains("After the unclosed tag."), "trailing prose is not swallowed:\n{tree}");
}

#[test]
fn fenced_and_inline_code_are_not_components() {
    let fenced = mdx_tree("```js\nconst x = <div />;\n```\n");
    assert!(fenced.contains("Code"), "expected a fence:\n{fenced}");
    assert!(!fenced.contains("MdxJsx"), "fence contents are not JSX:\n{fenced}");

    let inline = mdx_tree("Use `<Alert />` in prose.\n");
    assert!(inline.contains("InlineCode"), "expected inline code:\n{inline}");
    assert!(!inline.contains("MdxJsx"), "inline code is not JSX:\n{inline}");
}

#[test]
fn lowercase_html_stays_html_when_mdx_is_on() {
    let block = mdx_tree("<div>\nraw\n</div>\n\nAfter\n");
    assert!(block.contains("Html"), "lowercase block stays Html:\n{block}");
    assert!(!block.contains("MdxJsx"), "lowercase HTML is not JSX in this slice:\n{block}");

    let inline = mdx_tree("Hello <span class=\"x\">there</span>.\n");
    assert!(inline.contains("Html"), "lowercase inline stays Html:\n{inline}");
    assert!(!inline.contains("MdxJsx"), "lowercase HTML is not JSX in this slice:\n{inline}");
}

#[test]
fn flow_open_close_parses_markdown_children() {
    let tree = mdx_tree("<Alert>\nHello **world**\n</Alert>\n");
    assert!(
        tree.contains("MdxJsxFlowElement name=Some(\"Alert\") self_closing=false"),
        "expected open/close flow:\n{tree}"
    );
    assert!(tree.contains("Strong"), "markdown children should parse:\n{tree}");
    assert!(!tree.contains("MdxFlowExpression"), "this fixture has no child expression:\n{tree}");
}

#[test]
fn fence_inside_component_is_code_not_island() {
    let tree = mdx_tree("<Callout>\n\n```js\nconst x = <Alert />;\n```\n\n</Callout>\n");
    assert!(
        tree.contains("MdxJsxFlowElement name=Some(\"Callout\")"),
        "wrapper still parses:\n{tree}"
    );
    assert!(tree.contains("Code"), "inner fence is a code node:\n{tree}");
    assert!(!tree.contains("name=Some(\"Alert\")"), "fence JSX is not a component:\n{tree}");
}

#[test]
fn text_open_close_parses_phrasing_children() {
    let tree = mdx_tree("Hello <Badge>x</Badge> world.\n");
    assert!(
        tree.contains("MdxJsxTextElement name=Some(\"Badge\") self_closing=false"),
        "expected open/close text:\n{tree}"
    );
    assert!(tree.contains("Text \"x\""), "expected phrasing child:\n{tree}");
}

#[test]
fn pascal_case_flow_interrupts_paragraph() {
    let tree = mdx_tree("Hello\n<Alert />\n");
    assert!(tree.contains("Paragraph"), "leading prose stays a paragraph:\n{tree}");
    assert!(
        tree.contains("MdxJsxFlowElement name=Some(\"Alert\") self_closing=true"),
        "PascalCase at line start is flow:\n{tree}"
    );
}

#[test]
fn spread_attr_is_expression_entry() {
    let tree = mdx_tree("<Alert {...props} />\n");
    assert!(
        tree.contains("MdxJsxFlowElement name=Some(\"Alert\") self_closing=true"),
        "expected flow Alert with spread:\n{tree}"
    );
    assert!(
        tree.contains("AttrExpr value=\"...props\""),
        "spreads store source, not a evaluated object:\n{tree}"
    );
}

#[test]
fn fragment_parses_as_nameless_jsx() {
    let tree = mdx_tree("<>hello</>\n");
    assert!(
        tree.contains("MdxJsxFlowElement name=None self_closing=false"),
        "expected a fragment:\n{tree}"
    );
    assert!(tree.contains("Text \"hello\""), "fragment children stay markdown:\n{tree}");
}

#[test]
fn jsx_comment_is_flow_expression() {
    let tree = mdx_tree("{/* hide */}\n");
    assert!(
        tree.contains("MdxFlowExpression value=\"/* hide */\""),
        "JSX comments store source as an expression:\n{tree}"
    );
    assert!(!tree.contains("MdxJsx"), "a comment is not a JSX element:\n{tree}");
}

#[test]
fn member_expression_name_parses() {
    let tree = mdx_tree("<Foo.Bar />\n");
    assert!(
        tree.contains("MdxJsxFlowElement name=Some(\"Foo.Bar\") self_closing=true"),
        "expected member name:\n{tree}"
    );
}

#[test]
fn children_brace_expression_stores_source() {
    let tree = mdx_tree("<Alert>{items.map}</Alert>\n");
    assert!(
        tree.contains("MdxJsxFlowElement name=Some(\"Alert\")"),
        "wrapper still parses:\n{tree}"
    );
    assert!(
        tree.contains("MdxFlowExpression value=\"items.map\""),
        "children expressions store source, not a evaluated result:\n{tree}"
    );
}
