//! The parse/render entry points share one arena and one renderer per thread,
//! so a call must not be able to see anything the previous call left behind.
//! These cover the state that reuse could leak: the arena, the renderer's output
//! buffer, its heading-id counter, and the arena-sizing decision.

use ox_content_parser::ParserOptions;

use crate::render_scratch::{parse_and_render_html, parse_to_mdast_json};

fn render(source: &str) -> String {
    parse_and_render_html(source, ParserOptions::gfm()).expect("render should succeed")
}

#[test]
fn consecutive_renders_do_not_leak_into_each_other() {
    let first = render("# First\n\nOne paragraph.\n");
    let second = render("# Second\n");
    let third = render("# First\n\nOne paragraph.\n");

    assert!(first.contains("First"), "unexpected first render: {first}");
    assert!(!second.contains("First"), "second render kept the first document: {second}");
    assert!(!second.contains("paragraph"), "second render kept the first document: {second}");
    assert_eq!(first, third, "the same source rendered differently the second time");
}

#[test]
fn heading_ids_restart_for_each_document() {
    // Duplicate slugs get a `-1`, `-2`, … suffix within one document. A renderer
    // that carried its counter across calls would number the second document's
    // first `## Dup` as a duplicate of the first document's.
    let first = render("## Dup\n\n## Dup\n");
    assert!(first.contains("id=\"dup\""), "missing base id: {first}");
    assert!(first.contains("id=\"dup-1\""), "missing deduplicated id: {first}");

    let second = render("## Dup\n");
    assert!(second.contains("id=\"dup\""), "heading id did not restart: {second}");
    assert!(!second.contains("dup-1"), "heading id counter leaked across documents: {second}");
}

#[test]
fn a_large_document_does_not_corrupt_the_next_small_one() {
    // The large source both grows the arena past the retained bound and forces
    // the next call to decide between resetting and re-sizing it.
    let large = "# Heading\n\nParagraph with **bold** text.\n".repeat(4000);
    let large_html = render(&large);
    assert!(large_html.len() > large.len(), "large render looks truncated");

    let small = render("Just a sentence.\n");
    assert_eq!(small, "<p>Just a sentence.</p>\n");
}

#[test]
fn options_are_not_carried_over_between_calls() {
    let table = "| a | b |\n| - | - |\n| 1 | 2 |\n";

    let with_gfm = render(table);
    assert!(with_gfm.contains("<table>"), "GFM tables should render: {with_gfm}");

    let without_gfm =
        parse_and_render_html(table, ParserOptions::default()).expect("render should succeed");
    assert!(!without_gfm.contains("<table>"), "table parsed without GFM enabled: {without_gfm}");
}

#[test]
fn the_ast_entry_point_shares_the_arena_safely() {
    // `parse` and `parseAndRender` reset the same arena, so interleaving them is
    // the case where a stale borrow would surface.
    let ast = parse_to_mdast_json("# Title\n", ParserOptions::gfm()).expect("parse should succeed");
    let html = render("# Other\n");
    let ast_again =
        parse_to_mdast_json("# Title\n", ParserOptions::gfm()).expect("parse should succeed");

    assert!(ast.contains("Title"), "unexpected ast: {ast}");
    assert!(html.contains("Other"), "unexpected html: {html}");
    assert_eq!(ast, ast_again, "the same source parsed differently after an interleaved render");
}
