//! Fused pre-pass semantics: where link reference definitions and
//! footnote labels are (and are not) collected. These pin the exact
//! behavior of the former two standalone pre-passes.

use ox_content_allocator::Allocator;
use ox_content_ast::Node;
use ox_content_parser::ParserOptions;

use super::parse_with_options;

/// True when any node in the tree (recursively) is a resolved `Link`.
fn contains_link(nodes: &[Node<'_>]) -> bool {
    nodes.iter().any(|node| match node {
        Node::Link(_) => true,
        Node::Paragraph(p) => contains_link(&p.children),
        Node::Heading(h) => contains_link(&h.children),
        Node::BlockQuote(q) => contains_link(&q.children),
        Node::Emphasis(e) => contains_link(&e.children),
        Node::Strong(s) => contains_link(&s.children),
        Node::List(l) => l.children.iter().any(|item| contains_link(&item.children)),
        Node::ListItem(i) => contains_link(&i.children),
        _ => false,
    })
}

/// True when any node in the tree (recursively) is a `FootnoteReference`.
fn contains_footnote_ref(nodes: &[Node<'_>]) -> bool {
    nodes.iter().any(|node| match node {
        Node::FootnoteReference(_) => true,
        Node::Paragraph(p) => contains_footnote_ref(&p.children),
        Node::BlockQuote(q) => contains_footnote_ref(&q.children),
        _ => false,
    })
}

fn resolves_reference(source: &str) -> bool {
    let allocator = Allocator::new();
    let doc = parse_with_options(&allocator, source, ParserOptions::default());
    contains_link(&doc.children)
}

fn resolves_footnote(source: &str) -> bool {
    let allocator = Allocator::new();
    let doc = parse_with_options(&allocator, source, ParserOptions::gfm());
    contains_footnote_ref(&doc.children)
}

#[test]
fn definition_after_use_resolves() {
    assert!(resolves_reference("[a]\n\n[a]: /url"));
}

#[test]
fn definition_before_use_resolves() {
    assert!(resolves_reference("[a]: /url\n\n[a]"));
}

#[test]
fn fenced_code_hides_definition() {
    assert!(!resolves_reference("```\n[a]: /url\n```\n\n[a]"));
}

#[test]
fn tilde_fenced_code_hides_definition() {
    assert!(!resolves_reference("~~~\n[a]: /url\n~~~\n\n[a]"));
}

#[test]
fn unclosed_fence_hides_rest_of_document() {
    assert!(!resolves_reference("```\n[a]: /url\n\n[a]"));
}

#[test]
fn quoted_definition_resolves() {
    assert!(resolves_reference("> [a]: /url\n\n[a]"));
}

#[test]
fn indented_quote_marker_definition_resolves() {
    assert!(resolves_reference("  > [a]: /url\n\n[a]"));
}

#[test]
fn paragraph_continuation_is_not_a_definition() {
    assert!(!resolves_reference("text\n[a]: /url\n\n[a]"));
}

#[test]
fn list_item_continuation_is_not_a_definition() {
    assert!(!resolves_reference("- item\n[a]: /url\n\n[a]"));
}

#[test]
fn heading_line_closes_paragraph_context() {
    assert!(resolves_reference("# heading\n[a]: /url\n\n[a]"));
}

#[test]
fn setext_underline_closes_paragraph_context() {
    assert!(resolves_reference("text\n===\n[a]: /url\n\n[a]"));
}

#[test]
fn thematic_break_closes_paragraph_context() {
    assert!(resolves_reference("text\n---\n[a]: /url\n\n[a]"));
}

#[test]
fn chained_definitions_all_resolve() {
    let source = "[a]: /1\n[b]: /2\n[c]: /3\n\n[a] [b] [c]";
    let allocator = Allocator::new();
    let doc = parse_with_options(&allocator, source, ParserOptions::default());
    let links = doc
        .children
        .iter()
        .filter_map(|node| match node {
            Node::Paragraph(p) => {
                Some(p.children.iter().filter(|n| matches!(n, Node::Link(_))).count())
            }
            _ => None,
        })
        .sum::<usize>();
    assert_eq!(links, 3);
}

#[test]
fn three_space_indent_is_a_definition() {
    assert!(resolves_reference("   [a]: /url\n\n[a]"));
}

#[test]
fn four_space_indent_is_not_a_definition() {
    assert!(!resolves_reference("    [a]: /url\n\n[a]"));
}

#[test]
fn definition_at_start_of_source_resolves() {
    assert!(resolves_reference("[a]: /url\n[a]"));
}

#[test]
fn definition_without_trailing_newline_resolves() {
    assert!(resolves_reference("[a]\n\n[a]: /url"));
}

#[test]
fn crlf_definition_does_not_resolve() {
    // Parity pin, not an endorsement: the standalone passes never
    // recognized definitions in CRLF sources (the trailing `\r` defeats
    // the `]:` check and the `\r`-only blank line keeps the paragraph
    // open), and the fused pass preserves that.
    assert!(!resolves_reference("[a]\r\n\r\n[a]: /url"));
}

#[test]
fn footnote_definition_after_use_resolves() {
    assert!(resolves_footnote("[^n]\n\n[^n]: note"));
}

#[test]
fn footnote_definition_before_use_resolves() {
    assert!(resolves_footnote("[^n]: note\n\n[^n]"));
}

#[test]
fn fenced_code_hides_footnote_definition() {
    assert!(!resolves_footnote("```\n[^n]: note\n```\n\n[^n]"));
}

#[test]
fn footnote_without_definition_stays_literal() {
    assert!(!resolves_footnote("[^n] and no definition"));
}

#[test]
fn indented_footnote_definition_up_to_three_spaces_resolves() {
    assert!(resolves_footnote("   [^n]: note\n\n[^n]"));
}

// The two collectors deliberately classify fences differently: the
// reference side tracks fences on the quote-stripped line, the footnote
// side on the raw line. A quoted fence line therefore opens a fence for
// the reference collector only. These two tests pin that asymmetry.

#[test]
fn quoted_fence_hides_later_reference_definition() {
    assert!(!resolves_reference("> ```\n\n[a]: /url\n\n[a]"));
}

#[test]
fn quoted_fence_does_not_hide_later_footnote_definition() {
    assert!(resolves_footnote("> ```\n\n[^n]: note\n\n[^n]"));
}

// A multi-line reference definition (title on its own line) is skipped in
// one jump by the reference collector; the footnote scan must still see
// the skipped lines exactly as the standalone pass did.

#[test]
fn footnote_definition_directly_after_multiline_definition_resolves() {
    assert!(resolves_footnote("[a]: /url\n\"title\"\n[^n]: note\n\n[^n]"));
}

#[test]
fn multiline_definition_with_title_resolves() {
    assert!(resolves_reference("[a]: /url\n\"title\"\n\n[a]"));
}

#[test]
fn bracket_and_colon_split_across_lines_is_not_a_definition() {
    // `]` at end of one line and `:` starting the next never form a
    // definition (the joined chunk carries the newline between them), so
    // the pre-pass may skip sources without a contiguous "]:".
    assert!(!resolves_reference("[a]\n: /url\n\n[a]"));
}

#[test]
fn escaped_bracket_does_not_end_a_label() {
    // `\]` stays inside the label; the definition's real close is the
    // unescaped `]:` later, which the needle gate must also see.
    assert!(resolves_reference("[a\\]b]: /url\n\n[a\\]b]"));
}

#[test]
fn link_only_document_resolves_nothing_and_parses_fine() {
    let allocator = Allocator::new();
    let doc = parse_with_options(
        &allocator,
        "Here's a [link](https://example.com) and [another](/x).",
        ParserOptions::default(),
    );
    assert!(contains_link(&doc.children));
}
