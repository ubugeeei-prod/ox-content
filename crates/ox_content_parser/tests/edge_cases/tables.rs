use ox_content_allocator::Allocator;
use ox_content_ast::{AlignKind, Node};
use ox_content_parser::ParserOptions;

use super::parse_with_options;

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
fn table_header_and_delimiter_must_have_matching_valid_cells() {
    let allocator = Allocator::new();
    for source in [
        "| a | b |\n| --- |",
        "| a | b |\n| --- | --- | --- |",
        "| a | b |\n| : | --- |",
        "| a | b |\n| --:-- | --- |",
    ] {
        let doc = parse_with_options(&allocator, source, ParserOptions::gfm());
        assert!(
            matches!(doc.children.first(), Some(Node::Paragraph(_))),
            "invalid delimiter must remain paragraph text: {source:?}"
        );
    }
}

#[test]
fn table_body_rows_are_normalized_to_header_width() {
    let allocator = Allocator::new();
    let doc = parse_with_options(
        &allocator,
        "| a | b |\n| --- | --- |\n| one |\n| two | three | ignored |",
        ParserOptions::gfm(),
    );

    let Node::Table(table) = &doc.children[0] else {
        panic!("expected table, got {:?}", doc.children[0]);
    };
    assert_eq!(table.children.len(), 3);
    assert!(table.children.iter().all(|row| row.children.len() == 2));
    assert!(table.children[1].children[1].children.is_empty());
}

#[test]
fn table_accepts_pipe_less_rows_and_stops_at_block_starts() {
    let allocator = Allocator::new();
    let doc = parse_with_options(
        &allocator,
        "| a | b |\n| --- | --- |\nplain\n> quote | value",
        ParserOptions::gfm(),
    );

    let Node::Table(table) = &doc.children[0] else {
        panic!("expected table, got {:?}", doc.children[0]);
    };
    assert_eq!(table.children.len(), 2);
    assert_eq!(table.children[1].children.len(), 2);
    assert!(matches!(doc.children.get(1), Some(Node::BlockQuote(_))));
}
