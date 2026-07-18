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

/// Value of the first HTML block node.
fn first_html_value<'a>(doc: &ox_content_ast::Document<'a>) -> &'a str {
    match &doc.children[0] {
        Node::Html(html) => html.value,
        other => panic!("expected html block, got {other:?}"),
    }
}

#[test]
fn html_comment_closing_on_the_opener_line_ends_the_block() {
    let allocator = Allocator::new();
    let doc = parse_with_options(
        &allocator,
        "<!-- note --> tail\nstill html\n\nAfter",
        ParserOptions::default(),
    );
    // Type 2 closes on the first line containing `-->`; the block is that
    // line alone, and the following non-blank line starts a new block.
    assert_eq!(first_html_value(&doc), "<!-- note --> tail\n");
}

#[test]
fn html_comment_spans_lines_until_terminator_line() {
    let allocator = Allocator::new();
    let doc = parse_with_options(
        &allocator,
        "<!--\nline one\nline two --> tail\nAfter paragraph? no: same line ends block",
        ParserOptions::default(),
    );
    assert_eq!(first_html_value(&doc), "<!--\nline one\nline two --> tail\n");
}

#[test]
fn html_unclosed_comment_runs_to_eof() {
    let allocator = Allocator::new();
    let doc = parse_with_options(
        &allocator,
        "<!--\nnever closed\n\nstill inside",
        ParserOptions::default(),
    );
    assert_eq!(first_html_value(&doc), "<!--\nnever closed\n\nstill inside");
    assert_eq!(doc.children.len(), 1);
}

#[test]
fn html_type1_script_closes_case_insensitively() {
    let allocator = Allocator::new();
    let doc = parse_with_options(
        &allocator,
        "<script>\nlet a = 1 < 2;\n</SCRIPT> tail\n\nAfter",
        ParserOptions::default(),
    );
    assert_eq!(first_html_value(&doc), "<script>\nlet a = 1 < 2;\n</SCRIPT> tail\n");
}

#[test]
fn html_type1_close_on_opener_line_ends_the_block() {
    let allocator = Allocator::new();
    let doc = parse_with_options(
        &allocator,
        "<pre>one liner</pre>\nnext line\n\nAfter",
        ParserOptions::default(),
    );
    // The closing tag on the opening line ends the block with that line;
    // `next line` is paragraph continuation-free new content.
    assert_eq!(first_html_value(&doc), "<pre>one liner</pre>\n");
}

#[test]
fn html_type1_closing_tag_split_across_lines_does_not_close() {
    let allocator = Allocator::new();
    let doc =
        parse_with_options(&allocator, "<script>\nx = '</scr\nipt>';\n", ParserOptions::default());
    assert_eq!(doc.children.len(), 1);
    assert!(matches!(&doc.children[0], Node::Html(_)));
}

#[test]
fn html_cdata_and_processing_instruction_terminators() {
    let allocator = Allocator::new();
    let doc = parse_with_options(
        &allocator,
        "<![CDATA[\ndata ]]> tail\n\n<?php\necho 1; ?> tail\n\n<!DOCTYPE html>\n\nAfter",
        ParserOptions::default(),
    );
    assert_eq!(first_html_value(&doc), "<![CDATA[\ndata ]]> tail\n");
    assert!(
        matches!(&doc.children[1], Node::Html(html) if html.value == "<?php\necho 1; ?> tail\n")
    );
    assert!(matches!(&doc.children[2], Node::Html(html) if html.value == "<!DOCTYPE html>\n"));
}

#[test]
fn html_type6_whitespace_only_line_terminates_and_stays_unconsumed() {
    let allocator = Allocator::new();
    let doc =
        parse_with_options(&allocator, "<div>\ncontent\n \t\nAfter", ParserOptions::default());
    // The whitespace-only line is blank for termination purposes and is
    // not part of the block's raw value.
    assert_eq!(first_html_value(&doc), "<div>\ncontent\n");
    assert!(matches!(&doc.children[1], Node::Paragraph(_)));
}

#[test]
fn html_type6_trailing_whitespace_only_remainder_stays_unconsumed() {
    let allocator = Allocator::new();
    let doc = parse_with_options(&allocator, "<div>\ncontent\n   ", ParserOptions::default());
    assert_eq!(first_html_value(&doc), "<div>\ncontent\n");
}
