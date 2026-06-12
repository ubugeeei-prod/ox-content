use ox_content_allocator::Allocator;
use ox_content_ast::Node;
use ox_content_parser::ParserOptions;

use super::{flatten_text, parse_with_options};

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
