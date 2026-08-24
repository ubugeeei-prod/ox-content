//! Document-level / prose `{expression}` when `ParserOptions.mdx` is true.
//!
//! Flow and text expressions store raw source. Nothing is evaluated.
//! Fences, inline code, `mdx=false`, and unclosed `{` must not emit a
//! half-parsed expression node. `import` / `export` braces stay ESM.

use ox_content_allocator::Allocator;
use ox_content_parser::{Parser, ParserOptions};

#[path = "support/pretty.rs"]
mod pretty;

fn pretty_ast(source: &str, options: ParserOptions) -> String {
    let allocator = Allocator::new();
    let doc = Parser::with_options(&allocator, source, options)
        .parse()
        .expect("parser should not fail on MDX expression fixtures");
    let mut out = String::new();
    pretty::format_document(&doc, source, &mut out);
    out
}

fn mdx_tree(source: &str) -> String {
    pretty_ast(source, ParserOptions::mdx())
}

fn assert_flow(tree: &str, value: &str) {
    let needle = format!("MdxFlowExpression value={value:?}");
    assert!(tree.contains(&needle), "expected {needle} in:\n{tree}");
}

fn assert_text(tree: &str, value: &str) {
    let needle = format!("MdxTextExpression value={value:?}");
    assert!(tree.contains(&needle), "expected {needle} in:\n{tree}");
}

#[test]
fn mdx_false_flow_brace_stays_paragraph() {
    let tree = pretty_ast("{foo}\n", ParserOptions::default());
    assert!(tree.contains("Paragraph"), "mdx=false flow brace stays a paragraph:\n{tree}");
    assert!(tree.contains("Text \"{foo}\""), "expected literal text:\n{tree}");
    assert!(!tree.contains("MdxFlowExpression"), "mdx=false must not emit flow expr:\n{tree}");
}

#[test]
fn mdx_false_text_brace_stays_text() {
    let tree = pretty_ast("Hello {name}.\n", ParserOptions::default());
    assert!(tree.contains("Text \"Hello {name}.\""), "mdx=false keeps braces as text:\n{tree}");
    assert!(!tree.contains("MdxTextExpression"), "mdx=false must not emit text expr:\n{tree}");
}

#[test]
fn flow_identifier_is_expression() {
    let tree = mdx_tree("{foo}\n");
    assert_flow(&tree, "foo");
    assert!(!tree.contains("Paragraph"), "standalone flow expr is not a paragraph:\n{tree}");
}

#[test]
fn flow_arithmetic_stores_source() {
    let tree = mdx_tree("{count + 1}\n");
    assert_flow(&tree, "count + 1");
    assert!(!tree.contains("value=\"2\""), "must not evaluate JS:\n{tree}");
}

#[test]
fn text_identifier_in_prose() {
    let tree = mdx_tree("Hello {name}.\n");
    assert!(tree.contains("Paragraph"), "prose stays a paragraph:\n{tree}");
    assert!(tree.contains("Text \"Hello \""), "expected leading text:\n{tree}");
    assert_text(&tree, "name");
    assert!(tree.contains("Text \".\""), "expected trailing text:\n{tree}");
}

#[test]
fn text_arithmetic_in_prose() {
    let tree = mdx_tree("Total {count + 1} items.\n");
    assert_text(&tree, "count + 1");
    assert!(!tree.contains("value=\"2\""), "must not evaluate JS:\n{tree}");
}

#[test]
fn nested_braces_and_member_access() {
    let tree = mdx_tree("{foo({ bar: 1 })}\n");
    assert_flow(&tree, "foo({ bar: 1 })");

    let text = mdx_tree("See {user.name}.\n");
    assert_text(&text, "user.name");
}

#[test]
fn string_and_template_may_contain_closing_brace() {
    let tree = mdx_tree("{ foo(\"}\") }\n");
    assert_flow(&tree, " foo(\"}\") ");

    let template = mdx_tree("{ `close } here` }\n");
    assert_flow(&template, " `close } here` ");
}

#[test]
fn fenced_and_inline_code_are_not_expressions() {
    let fenced = mdx_tree("```js\nconst x = {foo};\n```\n");
    assert!(fenced.contains("Code"), "expected a fence:\n{fenced}");
    assert!(
        !fenced.contains("MdxFlowExpression") && !fenced.contains("MdxTextExpression"),
        "fence contents are not expressions:\n{fenced}"
    );

    let inline = mdx_tree("Use `{foo}` and `{count + 1}` in prose.\n");
    assert!(inline.contains("InlineCode"), "expected inline code:\n{inline}");
    assert!(
        !inline.contains("MdxTextExpression") && !inline.contains("MdxFlowExpression"),
        "inline code is not an expression:\n{inline}"
    );
}

#[test]
fn unclosed_brace_does_not_panic_or_emit_half_node() {
    for source in ["{foo\n", "Hello {name\n", "{ foo(\"\n", "{ /* hide\n"] {
        let tree = mdx_tree(source);
        assert!(
            !tree.contains("MdxFlowExpression") && !tree.contains("MdxTextExpression"),
            "unclosed {source:?} must not emit an expression:\n{tree}"
        );
    }
}

#[test]
fn hostile_script_stores_source_without_evaluating() {
    let tree = mdx_tree("{ \"<script>alert(1)</script>\" }\n");
    assert_flow(&tree, " \"<script>alert(1)</script>\" ");
    assert!(!tree.contains("value=\"2\""), "source is stored, not evaluated:\n{tree}");
}

#[test]
fn import_and_export_braces_stay_esm() {
    let import = mdx_tree("import { Chart } from './Chart'\n");
    assert!(
        import.contains("MdxjsEsm value=\"import { Chart } from './Chart'\""),
        "import:\n{import}"
    );
    assert!(
        !import.contains("MdxFlowExpression") && !import.contains("MdxTextExpression"),
        "ESM braces are not prose expressions:\n{import}"
    );

    let export = mdx_tree("export const meta = { title: 'Hi' }\n");
    assert!(export.contains("MdxjsEsm"), "export stays ESM:\n{export}");
    assert!(
        !export.contains("MdxFlowExpression") && !export.contains("MdxTextExpression"),
        "export object is not a flow expression:\n{export}"
    );
}

#[test]
fn flow_expression_interrupts_paragraph() {
    let tree = mdx_tree("Hello\n{name}\n");
    assert!(tree.contains("Paragraph"), "leading prose stays a paragraph:\n{tree}");
    assert!(tree.contains("Text \"Hello\""), "expected leading text:\n{tree}");
    assert_flow(&tree, "name");
}

#[test]
fn trailing_text_on_same_line_is_text_expression() {
    let tree = mdx_tree("{foo} extra\n");
    assert!(tree.contains("Paragraph"), "same-line trailing text is a paragraph:\n{tree}");
    assert_text(&tree, "foo");
    assert!(tree.contains("Text \" extra\""), "expected trailing text:\n{tree}");
    assert!(!tree.contains("MdxFlowExpression"), "not a flow construct:\n{tree}");
}

#[test]
fn escaped_brace_is_not_an_expression() {
    let tree = mdx_tree("Hello \\{name}\n");
    assert!(!tree.contains("MdxTextExpression"), "backslash-escaped brace is text:\n{tree}");
    assert!(tree.contains("Text \"{\""), "escaped {{ is literal:\n{tree}");
}

#[test]
fn multiline_flow_expression_stores_source() {
    let tree = mdx_tree("{\n  count + 1\n}\n");
    assert_flow(&tree, "\n  count + 1\n");
    assert!(!tree.contains("Paragraph"), "multiline flow is not a paragraph:\n{tree}");
}
