//! Remaining JSX grammar when `ParserOptions.mdx` is true.
//!
//! Fragments, spreads, JSX comments, member names, children expressions,
//! and markdown-in-JSX. Source is stored; nothing is evaluated. `mdx=false`,
//! fences, inline code, and unclosed input must not emit MDX nodes.

use ox_content_allocator::Allocator;
use ox_content_parser::{Parser, ParserOptions};

#[path = "support/pretty.rs"]
mod pretty;

fn pretty_ast(source: &str, options: ParserOptions) -> String {
    let allocator = Allocator::new();
    let doc = Parser::with_options(&allocator, source, options)
        .parse()
        .expect("parser should not fail on MDX JSX remainder fixtures");
    let mut out = String::new();
    pretty::format_document(&doc, source, &mut out);
    out
}

fn mdx_tree(source: &str) -> String {
    pretty_ast(source, ParserOptions::mdx())
}

#[test]
fn disabled_mdx_does_not_emit_remainder_constructs() {
    let sources = [
        "<>hello</>\n",
        "<Alert {...props} />\n",
        "{/* hide */}\n",
        "<Foo.Bar />\n",
        "<Alert>{items.map}</Alert>\n",
        "<Foo.Bar.Baz title=\"hi\" />\n",
    ];
    for source in sources {
        let tree = pretty_ast(source, ParserOptions::default());
        assert!(
            !tree.contains("MdxJsx")
                && !tree.contains("MdxFlowExpression")
                && !tree.contains("MdxTextExpression")
                && !tree.contains("AttrExpr"),
            "mdx=false must not emit MDX nodes for {source:?}:\n{tree}"
        );
    }
}

#[test]
fn text_fragment_parses_inside_paragraph() {
    let tree = mdx_tree("Hello <>x</> world.\n");
    assert!(tree.contains("Paragraph"), "expected a paragraph:\n{tree}");
    assert!(
        tree.contains("MdxJsxTextElement name=None self_closing=false"),
        "expected a text fragment:\n{tree}"
    );
    assert!(tree.contains("Text \"x\""), "expected fragment child:\n{tree}");
}

#[test]
fn named_fragment_component_stays_pascal_case() {
    let tree = mdx_tree("<Fragment>hello</Fragment>\n");
    assert!(
        tree.contains("MdxJsxFlowElement name=Some(\"Fragment\") self_closing=false"),
        "expected named Fragment:\n{tree}"
    );
    assert!(tree.contains("Text \"hello\""), "expected markdown child:\n{tree}");
}

#[test]
fn nested_fragments_match_by_depth() {
    let tree = mdx_tree("<><>inner</>outer</>\n");
    let flow = tree.matches("MdxJsxFlowElement name=None").count();
    assert!(flow >= 2, "expected nested fragments:\n{tree}");
    assert!(tree.contains("Text \"inner\""), "expected inner text:\n{tree}");
    assert!(tree.contains("Text \"outer\""), "expected outer text:\n{tree}");
}

#[test]
fn indented_flow_children_parse_as_mdx_blocks() {
    let tree = mdx_tree(
        "<Docs.Layout>\n  <Docs.Header eyebrow=\"Guide\">\n    <>Build <Icons.Sparkle /> faster</>\n  </Docs.Header>\n\n  <Docs.Body>\n    Use <Package.Name scope=\"@ox-content\" /> with {runtime}.\n  </Docs.Body>\n</Docs.Layout>\n",
    );
    assert!(
        tree.contains("MdxJsxFlowElement name=Some(\"Docs.Header\")"),
        "indented Header should stay JSX:\n{tree}"
    );
    assert!(
        tree.contains("MdxJsxFlowElement name=None self_closing=false"),
        "indented fragment should stay JSX:\n{tree}"
    );
    assert!(
        tree.contains("MdxJsxTextElement name=Some(\"Icons.Sparkle\")"),
        "nested icon should stay JSX:\n{tree}"
    );
    assert!(
        tree.contains("MdxTextExpression value=\"runtime\""),
        "indented prose expression should stay MDX:\n{tree}"
    );
    assert!(!tree.contains("CodeBlock"), "authoring indentation is not code:\n{tree}");
}

#[test]
fn indented_fenced_code_inside_flow_jsx_remains_code() {
    let tree = mdx_tree("<Callout>\n  ```sh\n  npm test\n  ```\n</Callout>\n");
    assert!(
        tree.contains("MdxJsxFlowElement name=Some(\"Callout\")"),
        "wrapper still parses:\n{tree}"
    );
    assert!(
        tree.contains("CodeBlock lang=Some(\"sh\")") && tree.contains("value=\"npm test\\n\""),
        "fenced code stays explicit code with authoring indent stripped:\n{tree}"
    );
}

#[test]
fn spread_can_mix_with_named_attrs() {
    let tree = mdx_tree("<Btn disabled {...rest} title=\"hi\" />\n");
    assert!(tree.contains("Attr name=\"disabled\" value=boolean"), "expected boolean:\n{tree}");
    assert!(tree.contains("AttrExpr value=\"...rest\""), "expected spread:\n{tree}");
    assert!(
        tree.contains("Attr name=\"title\" value=literal(\"hi\")"),
        "expected literal after spread:\n{tree}"
    );
}

#[test]
fn hostile_spread_stores_source_without_evaluating() {
    let tree = mdx_tree("<Alert {...{dangerouslySetInnerHTML:{__html:\"<script>\"}}} />\n");
    assert!(
        tree.contains("AttrExpr value=") && tree.contains("<script>"),
        "hostile spread must keep source text:\n{tree}"
    );
    assert!(tree.contains("MdxJsxFlowElement name=Some(\"Alert\")"), "tag still parses:\n{tree}");
}

#[test]
fn jsx_comment_inside_element_is_expression() {
    let tree = mdx_tree("<Alert>{/* note */}hello</Alert>\n");
    assert!(tree.contains("MdxJsxFlowElement name=Some(\"Alert\")"), "expected wrapper:\n{tree}");
    assert!(
        tree.contains("MdxTextExpression value=\"/* note */\"")
            || tree.contains("MdxFlowExpression value=\"/* note */\""),
        "comment child stores source:\n{tree}"
    );
    assert!(tree.contains("Text \"hello\""), "text after comment remains:\n{tree}");
}

#[test]
fn jsx_comment_in_text_jsx_is_text_expression() {
    let tree = mdx_tree("Hello <Badge>{/* n */}x</Badge>.\n");
    assert!(
        tree.contains("MdxJsxTextElement name=Some(\"Badge\")"),
        "expected text Badge:\n{tree}"
    );
    assert!(tree.contains("MdxTextExpression value=\"/* n */\""), "expected text comment:\n{tree}");
}

#[test]
fn member_name_can_nest_and_take_attrs() {
    let tree = mdx_tree("<Foo.Bar.Baz title=\"hi\" />\n");
    assert!(
        tree.contains("MdxJsxFlowElement name=Some(\"Foo.Bar.Baz\") self_closing=true"),
        "expected dotted name:\n{tree}"
    );
    assert!(
        tree.contains("Attr name=\"title\" value=literal(\"hi\")"),
        "expected literal attr:\n{tree}"
    );
}

#[test]
fn text_member_name_parses_inside_paragraph() {
    let tree = mdx_tree("Use <Icons.Star /> here.\n");
    assert!(
        tree.contains("MdxJsxTextElement name=Some(\"Icons.Star\") self_closing=true"),
        "expected text member:\n{tree}"
    );
}

#[test]
fn text_children_expression_stores_source() {
    let tree = mdx_tree("Hello <Badge>{label}</Badge> world.\n");
    assert!(
        tree.contains("MdxJsxTextElement name=Some(\"Badge\")"),
        "expected text Badge:\n{tree}"
    );
    assert!(
        tree.contains("MdxTextExpression value=\"label\""),
        "text children expressions store source:\n{tree}"
    );
}

#[test]
fn mixed_phrasing_and_expression_children() {
    let tree = mdx_tree("Hi <Badge>**{label}**</Badge>.\n");
    assert!(tree.contains("MdxJsxTextElement name=Some(\"Badge\")"), "expected Badge:\n{tree}");
    assert!(tree.contains("Strong"), "phrasing markdown still parses:\n{tree}");
    assert!(
        tree.contains("MdxTextExpression value=\"label\""),
        "expression inside emphasis stores source:\n{tree}"
    );
}

#[test]
fn fenced_and_inline_code_ignore_remainder_constructs() {
    let fenced = mdx_tree("```js\n<>x</>\n<Foo.Bar {...p} />\n{/* c */}\n```\n");
    assert!(fenced.contains("Code"), "expected a fence:\n{fenced}");
    assert!(
        !fenced.contains("MdxJsx")
            && !fenced.contains("MdxFlowExpression")
            && !fenced.contains("AttrExpr"),
        "fence contents are not JSX:\n{fenced}"
    );

    let inline = mdx_tree("Use `<>x</>` and `{/* c */}` and `<Foo.Bar />`.\n");
    assert!(inline.contains("InlineCode"), "expected inline code:\n{inline}");
    assert!(
        !inline.contains("MdxJsx") && !inline.contains("MdxTextExpression"),
        "inline code is not JSX:\n{inline}"
    );
}

#[test]
fn unclosed_remainder_constructs_do_not_panic_or_emit() {
    for source in ["<>hello\n", "<Alert {...props\n", "{/* hide\n", "<Foo.Bar\n", "<Foo.\n"] {
        let tree = mdx_tree(source);
        assert!(
            !tree.contains("MdxJsx") && !tree.contains("MdxFlowExpression"),
            "unclosed {source:?} stays non-JSX:\n{tree}"
        );
    }
}

#[test]
fn unclosed_children_expression_does_not_emit_expression() {
    let tree = mdx_tree("<Alert>{items.map</Alert>\n");
    assert!(
        !tree.contains("MdxFlowExpression") && !tree.contains("MdxTextExpression"),
        "unclosed children expr is not an expression node:\n{tree}"
    );
}

#[test]
fn flow_jsx_parses_heading_list_and_nested_markdown() {
    let tree = mdx_tree("<Alert>\n# Title\n\n- one\n- two\n\n> quote\n</Alert>\n");
    assert!(
        tree.contains("MdxJsxFlowElement name=Some(\"Alert\") self_closing=false"),
        "expected flow Alert:\n{tree}"
    );
    assert!(tree.contains("Heading depth=1"), "heading child:\n{tree}");
    assert!(tree.contains("List "), "list child:\n{tree}");
    assert!(tree.contains("BlockQuote"), "blockquote child:\n{tree}");
}

#[test]
fn flow_jsx_parses_nested_jsx_and_paragraphs() {
    let tree = mdx_tree("<Alert>\nHello **world**\n\n<Badge />\n</Alert>\n");
    assert!(tree.contains("Strong"), "paragraph phrasing:\n{tree}");
    assert!(tree.contains("MdxJsxFlowElement name=Some(\"Badge\")"), "nested flow JSX:\n{tree}");
}

#[test]
fn text_jsx_keeps_phrasing_not_flow() {
    let tree = mdx_tree("Hello <Badge>**x**</Badge> world.\n");
    assert!(tree.contains("Paragraph"), "outer prose is a paragraph:\n{tree}");
    assert!(
        tree.contains("MdxJsxTextElement name=Some(\"Badge\")"),
        "inline tag is text JSX:\n{tree}"
    );
    assert!(tree.contains("Strong"), "phrasing child:\n{tree}");
    assert!(!tree.contains("MdxJsxFlowElement"), "phrasing children must not become flow:\n{tree}");
}

#[test]
fn same_line_open_close_is_flow_with_paragraph_child() {
    let tree = mdx_tree("<Alert>hello</Alert>\n");
    assert!(
        tree.contains("MdxJsxFlowElement name=Some(\"Alert\") self_closing=false"),
        "line-start tag is flow:\n{tree}"
    );
    assert!(tree.contains("Paragraph"), "markdown child is a paragraph:\n{tree}");
    assert!(tree.contains("Text \"hello\""), "expected text:\n{tree}");
}
