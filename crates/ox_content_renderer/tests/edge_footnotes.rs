//! End-to-end footnote behaviour (GFM extension).

#[path = "support/edge.rs"]
mod edge_support;

use edge_support::render;
use ox_content_parser::ParserOptions;
use ox_content_renderer::HtmlRendererOptions;

fn gfm(source: &str) -> String {
    render(source, ParserOptions::gfm(), HtmlRendererOptions::default())
}

#[test]
fn reference_and_definition_render_as_linked_pair() {
    let html = gfm("Here is a note[^1].\n\n[^1]: The note text.\n");

    assert!(html.contains("<sup><a href=\"#fn-1\" id=\"fnref-1\">1</a></sup>"), "{html}");
    assert!(html.contains("<div id=\"fn-1\" class=\"footnote\">"), "{html}");
    assert!(html.contains("<p>The note text.</p>"), "{html}");
    assert!(html.contains("<a href=\"#fnref-1\">↩</a>"), "{html}");
}

#[test]
fn definition_is_not_treated_as_a_link_reference() {
    // Regression: `[^1]: text` used to parse as a link reference
    // definition with label `^1`, turning every `[^1]` into a link
    // pointing at the definition text.
    let html = gfm("A[^1] and B[^1].\n\n[^1]: Shared.\n");

    assert!(!html.contains("href=\"Shared.\""), "{html}");
    assert!(html.contains("href=\"#fn-1\""), "{html}");
}

#[test]
fn repeated_references_get_unique_ids() {
    let html = gfm("A[^1] and B[^1] and C[^1].\n\n[^1]: Shared.\n");

    assert!(html.contains("id=\"fnref-1\""), "{html}");
    assert!(html.contains("id=\"fnref-1-2\""), "{html}");
    assert!(html.contains("id=\"fnref-1-3\""), "{html}");
}

#[test]
fn undefined_reference_stays_literal_text() {
    let html = gfm("Missing[^nope].\n");

    assert_eq!(html, "<p>Missing[^nope].</p>\n");
}

#[test]
fn definition_body_takes_indented_continuation_blocks() {
    let html = gfm("A[^x].\n\n[^x]: First para.\n\n    Second para.\n");

    assert!(html.contains("<p>First para.</p>"), "{html}");
    assert!(html.contains("<p>Second para.</p>"), "{html}");
    // The continuation must land inside the footnote, not become code.
    assert!(!html.contains("<pre>"), "{html}");
}

#[test]
fn definition_body_supports_block_content() {
    let html = gfm("C[^l].\n\n[^l]: - item one\n    - item two\n");

    assert!(html.contains("<ul>"), "{html}");
    assert!(html.contains("<li>item one</li>"), "{html}");
    assert!(html.contains("<li>item two</li>"), "{html}");
}

#[test]
fn labels_match_case_insensitively() {
    let html = gfm("Ref[^Note].\n\n[^note]: Body.\n");

    assert!(html.contains("href=\"#fn-note\""), "{html}");
    assert!(html.contains("<p>Body.</p>"), "{html}");
}

#[test]
fn footnotes_stay_literal_without_the_extension() {
    // With footnotes disabled, `[^1]: url` is a valid CommonMark link
    // reference definition and `[^1]` a shortcut reference to it.
    let html =
        render("A[^1].\n\n[^1]: /url\n", ParserOptions::default(), HtmlRendererOptions::default());

    assert!(html.contains("href=\"/url\""), "{html}");
    assert!(!html.contains("footnote"), "{html}");
}

#[test]
fn definition_inside_fenced_code_is_not_collected() {
    let html = gfm("```\n[^x]: not a definition\n```\n\nPlain[^x].\n");

    assert!(html.contains("<pre><code>[^x]: not a definition"), "{html}");
    assert!(html.contains("Plain[^x]."), "{html}");
    assert!(!html.contains("<sup>"), "{html}");
}
