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
