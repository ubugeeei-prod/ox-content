//! Definition lists let raw `<pre>` / `<code>` / `<script>` / `<style>`
//! blocks through untouched, which means every line starting with `<` is
//! tested against those four names.
//!
//! That test sliced the line at each name's byte length. A line opening
//! with `<x` followed by anything non-ASCII — Japanese prose, an emoji, a
//! byte-order mark — put that boundary inside a character and aborted the
//! process, because release builds do not unwind.

use std::panic::{AssertUnwindSafe, catch_unwind};

use ox_content_transform::transformer::MarkdownTransformer;
use ox_content_transform::{DefinitionListOptions, TransformOptions};

fn transformer() -> MarkdownTransformer {
    MarkdownTransformer::from_options(&TransformOptions {
        gfm: Some(true),
        definition_lists: Some(DefinitionListOptions { enabled: Some(true) }),
        ..Default::default()
    })
}

fn transform(markdown: &str) -> String {
    transformer().transform(markdown).html.trim().to_string()
}

/// A definition list has to be open for the raw-HTML check to run at all.
fn after_a_definition_list(line: &str) -> String {
    let mut markdown = String::from("term\n: def\n\n");
    markdown.push_str(line);
    markdown.push('\n');
    markdown
}

#[test]
fn a_line_opening_with_non_ascii_does_not_abort() {
    let transformer = transformer();
    // One case per byte length the tag-name check slices at: the boundary
    // has to land inside a character for each of `pre`, `code`, `script`,
    // and `style`.
    let lines = [
        "<x\u{3042}",
        "<\u{3042}\u{3042}",
        "<pr\u{3042}",
        "<cod\u{3042}",
        "<scrip\u{3042}",
        "<styl\u{3042}",
        "</x\u{3042}",
        "  <x\u{1F600}",
        "<x\u{FEFF}y",
        "<x\u{202E}y",
        "<\u{1F600}",
        "<x\u{0301}",
    ];
    for line in lines {
        let markdown = after_a_definition_list(line);
        let outcome = catch_unwind(AssertUnwindSafe(|| transformer.transform(&markdown)));
        assert!(outcome.is_ok(), "aborted on {line:?}");
    }
}

#[test]
fn non_ascii_after_a_definition_list_still_renders_as_text() {
    let html = transform(&after_a_definition_list("<x\u{3042}"));
    assert!(html.contains("<dl class=\"ox-definition-list\">"), "{html}");
    assert!(html.contains("&lt;x\u{3042}"), "{html}");
}

#[test]
fn raw_html_blocks_are_still_recognized() {
    // The byte-wise comparison must keep matching the four names, their
    // closing forms, and their case-insensitive spellings.
    for (line, expected) in [
        ("<pre>\u{3042}</pre>", "<pre>\u{3042}</pre>"),
        ("<PRE>x</PRE>", "<PRE>x</PRE>"),
        ("<script>x</script>", "<script>x</script>"),
        ("<style>y</style>", "<style>y</style>"),
        ("<pre/>", "<pre/>"),
    ] {
        let html = transform(&after_a_definition_list(line));
        assert!(html.contains(expected), "{line:?} produced {html}");
    }
}

#[test]
fn a_definition_list_around_raw_html_keeps_both() {
    let html = transform("term\n: def\n\n<pre>code</pre>\n\nother\n: meaning\n");
    assert_eq!(html.matches("<dl class=\"ox-definition-list\">").count(), 2, "{html}");
    assert!(html.contains("<pre>code</pre>"), "{html}");
}
