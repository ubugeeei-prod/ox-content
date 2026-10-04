use ox_content_allocator::Allocator;
use ox_content_ast::{ListItem, Node, Span, TableCell, TableRow};
use ox_content_parser::ParserOptions;

use super::parse_with_options;

#[test]
fn block_quote_child_spans_index_the_document_source() {
    let allocator = Allocator::new();
    let source = "para\n\n> quote";
    let doc = parse_with_options(&allocator, source, ParserOptions::gfm());

    let Node::BlockQuote(block_quote) = &doc.children[1] else {
        panic!("expected block quote, got {:?}", doc.children[1]);
    };
    let Node::Paragraph(paragraph) = &block_quote.children[0] else {
        panic!("expected quoted paragraph, got {:?}", block_quote.children[0]);
    };

    assert_eq!(paragraph.span, Span::new(8, 13));
    assert_eq!(paragraph.span.source_text(source), "quote");
    assert_all_spans_index_source(source, &doc.children);
}

#[test]
fn block_quote_tab_and_unicode_spans_stay_in_bounds() {
    for source in [">\t\tfoo\n", "> 日本語の段落\n"] {
        let allocator = Allocator::new();
        let doc = parse_with_options(&allocator, source, ParserOptions::gfm());
        assert_all_spans_index_source(source, &doc.children);
    }
}

#[test]
fn list_continuation_child_spans_account_for_stripped_indent() {
    let allocator = Allocator::new();
    let source = "- item\n  ```rust\n  code\n  ```\n";
    let doc = parse_with_options(&allocator, source, ParserOptions::gfm());

    let Node::List(list) = &doc.children[0] else {
        panic!("expected list, got {:?}", doc.children[0]);
    };
    let Node::CodeBlock(code) = &list.children[0].children[1] else {
        panic!("expected code block, got {:?}", list.children[0].children[1]);
    };

    assert_eq!(code.span, Span::new(7, 30));
    assert_eq!(code.span.source_text(source), "  ```rust\n  code\n  ```\n");
    assert_all_spans_index_source(source, &doc.children);
}

#[test]
fn nested_list_child_spans_account_for_marker_indent() {
    let allocator = Allocator::new();
    let source = "- item\n  - nested\n";
    let doc = parse_with_options(&allocator, source, ParserOptions::gfm());

    let Node::List(list) = &doc.children[0] else {
        panic!("expected list, got {:?}", doc.children[0]);
    };
    let Node::List(nested) = &list.children[0].children[1] else {
        panic!("expected nested list, got {:?}", list.children[0].children[1]);
    };
    let Node::Paragraph(paragraph) = &nested.children[0].children[0] else {
        panic!("expected paragraph, got {:?}", nested.children[0].children[0]);
    };

    assert_eq!(nested.span, Span::new(7, 18));
    assert_eq!(paragraph.span, Span::new(11, 18));
    assert_eq!(paragraph.span.source_text(source), "nested\n");
    assert_all_spans_index_source(source, &doc.children);
}

#[test]
fn footnote_definition_child_spans_index_the_document_source() {
    let allocator = Allocator::new();
    let source = "Ref[^a]\n\n[^a]: first\n\n    second\n\n    third\n";
    let options = ParserOptions { footnotes: true, ..ParserOptions::gfm() };
    let doc = parse_with_options(&allocator, source, options);

    let Node::FootnoteDefinition(definition) = &doc.children[1] else {
        panic!("expected footnote definition, got {:?}", doc.children[1]);
    };

    for (index, expected_text, expected_span) in [
        (0, "first", Span::new(15, 20)),
        (1, "second", Span::new(26, 32)),
        (2, "third", Span::new(38, 43)),
    ] {
        let Node::Paragraph(paragraph) = &definition.children[index] else {
            panic!("expected footnote paragraph, got {:?}", definition.children[index]);
        };
        let Node::Text(text) = &paragraph.children[0] else {
            panic!("expected paragraph text, got {:?}", paragraph.children[0]);
        };
        assert_eq!(text.span, expected_span);
        assert_eq!(text.span.source_text(source), expected_text);
    }

    assert_all_spans_index_source(source, &doc.children);
}

#[test]
fn table_rows_cells_and_inline_children_index_source() {
    let allocator = Allocator::new();
    let source = "| a | b |\n| - | - |\n| c | d |\n";
    let doc = parse_with_options(&allocator, source, ParserOptions::gfm());

    let Node::Table(table) = &doc.children[0] else {
        panic!("expected table, got {:?}", doc.children[0]);
    };

    assert_eq!(table.children[0].span, Span::new(0, 9));
    assert_eq!(table.children[0].children[0].span, Span::new(2, 3));
    assert_eq!(table.children[0].children[1].span, Span::new(6, 7));
    assert_eq!(table.children[1].span, Span::new(20, 29));
    assert_eq!(table.children[1].children[0].span, Span::new(22, 23));
    assert_eq!(table.children[1].children[0].children[0].span(), Span::new(22, 23));
    assert_all_spans_index_source(source, &doc.children);
}

#[test]
fn table_inline_strong_spans_after_escaped_pipes_index_document_source() {
    for (source, text_value, text_source, cell_source) in [
        ("| a **bold** |\n| --- |\n", "a ", "a ", "a **bold**"),
        ("| a\\|b **bold** |\n| --- |\n", "a|b ", "a\\|b ", "a\\|b **bold**"),
        ("| a\\|b\\|c **bold** |\n| --- |\n", "a|b|c ", "a\\|b\\|c ", "a\\|b\\|c **bold**"),
    ] {
        let allocator = Allocator::new();
        let doc = parse_with_options(&allocator, source, ParserOptions::gfm());

        let Node::Table(table) = &doc.children[0] else {
            panic!("expected table, got {:?}", doc.children[0]);
        };
        let cell = &table.children[0].children[0];
        let cell_start = source.find(cell_source).expect("cell source") as u32;
        assert_eq!(cell.span, Span::new(cell_start, cell_start + cell_source.len() as u32));
        assert_eq!(cell.span.source_text(source), cell_source);

        let Node::Text(text) = &cell.children[0] else {
            panic!("expected leading text, got {:?}", cell.children[0]);
        };
        let text_start = source.find(text_source).expect("text source") as u32;
        assert_eq!(text.value, text_value);
        assert_eq!(text.span, Span::new(text_start, text_start + text_source.len() as u32));
        assert_eq!(text.span.source_text(source), text_source);

        let Node::Strong(strong) = &cell.children[1] else {
            panic!("expected strong, got {:?}", cell.children[1]);
        };
        let strong_start = source.find("**bold**").expect("strong source") as u32;
        assert_eq!(strong.span, Span::new(strong_start, strong_start + 8));
        assert_eq!(strong.span.source_text(source), "**bold**");
        assert_all_spans_index_source(source, &doc.children);
    }
}

#[test]
fn table_wiki_link_span_after_escaped_pipes_indexes_document_source() {
    let allocator = Allocator::new();
    let source = "| a\\|b\\|c [[Guide\\|Label]] |\n| --- |\n";
    let options = ParserOptions { wiki_links: true, ..ParserOptions::gfm() };
    let doc = parse_with_options(&allocator, source, options);

    let Node::Table(table) = &doc.children[0] else {
        panic!("expected table, got {:?}", doc.children[0]);
    };
    let cell = &table.children[0].children[0];
    let Node::Link(link) = &cell.children[1] else {
        panic!("expected wiki link, got {:?}", cell.children[1]);
    };

    assert_eq!(link.url, "Guide");
    let link_source = "[[Guide\\|Label]]";
    let link_start = source.find(link_source).expect("link source") as u32;
    assert_eq!(link.span, Span::new(link_start, link_start + link_source.len() as u32));
    assert_eq!(link.span.source_text(source), link_source);

    let Node::Text(label) = &link.children[0] else {
        panic!("expected wiki link label, got {:?}", link.children[0]);
    };
    assert_eq!(label.value, "Label");
    assert_eq!(label.span.source_text(source), "Label");
    assert_all_spans_index_source(source, &doc.children);
}

fn assert_all_spans_index_source(source: &str, nodes: &[Node<'_>]) {
    for node in nodes {
        assert_node_span_indexes_source(source, node);
    }
}

fn assert_node_span_indexes_source(source: &str, node: &Node<'_>) {
    assert_span_indexes_source(source, node.span());
    match node {
        Node::Paragraph(node) => assert_all_spans_index_source(source, &node.children),
        Node::Heading(node) => assert_all_spans_index_source(source, &node.children),
        Node::BlockQuote(node) => assert_all_spans_index_source(source, &node.children),
        Node::List(node) => {
            for item in &node.children {
                assert_list_item_span_indexes_source(source, item);
            }
        }
        Node::ListItem(item) => assert_list_item_span_indexes_source(source, item),
        Node::DefinitionList(node) => assert_all_spans_index_source(source, &node.children),
        Node::DefinitionListTerm(node) => assert_all_spans_index_source(source, &node.children),
        Node::DefinitionListDefinition(node) => {
            assert_all_spans_index_source(source, &node.children);
        }
        Node::Table(table) => {
            for row in &table.children {
                assert_table_row_span_indexes_source(source, row);
            }
        }
        Node::Emphasis(node) => assert_all_spans_index_source(source, &node.children),
        Node::Strong(node) => assert_all_spans_index_source(source, &node.children),
        Node::Link(node) => assert_all_spans_index_source(source, &node.children),
        Node::Delete(node) => assert_all_spans_index_source(source, &node.children),
        Node::Superscript(node) => assert_all_spans_index_source(source, &node.children),
        Node::Subscript(node) => assert_all_spans_index_source(source, &node.children),
        Node::FootnoteDefinition(node) => assert_all_spans_index_source(source, &node.children),
        Node::MdxJsxFlowElement(node) => assert_all_spans_index_source(source, &node.children),
        Node::MdxJsxTextElement(node) => assert_all_spans_index_source(source, &node.children),
        Node::ThematicBreak(_)
        | Node::CodeBlock(_)
        | Node::MathBlock(_)
        | Node::Html(_)
        | Node::Text(_)
        | Node::InlineCode(_)
        | Node::InlineMath(_)
        | Node::Break(_)
        | Node::Image(_)
        | Node::FootnoteReference(_)
        | Node::Definition(_)
        | Node::MdxjsEsm(_)
        | Node::MdxFlowExpression(_)
        | Node::MdxTextExpression(_) => {}
    }
}

fn assert_list_item_span_indexes_source(source: &str, item: &ListItem<'_>) {
    assert_span_indexes_source(source, item.span);
    assert_all_spans_index_source(source, &item.children);
}

fn assert_table_row_span_indexes_source(source: &str, row: &TableRow<'_>) {
    assert_span_indexes_source(source, row.span);
    for cell in &row.children {
        assert_table_cell_span_indexes_source(source, cell);
    }
}

fn assert_table_cell_span_indexes_source(source: &str, cell: &TableCell<'_>) {
    assert_span_indexes_source(source, cell.span);
    assert_all_spans_index_source(source, &cell.children);
}

fn assert_span_indexes_source(source: &str, span: Span) {
    assert!(
        source.get(span.start as usize..span.end as usize).is_some(),
        "invalid span {span:?} for {source:?}"
    );
}

#[test]
fn trimmed_prose_spans_point_to_visible_text() {
    struct TextSpans<'s> {
        source: &'s str,
        count: usize,
    }
    impl<'a> ox_content_ast::Visit<'a> for TextSpans<'_> {
        fn visit_text(&mut self, node: &ox_content_ast::Text<'a>) {
            assert_eq!(node.span.source_text(self.source), node.value);
            self.count += 1;
        }
    }
    for source in [
        "  visible visible\n",
        "  Heading\n  =======\n",
        "- Item\n\n    visible visible\n",
        "-   visible\n",
        ">   visible\n",
    ] {
        let allocator = Allocator::new();
        let doc = parse_with_options(&allocator, source, ParserOptions::gfm());
        let mut visitor = TextSpans { source, count: 0 };
        ox_content_ast::Visit::visit_document(&mut visitor, &doc);
        assert!(visitor.count > 0);
    }
}
