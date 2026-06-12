use ox_content_allocator::Allocator;
use ox_content_ast::Node;
use ox_content_parser::ParserOptions;

use super::{first_text, parse_with_options};

#[test]
fn inline_link_handles_nested_parentheses() {
    let allocator = Allocator::new();
    let doc = parse_with_options(
        &allocator,
        "[docs](https://example.com/a(b)c)",
        ParserOptions::default(),
    );

    match &doc.children[0] {
        Node::Paragraph(paragraph) => match &paragraph.children[0] {
            Node::Link(link) => assert_eq!(link.url, "https://example.com/a(b)c"),
            other => panic!("expected link, got {other:?}"),
        },
        other => panic!("expected paragraph, got {other:?}"),
    }
}

#[test]
fn inline_raw_html_is_preserved_as_html_node() {
    let allocator = Allocator::new();
    let doc = parse_with_options(
        &allocator,
        "before <input type=\"checkbox\"> after",
        ParserOptions::default(),
    );

    match &doc.children[0] {
        Node::Paragraph(paragraph) => {
            assert!(
                matches!(&paragraph.children[1], Node::Html(html) if html.value == "<input type=\"checkbox\">")
            );
        }
        other => panic!("expected paragraph, got {other:?}"),
    }
}

#[test]
fn list_item_allows_inline_raw_html() {
    let allocator = Allocator::new();
    let doc = parse_with_options(
        &allocator,
        "- <input type=\"checkbox\"> task",
        ParserOptions::default(),
    );

    match &doc.children[0] {
        Node::List(list) => match &list.children[0].children[0] {
            Node::Paragraph(paragraph) => {
                assert!(
                    matches!(&paragraph.children[0], Node::Html(html) if html.value == "<input type=\"checkbox\">")
                );
            }
            other => panic!("expected paragraph, got {other:?}"),
        },
        other => panic!("expected list, got {other:?}"),
    }
}

#[test]
fn inline_code_keeps_raw_html_literal() {
    let allocator = Allocator::new();
    let doc = parse_with_options(&allocator, "`<input>`", ParserOptions::default());

    match &doc.children[0] {
        Node::Paragraph(paragraph) => {
            assert!(
                matches!(&paragraph.children[0], Node::InlineCode(code) if code.value == "<input>")
            );
        }
        other => panic!("expected paragraph, got {other:?}"),
    }
}

#[test]
fn image_url_handles_nested_parentheses() {
    let allocator = Allocator::new();
    let doc =
        parse_with_options(&allocator, "![diagram](./img(test).png)", ParserOptions::default());

    match &doc.children[0] {
        Node::Paragraph(paragraph) => match &paragraph.children[0] {
            Node::Image(image) => assert_eq!(image.url, "./img(test).png"),
            other => panic!("expected image, got {other:?}"),
        },
        other => panic!("expected paragraph, got {other:?}"),
    }
}

#[test]
fn escaped_marker_remains_literal_text() {
    let allocator = Allocator::new();
    let doc = parse_with_options(&allocator, "\\*literal\\*", ParserOptions::default());

    match &doc.children[0] {
        Node::Paragraph(paragraph) => {
            let text = paragraph
                .children
                .iter()
                .filter_map(first_text)
                .collect::<std::vec::Vec<_>>()
                .join("");
            assert_eq!(text, "*literal*");
        }
        other => panic!("expected paragraph, got {other:?}"),
    }
}

#[test]
fn unmatched_strikethrough_remains_text() {
    let allocator = Allocator::new();
    let doc = parse_with_options(&allocator, "~~open", ParserOptions::gfm());

    match &doc.children[0] {
        Node::Paragraph(paragraph) => {
            assert!(matches!(&paragraph.children[0], Node::Text(_)));
            assert_eq!(first_text(&paragraph.children[0]), Some("~~"));
        }
        other => panic!("expected paragraph, got {other:?}"),
    }
}

#[test]
fn hard_break_creates_break_node() {
    let allocator = Allocator::new();
    let doc = parse_with_options(&allocator, "line 1\\\nline 2", ParserOptions::default());

    match &doc.children[0] {
        Node::Paragraph(paragraph) => {
            assert!(paragraph.children.iter().any(|node| matches!(node, Node::Break(_))));
        }
        other => panic!("expected paragraph, got {other:?}"),
    }
}
