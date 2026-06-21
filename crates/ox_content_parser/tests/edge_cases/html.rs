use ox_content_allocator::Allocator;
use ox_content_ast::Node;
use ox_content_parser::ParserOptions;

use super::{first_text_in_nodes, parse_with_options};

#[test]
fn html_details_block_is_preserved_as_raw_html() {
    let allocator = Allocator::new();
    let doc = parse_with_options(
        &allocator,
        "<details id=\"symbol\">\n<summary>Symbol</summary>\n<p>Expanded docs</p>\n</details>\n\nAfter",
        ParserOptions::default(),
    );

    match &doc.children[0] {
        Node::Html(html) => {
            insta::assert_snapshot!(html.value);
        }
        other => panic!("expected html block, got {other:?}"),
    }

    match &doc.children[1] {
        Node::Paragraph(paragraph) => {
            assert_eq!(first_text_in_nodes(paragraph.children.iter()), Some("After"));
        }
        other => panic!("expected paragraph after html block, got {other:?}"),
    }
}

#[test]
fn html_type6_details_terminates_at_first_blank_line() {
    let allocator = Allocator::new();
    let doc = parse_with_options(
        &allocator,
        "<details>\n\n<summary>Click to expand</summary>\n\n**bold should be markdown**\n\n- list\n\n```js\nconsole.log(\"code\");\n```\n\n</details>",
        ParserOptions::default(),
    );

    assert!(matches!(&doc.children[0], Node::Html(html) if html.value.trim() == "<details>"));
    assert!(matches!(&doc.children[1], Node::Html(_)));
    assert!(
        matches!(&doc.children[2], Node::Paragraph(paragraph) if matches!(&paragraph.children[0], Node::Strong(_)))
    );
    assert!(matches!(&doc.children[3], Node::List(_)));
    assert!(matches!(&doc.children[4], Node::CodeBlock(block) if block.lang == Some("js")));
    assert!(matches!(&doc.children[5], Node::Html(_)));
    insta::assert_debug_snapshot!(doc);
}

#[test]
fn html_type6_div_stops_before_markdown_after_blank_line() {
    let allocator = Allocator::new();
    let doc = parse_with_options(
        &allocator,
        "<div>\nraw html line\n\n**markdown**\n\n</div>",
        ParserOptions::default(),
    );

    assert!(matches!(&doc.children[0], Node::Html(_)));
    assert!(
        matches!(&doc.children[1], Node::Paragraph(paragraph) if matches!(&paragraph.children[0], Node::Strong(_)))
    );
    assert!(matches!(&doc.children[2], Node::Html(_)));
    insta::assert_debug_snapshot!(doc);
}

#[test]
fn html_type1_pre_ignores_blank_lines_until_closing_tag() {
    let allocator = Allocator::new();
    let doc = parse_with_options(
        &allocator,
        "<pre>\n\n**not markdown**\n</pre>\n\nAfter",
        ParserOptions::default(),
    );

    match &doc.children[0] {
        Node::Html(html) => {
            insta::assert_snapshot!(html.value);
        }
        other => panic!("expected html block, got {other:?}"),
    }
    assert!(matches!(&doc.children[1], Node::Paragraph(_)));
}
