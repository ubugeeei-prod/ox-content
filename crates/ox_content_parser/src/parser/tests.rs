use super::*;
use ox_content_ast::Node;

#[test]
fn ascii_contains_closing_tag_matches_case_insensitively() {
    assert!(super::html::ascii_contains_closing_tag("end </SCRIPT> tail", b"script"));
    assert!(super::html::ascii_contains_closing_tag("</style ", b"style"));
    assert!(!super::html::ascii_contains_closing_tag("<scriptsource>", b"script"));
    assert!(!super::html::ascii_contains_closing_tag("", b"pre"));
    assert!(!super::html::ascii_contains_closing_tag("</pr", b"pre"));
}

#[test]
fn test_parse_image() {
    let allocator = Allocator::new();
    let doc = Parser::new(&allocator, "![Alt text](/path/to/image.png)").parse().unwrap();
    assert_eq!(doc.children.len(), 1);
    match &doc.children[0] {
        Node::Paragraph(p) => {
            assert_eq!(p.children.len(), 1);
            match &p.children[0] {
                Node::Image(img) => {
                    assert_eq!(img.alt, "Alt text");
                    assert_eq!(img.url, "/path/to/image.png");
                }
                _ => panic!("expected image, got {:?}", p.children[0]),
            }
        }
        _ => panic!("expected paragraph"),
    }
}

#[test]
fn test_parse_heading() {
    let allocator = Allocator::new();
    // Use "# " with trailing space - our parser requires space/tab/newline after #
    let doc = Parser::new(&allocator, "# Hello\n").parse().unwrap();
    assert_eq!(doc.children.len(), 1);
    match &doc.children[0] {
        Node::Heading(h) => {
            assert_eq!(h.depth, 1);
        }
        _ => panic!("expected heading"),
    }
}

#[test]
fn indented_heading_like_text_does_not_loop() {
    // Regression: `line_starts_block` once tested ATX headings against
    // the trimmed bytes while `parse_block` tested at the un-trimmed
    // position, so " # heading" caused `parse_paragraph` to break
    // immediately ("looks like a heading") and `parse_block` to return
    // `Ok(None)` without advancing — spinning the outer loop forever.
    // The fix is for `line_starts_block` and `parse_block` to share the same
    // untrimmed-line-start heading check.
    let allocator = Allocator::new();
    let doc = Parser::new(&allocator, " # heading\n").parse().unwrap();
    assert_eq!(doc.children.len(), 1);
    assert!(
        matches!(&doc.children[0], Node::Paragraph(_)),
        "expected leading-indented `#` to parse as paragraph text, got {:?}",
        doc.children[0]
    );
}

#[test]
fn test_parse_paragraph() {
    let allocator = Allocator::new();
    let doc = Parser::new(&allocator, "Hello world").parse().unwrap();
    assert_eq!(doc.children.len(), 1);
    assert!(matches!(&doc.children[0], Node::Paragraph(_)));
}

#[test]
fn test_parse_thematic_break() {
    let allocator = Allocator::new();
    let doc = Parser::new(&allocator, "---").parse().unwrap();
    assert_eq!(doc.children.len(), 1);
    assert!(matches!(&doc.children[0], Node::ThematicBreak(_)));
}

#[test]
fn test_parse_fenced_code() {
    let allocator = Allocator::new();
    let doc = Parser::new(
        &allocator,
        "```rust\nfn main() {}
```",
    )
    .parse()
    .unwrap();
    assert_eq!(doc.children.len(), 1);
    match &doc.children[0] {
        Node::CodeBlock(cb) => {
            assert_eq!(cb.lang, Some("rust"));
        }
        _ => panic!("expected code block"),
    }
}

#[test]
fn test_parse_inline_code() {
    let allocator = Allocator::new();
    let doc = Parser::new(&allocator, "Use `code` here").parse().unwrap();
    assert_eq!(doc.children.len(), 1);
    match &doc.children[0] {
        Node::Paragraph(p) => {
            assert!(p.children.iter().any(|n| matches!(n, Node::InlineCode(_))));
        }
        _ => panic!("expected paragraph"),
    }
}

#[test]
fn test_parse_strikethrough() {
    let allocator = Allocator::new();
    let doc = Parser::with_options(&allocator, "~~done~~", ParserOptions::gfm()).parse().unwrap();
    assert_eq!(doc.children.len(), 1);
    match &doc.children[0] {
        Node::Paragraph(p) => {
            assert!(matches!(&p.children[0], Node::Delete(_)));
        }
        _ => panic!("expected paragraph"),
    }
}

#[test]
fn test_parse_strikethrough_lone_tilde_not_matched() {
    // A trailing lone `~` (and a single `~` mid-text) must not be treated as a
    // closing run: the `~~` opener falls back to literal text. This pins the
    // `inner_end + 1 < len` boundary preserved by the memchr-based scan.
    let allocator = Allocator::new();
    for input in ["~~open ~ but no close", "~~trailing tilde~"] {
        let doc = Parser::with_options(&allocator, input, ParserOptions::gfm()).parse().unwrap();
        match &doc.children[0] {
            Node::Paragraph(p) => {
                assert!(
                    !p.children.iter().any(|n| matches!(n, Node::Delete(_))),
                    "{input:?} should not produce a Delete node"
                );
            }
            _ => panic!("expected paragraph"),
        }
    }
}

#[test]
fn test_parse_hard_break() {
    let allocator = Allocator::new();
    let doc = Parser::new(&allocator, "line 1\\\nline 2").parse().unwrap();
    assert_eq!(doc.children.len(), 1);
    match &doc.children[0] {
        Node::Paragraph(p) => {
            assert!(p.children.iter().any(|n| matches!(n, Node::Break(_))));
        }
        _ => panic!("expected paragraph"),
    }
}

#[test]
fn test_parse_table() {
    let allocator = Allocator::new();
    let table_md = "| Header 1 | Header 2 |\n|----------|----------|\n| Cell 1   | Cell 2   |";
    let parser = Parser::with_options(&allocator, table_md, ParserOptions::gfm());
    let doc = parser.parse().unwrap();
    assert_eq!(doc.children.len(), 1);
    match &doc.children[0] {
        Node::Table(t) => {
            assert_eq!(t.children.len(), 2); // header + 1 body row
        }
        _ => panic!("expected table, got {:?}", doc.children[0]),
    }
}

#[test]
fn test_parse_table_preserves_escaped_pipes() {
    let allocator = Allocator::new();
    let table_md = "| Description |\n| --- |\n| Disallow filters (the `\\|` pipe) |";
    let parser = Parser::with_options(&allocator, table_md, ParserOptions::gfm());
    let doc = parser.parse().unwrap();

    let Node::Table(table) = &doc.children[0] else {
        panic!("expected table, got {:?}", doc.children[0]);
    };
    assert_eq!(table.children[1].children.len(), 1);
    let inline_code = table.children[1].children[0].children.iter().find_map(|node| match node {
        Node::InlineCode(code) => Some(code.value),
        _ => None,
    });
    assert_eq!(inline_code, Some("|"));
}

#[test]
fn test_parse_unordered_list() {
    let allocator = Allocator::new();
    let list_md = "- Item 1\n- Item 2\n- Item 3";
    let parser = Parser::new(&allocator, list_md);
    let doc = parser.parse().unwrap();
    assert_eq!(doc.children.len(), 1);
    match &doc.children[0] {
        Node::List(list) => {
            assert!(!list.ordered);
            assert_eq!(list.children.len(), 3);
        }
        _ => panic!("expected list, got {:?}", doc.children[0]),
    }
}

#[test]
fn test_parse_ordered_list() {
    let allocator = Allocator::new();
    let list_md = "1. First\n2. Second\n3. Third";
    let parser = Parser::new(&allocator, list_md);
    let doc = parser.parse().unwrap();
    assert_eq!(doc.children.len(), 1);
    match &doc.children[0] {
        Node::List(list) => {
            assert!(list.ordered);
            assert_eq!(list.children.len(), 3);
        }
        _ => panic!("expected list, got {:?}", doc.children[0]),
    }
}

#[test]
fn test_parse_block_quote() {
    let allocator = Allocator::new();
    let doc = Parser::new(&allocator, "> Hello world").parse().unwrap();
    assert_eq!(doc.children.len(), 1);
    match &doc.children[0] {
        Node::BlockQuote(bq) => {
            assert_eq!(bq.children.len(), 1);
            assert!(matches!(&bq.children[0], Node::Paragraph(_)));
        }
        _ => panic!("expected block quote, got {:?}", doc.children[0]),
    }
}

#[test]
fn test_parse_block_quote_multiline() {
    let allocator = Allocator::new();
    let doc = Parser::new(&allocator, "> line 1\n> line 2").parse().unwrap();
    assert_eq!(doc.children.len(), 1);
    match &doc.children[0] {
        Node::BlockQuote(bq) => {
            assert_eq!(bq.children.len(), 1);
        }
        _ => panic!("expected block quote, got {:?}", doc.children[0]),
    }
}

#[test]
fn test_parse_nested_block_quote() {
    let allocator = Allocator::new();
    let doc = Parser::new(&allocator, "> > nested").parse().unwrap();
    assert_eq!(doc.children.len(), 1);
    match &doc.children[0] {
        Node::BlockQuote(bq) => {
            assert_eq!(bq.children.len(), 1);
            assert!(matches!(&bq.children[0], Node::BlockQuote(_)));
        }
        _ => panic!("expected block quote, got {:?}", doc.children[0]),
    }
}
