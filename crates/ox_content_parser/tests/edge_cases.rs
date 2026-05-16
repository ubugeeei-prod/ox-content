use ox_content_allocator::Allocator;
use ox_content_ast::{AlignKind, Node};
use ox_content_parser::{Parser, ParserOptions};

fn parse_with_options<'a>(
    allocator: &'a Allocator,
    source: &'a str,
    options: ParserOptions,
) -> ox_content_ast::Document<'a> {
    Parser::with_options(allocator, source, options).parse().unwrap()
}

fn first_text<'a>(node: &'a Node<'a>) -> Option<&'a str> {
    match node {
        Node::Text(text) => Some(text.value),
        Node::Paragraph(paragraph) => paragraph.children.iter().find_map(first_text),
        Node::Heading(heading) => heading.children.iter().find_map(first_text),
        Node::Emphasis(emphasis) => emphasis.children.iter().find_map(first_text),
        Node::Strong(strong) => strong.children.iter().find_map(first_text),
        Node::Delete(delete) => delete.children.iter().find_map(first_text),
        Node::Link(link) => link.children.iter().find_map(first_text),
        Node::List(list) => {
            list.children.iter().flat_map(|item| item.children.iter()).find_map(first_text)
        }
        Node::ListItem(item) => item.children.iter().find_map(first_text),
        _ => None,
    }
}

fn first_text_in_nodes<'a>(nodes: impl IntoIterator<Item = &'a Node<'a>>) -> Option<&'a str> {
    nodes.into_iter().find_map(first_text)
}

fn flatten_text(node: &Node<'_>) -> String {
    match node {
        Node::Text(text) => text.value.to_string(),
        Node::Paragraph(paragraph) => paragraph.children.iter().map(flatten_text).collect(),
        Node::Heading(heading) => heading.children.iter().map(flatten_text).collect(),
        Node::Emphasis(emphasis) => emphasis.children.iter().map(flatten_text).collect(),
        Node::Strong(strong) => strong.children.iter().map(flatten_text).collect(),
        Node::Delete(delete) => delete.children.iter().map(flatten_text).collect(),
        Node::Link(link) => link.children.iter().map(flatten_text).collect(),
        Node::List(list) => {
            list.children.iter().flat_map(|item| item.children.iter()).map(flatten_text).collect()
        }
        _ => String::new(),
    }
}

#[test]
fn blank_input_yields_empty_document() {
    let allocator = Allocator::new();
    let doc = parse_with_options(&allocator, "\n \n\t\n", ParserOptions::default());
    assert!(doc.children.is_empty());
}

#[test]
fn heading_trims_closing_hashes() {
    let allocator = Allocator::new();
    let doc = parse_with_options(&allocator, "## Title ###", ParserOptions::default());
    match &doc.children[0] {
        Node::Heading(heading) => {
            assert_eq!(heading.depth, 2);
            assert_eq!(first_text_in_nodes(heading.children.iter()), Some("Title"));
        }
        other => panic!("expected heading, got {other:?}"),
    }
}

#[test]
fn invalid_atx_heading_without_space_stays_paragraph() {
    let allocator = Allocator::new();
    let doc = parse_with_options(&allocator, "#Title", ParserOptions::default());
    assert!(matches!(&doc.children[0], Node::Paragraph(_)));
}

#[test]
fn too_many_hashes_stays_paragraph() {
    let allocator = Allocator::new();
    let doc = parse_with_options(&allocator, "####### nope", ParserOptions::default());
    assert!(matches!(&doc.children[0], Node::Paragraph(_)));
}

#[test]
fn thematic_break_accepts_spaces() {
    let allocator = Allocator::new();
    let doc = parse_with_options(&allocator, " * * * ", ParserOptions::default());
    assert!(matches!(&doc.children[0], Node::ThematicBreak(_)));
}

#[test]
fn mixed_thematic_break_is_not_recognized() {
    let allocator = Allocator::new();
    let doc = parse_with_options(&allocator, "- * -", ParserOptions::default());
    assert!(!matches!(&doc.children[0], Node::ThematicBreak(_)));
}

#[test]
fn fenced_code_supports_tildes_and_meta() {
    let allocator = Allocator::new();
    let doc = parse_with_options(
        &allocator,
        "~~~ts filename=main.ts\nconsole.log(1)\n~~~",
        ParserOptions::default(),
    );

    match &doc.children[0] {
        Node::CodeBlock(block) => {
            assert_eq!(block.lang, Some("ts"));
            assert_eq!(block.meta, Some("filename=main.ts"));
            assert_eq!(block.value, "console.log(1)\n");
        }
        other => panic!("expected code block, got {other:?}"),
    }
}

#[test]
fn unclosed_fence_consumes_until_eof() {
    let allocator = Allocator::new();
    let doc = parse_with_options(&allocator, "```rs\nfn main() {}\n", ParserOptions::default());

    match &doc.children[0] {
        Node::CodeBlock(block) => assert_eq!(block.value, "fn main() {}\n"),
        other => panic!("expected code block, got {other:?}"),
    }
}

#[test]
fn ordered_list_start_and_parenthesis_marker_are_preserved() {
    let allocator = Allocator::new();
    let doc = parse_with_options(&allocator, "3) third\n4) fourth", ParserOptions::default());

    match &doc.children[0] {
        Node::List(list) => {
            assert!(list.ordered);
            assert_eq!(list.start, Some(3));
            assert_eq!(list.children.len(), 2);
        }
        other => panic!("expected list, got {other:?}"),
    }
}

#[test]
fn task_list_marker_is_literal_when_option_is_disabled() {
    let allocator = Allocator::new();
    let doc = parse_with_options(&allocator, "- [x] done", ParserOptions::default());

    match &doc.children[0] {
        Node::List(list) => {
            assert_eq!(list.children[0].checked, None);
            assert_eq!(flatten_text(&list.children[0].children[0]), "[x] done");
        }
        other => panic!("expected list, got {other:?}"),
    }
}

#[test]
fn task_list_marker_sets_checked_when_enabled() {
    let allocator = Allocator::new();
    let doc = parse_with_options(&allocator, "- [ ] todo\n- [x] done", ParserOptions::gfm());

    match &doc.children[0] {
        Node::List(list) => {
            assert_eq!(list.children[0].checked, Some(false));
            assert_eq!(list.children[1].checked, Some(true));
        }
        other => panic!("expected list, got {other:?}"),
    }
}

#[test]
fn nested_list_is_attached_to_previous_item() {
    let allocator = Allocator::new();
    let doc =
        parse_with_options(&allocator, "- parent\n  - child\n- sibling", ParserOptions::default());

    match &doc.children[0] {
        Node::List(list) => {
            assert_eq!(list.children.len(), 2);
            assert!(list.children[0].children.iter().any(|node| matches!(node, Node::List(_))));
            assert!(list.children[0].span.end >= list.children[0].children[1].span().end);
        }
        other => panic!("expected list, got {other:?}"),
    }
}

#[test]
fn blockquote_supports_multiple_paragraphs_when_blank_quote_line_is_used() {
    let allocator = Allocator::new();
    let doc = parse_with_options(&allocator, "> first\n>\n> second", ParserOptions::default());

    match &doc.children[0] {
        Node::BlockQuote(block_quote) => {
            assert_eq!(block_quote.children.len(), 2);
            assert!(matches!(&block_quote.children[0], Node::Paragraph(_)));
            assert!(matches!(&block_quote.children[1], Node::Paragraph(_)));
        }
        other => panic!("expected blockquote, got {other:?}"),
    }
}

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
            assert!(html.value.contains("<details id=\"symbol\">"));
            assert!(html.value.contains("</details>"));
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
    assert!(matches!(&doc.children[1], Node::Html(html) if html.value.contains("<summary>")));
    assert!(
        matches!(&doc.children[2], Node::Paragraph(paragraph) if matches!(&paragraph.children[0], Node::Strong(_)))
    );
    assert!(matches!(&doc.children[3], Node::List(_)));
    assert!(matches!(&doc.children[4], Node::CodeBlock(block) if block.lang == Some("js")));
    assert!(matches!(&doc.children[5], Node::Html(html) if html.value.contains("</details>")));
}

#[test]
fn html_type6_div_stops_before_markdown_after_blank_line() {
    let allocator = Allocator::new();
    let doc = parse_with_options(
        &allocator,
        "<div>\nraw html line\n\n**markdown**\n\n</div>",
        ParserOptions::default(),
    );

    assert!(matches!(&doc.children[0], Node::Html(html) if !html.value.contains("**markdown**")));
    assert!(
        matches!(&doc.children[1], Node::Paragraph(paragraph) if matches!(&paragraph.children[0], Node::Strong(_)))
    );
    assert!(matches!(&doc.children[2], Node::Html(html) if html.value.contains("</div>")));
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
            assert!(html.value.contains("**not markdown**"));
            assert!(html.value.contains("</pre>"));
        }
        other => panic!("expected html block, got {other:?}"),
    }
    assert!(matches!(&doc.children[1], Node::Paragraph(_)));
}

#[test]
fn table_alignment_variants_are_parsed() {
    let allocator = Allocator::new();
    let doc = parse_with_options(
        &allocator,
        "| a | b | c |\n| :-- | :-: | --: |\n| 1 | 2 | 3 |",
        ParserOptions::gfm(),
    );

    match &doc.children[0] {
        Node::Table(table) => {
            assert_eq!(table.align.len(), 3);
            assert_eq!(table.align[0], AlignKind::Left);
            assert_eq!(table.align[1], AlignKind::Center);
            assert_eq!(table.align[2], AlignKind::Right);
        }
        other => panic!("expected table, got {other:?}"),
    }
}

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

#[test]
fn list_item_paragraph_span_tracks_inline_content() {
    let allocator = Allocator::new();
    let source = "- alpha";
    let doc = parse_with_options(&allocator, source, ParserOptions::default());

    match &doc.children[0] {
        Node::List(list) => match &list.children[0].children[0] {
            Node::Paragraph(paragraph) => {
                assert_eq!(paragraph.span.start as usize, 2);
                assert_eq!(paragraph.span.end as usize, source.len());
            }
            other => panic!("expected paragraph, got {other:?}"),
        },
        other => panic!("expected list, got {other:?}"),
    }
}
