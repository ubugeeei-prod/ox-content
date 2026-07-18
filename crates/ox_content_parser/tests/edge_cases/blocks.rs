use ox_content_allocator::Allocator;
use ox_content_ast::Node;
use ox_content_parser::ParserOptions;

use super::{first_text_in_nodes, parse_with_options};

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
fn indented_fence_inside_list_item_stays_nested_block() {
    let allocator = Allocator::new();
    let doc = parse_with_options(
        &allocator,
        "1. text\n\n   ```ts\n   const a = 1;\n   ```",
        ParserOptions::default(),
    );

    match &doc.children[0] {
        Node::List(list) => {
            assert_eq!(list.children.len(), 1);
            assert_eq!(list.children[0].children.len(), 2);
            assert!(matches!(&list.children[0].children[0], Node::Paragraph(_)));
            assert!(
                matches!(&list.children[0].children[1], Node::CodeBlock(block) if block.lang == Some("ts") && block.value == "const a = 1;\n")
            );
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
fn whitespace_only_document_without_trailing_newline_terminates() {
    // Regression: a document holding only spaces/tabs and no final
    // newline left the block loop parked on whitespace that no block
    // parser consumed, spinning forever. Every variant must parse to an
    // empty document.
    for source in ["  ", "\t", "  \t", "   \t", "    \t", " \t \t"] {
        let allocator = Allocator::new();
        let doc = parse_with_options(&allocator, source, ParserOptions::default());
        assert!(doc.children.is_empty(), "expected no blocks for {source:?}");
    }
}

#[test]
fn trailing_whitespace_line_does_not_swallow_preceding_content() {
    let allocator = Allocator::new();
    let doc = parse_with_options(&allocator, "text\n  ", ParserOptions::default());

    assert_eq!(doc.children.len(), 1);
    assert_eq!(first_text_in_nodes(&doc.children), Some("text"));
}

#[test]
fn indented_content_at_end_of_input_still_parses() {
    // The blank-line skipper must only consume a trailing whitespace run
    // when nothing follows it; indentation before real content is
    // meaningful.
    let allocator = Allocator::new();
    let doc = parse_with_options(&allocator, "    code", ParserOptions::default());

    assert_eq!(doc.children.len(), 1);
    assert!(
        matches!(&doc.children[0], Node::CodeBlock(_)),
        "expected indented code, got {:?}",
        doc.children[0]
    );
}
