//! ESM `import` / `export` parse when `ParserOptions.mdx` is true.
//!
//! Block-level statements become `MdxjsEsm` and keep their source text.
//! Fences, inline code, and invalid or unclosed statements stay Markdown.
//! JSX from the previous slice must keep working.

use ox_content_allocator::Allocator;
use ox_content_ast::{Document, MdxjsEsm, Node};
use ox_content_parser::{Parser, ParserOptions};

#[path = "support/pretty.rs"]
mod pretty;

fn pretty_ast(source: &str, options: ParserOptions) -> String {
    let allocator = Allocator::new();
    let doc = Parser::with_options(&allocator, source, options)
        .parse()
        .expect("parser should not fail on MDX ESM fixtures");
    let mut out = String::new();
    pretty::format_document(&doc, source, &mut out);
    out
}

fn mdx_tree(source: &str) -> String {
    pretty_ast(source, ParserOptions::mdx())
}

fn parse_mdx<'a>(allocator: &'a Allocator, source: &'a str) -> Document<'a> {
    Parser::with_options(allocator, source, ParserOptions::mdx())
        .parse()
        .expect("parser should not fail on MDX ESM fixtures")
}

fn only_esm<'a>(doc: &'a Document<'a>) -> &'a MdxjsEsm<'a> {
    assert_eq!(doc.children.len(), 1, "expected a single top-level node, got {:?}", doc.children);
    match &doc.children[0] {
        Node::MdxjsEsm(esm) => esm,
        other => panic!("expected MdxjsEsm, got {other:?}"),
    }
}

#[test]
fn disabled_mdx_leaves_import_as_paragraph() {
    let source = "import Foo from \"./Foo\"\n";
    let tree = pretty_ast(source, ParserOptions::default());
    assert!(!tree.contains("MdxjsEsm"), "mdx=false must not emit ESM:\n{tree}");
    assert!(tree.contains("Paragraph"), "import stays a paragraph when mdx is off:\n{tree}");
    assert!(tree.contains("import Foo from"), "source text is preserved:\n{tree}");
}

#[test]
fn import_default_becomes_esm_node() {
    let source = "import Foo from \"./Foo\"\n";
    let allocator = Allocator::new();
    let doc = parse_mdx(&allocator, source);
    let esm = only_esm(&doc);
    assert_eq!(esm.value, "import Foo from \"./Foo\"");
    assert!(esm.value.contains("./Foo"), "module source stays on the node: {}", esm.value);
    let tree = mdx_tree(source);
    assert!(tree.contains("MdxjsEsm"), "expected ESM node:\n{tree}");
    assert!(!tree.contains("Paragraph"), "standalone import is not a paragraph:\n{tree}");
}

#[test]
fn import_named_becomes_esm_node() {
    let source = "import { Foo, Bar as Baz } from \"./mod\"\n";
    let allocator = Allocator::new();
    let doc = parse_mdx(&allocator, source);
    let esm = only_esm(&doc);
    assert_eq!(esm.value, "import { Foo, Bar as Baz } from \"./mod\"");
    assert!(esm.value.contains("Bar as Baz"), "named specifiers stay source text: {}", esm.value);
    assert!(esm.value.contains("./mod"), "module source stays source text: {}", esm.value);
}

#[test]
fn export_const_becomes_esm_node() {
    let source = "export const meta = { title: \"Hi\" }\n";
    let allocator = Allocator::new();
    let doc = parse_mdx(&allocator, source);
    let esm = only_esm(&doc);
    assert_eq!(esm.value, "export const meta = { title: \"Hi\" }");
    assert!(esm.value.contains("title"), "export initializer stays source text: {}", esm.value);
    let tree = mdx_tree(source);
    assert!(!tree.contains("Paragraph"), "standalone export is not a paragraph:\n{tree}");
}

#[test]
fn export_default_becomes_esm_node() {
    let source = "export default Foo\n";
    let allocator = Allocator::new();
    let doc = parse_mdx(&allocator, source);
    let esm = only_esm(&doc);
    assert_eq!(esm.value, "export default Foo");
}

#[test]
fn skips_fenced_code() {
    let tree =
        mdx_tree("```js\nimport Foo from \"./Foo\"\nexport const meta = { title: \"Hi\" }\n```\n");
    assert!(tree.contains("CodeBlock"), "expected a fence:\n{tree}");
    assert!(!tree.contains("MdxjsEsm"), "fence contents are not ESM:\n{tree}");
}

#[test]
fn skips_inline_code() {
    let tree = mdx_tree("Use `import Foo from \"./Foo\"` in prose.\n");
    assert!(tree.contains("InlineCode"), "expected inline code:\n{tree}");
    assert!(!tree.contains("MdxjsEsm"), "inline code is not ESM:\n{tree}");
    assert!(tree.contains("Paragraph"), "prose stays a paragraph:\n{tree}");
}

#[test]
fn invalid_import_stays_markdown() {
    let fixtures = [
        "import\n",
        "import Foo\n",
        "import Foo from\n",
        "import { Foo\n",
        "import Foo from \"./Foo\n",
        "important thing\n",
        "importantly Foo from \"./Foo\"\n",
        "import Foo from\n\n\"./Foo\"\n",
    ];
    for source in fixtures {
        let tree = mdx_tree(source);
        assert!(
            !tree.contains("MdxjsEsm"),
            "invalid or unclosed import must stay Markdown for {source:?}:\n{tree}"
        );
        assert!(
            tree.contains("Paragraph") || tree.contains("Text"),
            "invalid import must remain Markdown for {source:?}:\n{tree}"
        );
    }
}

#[test]
fn existing_jsx_parse_still_works() {
    let tree = mdx_tree("<Alert title=\"hi\" />\n");
    assert!(
        tree.contains("MdxJsxFlowElement name=Some(\"Alert\") self_closing=true"),
        "JSX flow parse must still work:\n{tree}"
    );
    assert!(
        tree.contains("Attr name=\"title\" value=literal(\"hi\")"),
        "JSX attributes must still parse:\n{tree}"
    );

    let mixed = mdx_tree("import Foo from \"./Foo\"\n\n<Alert title=\"hi\" />\n");
    assert!(mixed.contains("MdxjsEsm"), "import should parse beside JSX:\n{mixed}");
    assert!(
        mixed.contains("MdxJsxFlowElement name=Some(\"Alert\")"),
        "JSX after import must still parse:\n{mixed}"
    );
    assert!(!mixed.contains("Paragraph"), "mixed ESM + JSX must not wrap in paragraphs:\n{mixed}");
}

#[test]
fn disabled_mdx_leaves_export_as_paragraph() {
    let tree = pretty_ast("export const meta = { title: \"Hi\" }\n", ParserOptions::default());
    assert!(!tree.contains("MdxjsEsm"), "mdx=false must not emit ESM:\n{tree}");
    assert!(tree.contains("Paragraph"), "export stays a paragraph when mdx is off:\n{tree}");
}

#[test]
fn hostile_module_source_is_kept_as_text() {
    let source = "import Foo from \"../../../../etc/passwd\"\n";
    let allocator = Allocator::new();
    let doc = parse_mdx(&allocator, source);
    let esm = only_esm(&doc);
    assert_eq!(esm.value, "import Foo from \"../../../../etc/passwd\"");
    assert!(
        !std::path::Path::new("../../../../etc/passwd").exists()
            || esm.value.contains("../../../../etc/passwd"),
        "hostile paths are stored as text; the parser must not resolve them"
    );
}

#[test]
fn optional_semicolon_is_kept_on_the_node() {
    let source = "import Foo from \"./Foo\";\n";
    let allocator = Allocator::new();
    let doc = parse_mdx(&allocator, source);
    let esm = only_esm(&doc);
    assert_eq!(esm.value, "import Foo from \"./Foo\";");
}

#[test]
fn import_interrupts_paragraph() {
    let tree = mdx_tree("Hello\nimport Foo from \"./Foo\"\n");
    assert!(tree.contains("Paragraph"), "leading prose stays a paragraph:\n{tree}");
    assert!(tree.contains("MdxjsEsm"), "import at line start is a block:\n{tree}");
    assert!(tree.contains("Text \"Hello\""), "prose text is unchanged:\n{tree}");
}

#[test]
fn invalid_import_does_not_interrupt_paragraph() {
    let tree = mdx_tree("Hello\nimport Foo from\n");
    assert!(!tree.contains("MdxjsEsm"), "incomplete import must not become ESM:\n{tree}");
    assert!(tree.contains("Paragraph"), "incomplete import stays Markdown with the prose:\n{tree}");
}
