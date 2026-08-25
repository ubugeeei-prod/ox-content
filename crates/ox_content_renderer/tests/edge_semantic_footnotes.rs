//! Semantic ordered footnote rendering (`semantic_footnotes`).

#[path = "support/edge.rs"]
mod edge_support;

use edge_support::render;
use ox_content_parser::ParserOptions;
use ox_content_renderer::HtmlRendererOptions;

fn semantic(source: &str) -> String {
    render(
        source,
        ParserOptions::gfm(),
        HtmlRendererOptions { semantic_footnotes: true, ..HtmlRendererOptions::default() },
    )
}

#[test]
fn default_keeps_legacy_in_place_markup() {
    let html = render(
        "Here is a note[^deployment-note].\n\n[^deployment-note]: The note text.\n",
        ParserOptions::gfm(),
        HtmlRendererOptions::default(),
    );

    assert!(html.contains("id=\"fnref-deployment-note\">deployment-note</a>"), "{html}");
    assert!(html.contains("<div id=\"fn-deployment-note\" class=\"footnote\">"), "{html}");
    assert!(!html.contains("<section class=\"footnotes\""), "{html}");
}

#[test]
fn named_labels_get_document_order_numeric_markers() {
    let html = semantic(
        "First[^deployment-note] then[^see-also].\n\n[^see-also]: Also.\n\n[^deployment-note]: Deploy.\n",
    );

    assert!(
        html.contains(
            "<sup><a href=\"#fn-deployment-note\" id=\"fnref-deployment-note\">1</a></sup>"
        ),
        "{html}"
    );
    assert!(
        html.contains("<sup><a href=\"#fn-see-also\" id=\"fnref-see-also\">2</a></sup>"),
        "{html}"
    );
    assert!(!html.contains(">deployment-note</a>"), "{html}");
    assert!(!html.contains(">see-also</a>"), "{html}");
}

#[test]
fn numeric_source_labels_are_renumbered_in_document_order() {
    let html = semantic("Start[^2] then[^1].\n\n[^1]: One.\n\n[^2]: Two.\n");

    assert!(html.contains("<sup><a href=\"#fn-2\" id=\"fnref-2\">1</a></sup>"), "{html}");
    assert!(html.contains("<sup><a href=\"#fn-1\" id=\"fnref-1\">2</a></sup>"), "{html}");
}

#[test]
fn definitions_emit_one_accessible_ordered_section() {
    let html = semantic("A[^note].\n\n[^note]: The note text.\n");

    assert!(html.contains("<section class=\"footnotes\" aria-label=\"Footnotes\">"), "{html}");
    assert!(html.contains("<ol>"), "{html}");
    assert!(html.contains("<li id=\"fn-note\">"), "{html}");
    assert!(html.contains("<p>The note text.</p>"), "{html}");
    assert!(html.contains("aria-label=\"Back to reference 1\">↩</a>"), "{html}");
    assert!(!html.contains("<div"), "{html}");
    assert!(!html.contains("class=\"footnote\""), "{html}");
    assert_eq!(html.matches("<section class=\"footnotes\"").count(), 1, "{html}");
}

#[test]
fn repeated_references_keep_unique_ids_and_every_backlink() {
    let html = semantic("A[^1] and B[^1] and C[^1].\n\n[^1]: Shared.\n");

    assert!(html.contains("id=\"fnref-1\""), "{html}");
    assert!(html.contains("id=\"fnref-1-2\""), "{html}");
    assert!(html.contains("id=\"fnref-1-3\""), "{html}");
    assert!(html.contains("href=\"#fnref-1\" aria-label=\"Back to reference 1\">↩</a>"), "{html}");
    assert!(
        html.contains("href=\"#fnref-1-2\" aria-label=\"Back to reference 1, occurrence 2\">↩</a>"),
        "{html}"
    );
    assert!(
        html.contains("href=\"#fnref-1-3\" aria-label=\"Back to reference 1, occurrence 3\">↩</a>"),
        "{html}"
    );
}

#[test]
fn definition_block_content_is_preserved_inside_the_list_item() {
    let html = semantic("C[^l].\n\n[^l]: - item one\n    - item two\n");

    let start = html.find("<li id=\"fn-l\">").expect(&html);
    let item = &html[start..];
    assert!(item.contains("<ul>"), "{html}");
    assert!(item.contains("<li>item one</li>"), "{html}");
    assert!(item.contains("<li>item two</li>"), "{html}");
}

#[test]
fn unicode_and_empty_slugs_and_collisions_stay_unique() {
    let html = semantic(
        "A[^注釈] B[^---] C[^foo_bar] D[^foo-bar].\n\n[^注釈]: U.\n\n[^---]: E.\n\n[^foo_bar]: X.\n\n[^foo-bar]: Y.\n",
    );

    assert!(html.contains("<sup><a href=\"#fn-注釈\" id=\"fnref-注釈\">1</a></sup>"), "{html}");
    assert!(
        html.contains("<sup><a href=\"#fn-footnote-2\" id=\"fnref-footnote-2\">2</a></sup>"),
        "{html}"
    );
    assert!(
        html.contains("<sup><a href=\"#fn-foo-bar\" id=\"fnref-foo-bar\">3</a></sup>"),
        "{html}"
    );
    assert!(
        html.contains("<sup><a href=\"#fn-foo-bar-2\" id=\"fnref-foo-bar-2\">4</a></sup>"),
        "{html}"
    );
    assert!(html.contains("<li id=\"fn-footnote-2\">"), "{html}");
    assert!(html.contains("<li id=\"fn-foo-bar-2\">"), "{html}");
}

#[test]
fn unused_definition_is_still_listed_without_backlinks() {
    let html = semantic("Only[^used].\n\n[^used]: Used.\n\n[^spare]: Unused.\n");

    assert!(html.contains("<li id=\"fn-used\">"), "{html}");
    assert!(html.contains("<li id=\"fn-spare\">"), "{html}");
    assert!(html.contains("<p>Unused.</p>"), "{html}");
    assert!(!html.contains("href=\"#fnref-spare\""), "{html}");
}

#[test]
fn undefined_reference_stays_literal_with_the_option_on() {
    assert_eq!(semantic("Missing[^nope].\n"), "<p>Missing[^nope].</p>\n");
}

#[test]
fn labels_still_match_case_insensitively() {
    let html = semantic("Ref[^Note].\n\n[^note]: Body.\n");

    assert!(html.contains("href=\"#fn-note\""), "{html}");
    assert!(html.contains(">1</a></sup>"), "{html}");
    assert!(html.contains("<p>Body.</p>"), "{html}");
}
