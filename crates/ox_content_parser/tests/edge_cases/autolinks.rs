//! GFM autolink extension edge cases.
//!
//! Split out of `inline.rs`: the autolink post-pass has its own boundary,
//! punctuation, and code-span rules, and they outgrew the file.

use ox_content_allocator::Allocator;
use ox_content_ast::Node;
use ox_content_parser::ParserOptions;

use super::parse_with_options;

#[test]
fn gfm_autolink_does_not_fire_inside_link_text() {
    let allocator = Allocator::new();
    let doc = parse_with_options(
        &allocator,
        "[visit www.example.com](https://real.example)",
        ParserOptions::gfm(),
    );

    match &doc.children[0] {
        Node::Paragraph(paragraph) => match &paragraph.children[0] {
            Node::Link(link) => {
                assert_eq!(link.url, "https://real.example");
                // GFM excludes link text from the autolink extension; a
                // nested Link here would render as invalid nested <a> tags.
                assert!(
                    link.children.iter().all(|child| matches!(child, Node::Text(_))),
                    "link text must stay plain text, got {:?}",
                    link.children
                );
            }
            other => panic!("expected link, got {other:?}"),
        },
        other => panic!("expected paragraph, got {other:?}"),
    }
}

#[test]
fn gfm_autolink_still_fires_inside_strikethrough() {
    let allocator = Allocator::new();
    let doc = parse_with_options(&allocator, "~~see www.example.com~~", ParserOptions::gfm());

    match &doc.children[0] {
        Node::Paragraph(paragraph) => match &paragraph.children[0] {
            Node::Delete(delete) => {
                let link_count =
                    delete.children.iter().filter(|child| matches!(child, Node::Link(_))).count();
                // The root-level pass recurses into emphasis-like
                // containers, so the bare URL still links exactly once.
                assert_eq!(link_count, 1, "expected one autolink, got {:?}", delete.children);
            }
            other => panic!("expected strikethrough, got {other:?}"),
        },
        other => panic!("expected paragraph, got {other:?}"),
    }
}

#[test]
fn gfm_autolink_does_not_link_ampersand_only_code_spans() {
    let allocator = Allocator::new();
    let doc = parse_with_options(&allocator, "see `&str` and `&mut T`", ParserOptions::gfm());

    match &doc.children[0] {
        Node::Paragraph(paragraph) => {
            assert!(
                paragraph
                    .children
                    .iter()
                    .all(|child| matches!(child, Node::Text(_) | Node::InlineCode(_))),
                "ampersand in code must not start an autolink pass rewrite, got {:?}",
                paragraph.children
            );
        }
        other => panic!("expected paragraph, got {other:?}"),
    }
}

#[test]
fn gfm_autolink_url_query_ampersand_still_links() {
    let allocator = Allocator::new();
    let doc = parse_with_options(
        &allocator,
        "www.google.com/search?q=commonmark&hl=en",
        ParserOptions::gfm(),
    );

    match &doc.children[0] {
        Node::Paragraph(paragraph) => {
            let link = paragraph.children.iter().find_map(|child| match child {
                Node::Link(link) => Some(link),
                _ => None,
            });
            let link = link.expect("expected autolink");
            assert!(link.url.contains("google.com/search"), "got {}", link.url);
        }
        other => panic!("expected paragraph, got {other:?}"),
    }
}

#[test]
fn gfm_autolink_stops_at_cjk_sentence_punctuation() {
    // Japanese prose puts no space between a URL and the `。` that closes
    // the sentence, so scanning to whitespace swallowed the rest of it.
    for (source, expected) in [
        ("句点直後: https://example.com/foo。次の文。", "https://example.com/foo"),
        ("読点: https://example.com/qux、続き。", "https://example.com/qux"),
        ("全角: https://example.com/baz）です。", "https://example.com/baz"),
        ("感嘆: https://example.com/a！", "https://example.com/a"),
        ("鉤括弧: https://example.com/b」", "https://example.com/b"),
    ] {
        let allocator = Allocator::new();
        let doc = parse_with_options(&allocator, source, ParserOptions::gfm());

        match &doc.children[0] {
            Node::Paragraph(paragraph) => {
                let link = paragraph
                    .children
                    .iter()
                    .find_map(|child| match child {
                        Node::Link(link) => Some(link),
                        _ => None,
                    })
                    .expect("expected autolink");
                assert_eq!(link.url, expected, "source: {source}");
            }
            other => panic!("expected paragraph, got {other:?}"),
        }
    }
}

#[test]
fn gfm_autolink_keeps_non_ascii_iri_paths() {
    let allocator = Allocator::new();
    let doc = parse_with_options(
        &allocator,
        "https://ja.wikipedia.org/wiki/日本語 です。",
        ParserOptions::gfm(),
    );

    match &doc.children[0] {
        Node::Paragraph(paragraph) => match &paragraph.children[0] {
            Node::Link(link) => assert_eq!(link.url, "https://ja.wikipedia.org/wiki/日本語"),
            other => panic!("expected link, got {other:?}"),
        },
        other => panic!("expected paragraph, got {other:?}"),
    }
}
