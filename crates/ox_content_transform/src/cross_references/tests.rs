use super::*;

fn options() -> CrossReferencesOptions {
    CrossReferencesOptions { enabled: true, ..CrossReferencesOptions::default() }
}

fn warn_options() -> CrossReferencesOptions {
    CrossReferencesOptions {
        enabled: true,
        missing: FailureMode::Warn,
        duplicates: FailureMode::Warn,
        mismatches: FailureMode::Warn,
        ..CrossReferencesOptions::default()
    }
}

#[test]
fn disabled_returns_the_input_untouched() {
    let html = "<h1 id=\"sec-a\">A</h1><p>@sec-a</p>";
    let output = transform_cross_references(html, &CrossReferencesOptions::default());
    assert_eq!(output.html, html);
    assert!(output.references.is_empty());
}

#[test]
fn sections_number_hierarchically() {
    let html = "<h1 id=\"sec-a\">A</h1><h2 id=\"sec-b\">B</h2><h3 id=\"sec-c\">C</h3>";
    let output = transform_cross_references(html, &options());
    let numbers: Vec<&str> = output.references.iter().map(|entry| entry.number.as_str()).collect();
    assert_eq!(numbers, vec!["1", "1.1", "1.1.1"]);
}

#[test]
fn a_deeper_level_restarts_when_its_parent_moves_on() {
    let html = "<h1 id=\"sec-a\">A</h1><h2 id=\"sec-b\">B</h2><h1 id=\"sec-c\">C</h1><h2 id=\"sec-d\">D</h2>";
    let output = transform_cross_references(html, &options());
    let numbers: Vec<&str> = output.references.iter().map(|entry| entry.number.as_str()).collect();
    assert_eq!(numbers, vec!["1", "1.1", "2", "2.1"]);
}

#[test]
fn a_figure_without_its_own_id_borrows_the_first_image() {
    let html = "<figure><img id=\"fig-a\" src=\"a.png\" alt=\"Alt\"></figure>";
    let output = transform_cross_references(html, &options());
    assert_eq!(output.references.len(), 1);
    assert_eq!(output.references[0].id, "fig-a");
    // The attributes land on the image that supplied the id, not the figure.
    assert!(output.html.contains("<img id=\"fig-a\""), "{}", output.html);
    assert!(output.html.contains("data-ox-xref-number=\"1\""), "{}", output.html);
    assert!(!output.html.contains("<figure data-ox"), "{}", output.html);
}

#[test]
fn figures_and_images_share_one_sequence() {
    let html = "<figure id=\"fig-a\"><img src=\"a.png\"></figure><img id=\"fig-b\" src=\"b.png\">";
    let output = transform_cross_references(html, &options());
    let numbers: Vec<&str> = output.references.iter().map(|entry| entry.number.as_str()).collect();
    assert_eq!(numbers, vec!["1", "2"]);
}

#[test]
fn reported_order_follows_the_offsets_each_pass_saw() {
    // Not document order, though it is close enough to look like it. Each pass
    // records offsets into the string *it* walked, and every pass rewrites the
    // document, so the figure's offset is measured in a string already grown by
    // the section attributes — placing it after both headings rather than
    // between them. The TypeScript implementation did the same; this pins the
    // behaviour rather than the intention.
    let html = "<h1 id=\"sec-a\">A</h1><img id=\"fig-a\" src=\"a.png\"><h2 id=\"sec-b\">B</h2>";
    let output = transform_cross_references(html, &options());
    let ids: Vec<&str> = output.references.iter().map(|entry| entry.id.as_str()).collect();
    assert_eq!(ids, vec!["sec-a", "sec-b", "fig-a"]);
}

#[test]
fn a_trailing_paragraph_label_is_lifted_onto_the_table() {
    let html = "<table><tbody><tr><td>x</td></tr></tbody></table>\n<p>{#tbl-a}</p>";
    let output = transform_cross_references(html, &options());
    assert_eq!(output.references.len(), 1);
    assert_eq!(output.references[0].id, "tbl-a");
    assert!(!output.html.contains("{#tbl-a}"), "{}", output.html);
}

#[test]
fn a_trailing_empty_row_label_is_lifted_and_removed() {
    let html =
        "<table><tbody><tr><td>x</td></tr><tr><td id=\"tbl-a\"></td><td></td></tr></tbody></table>";
    let output = transform_cross_references(html, &options());
    assert_eq!(output.references.len(), 1);
    assert_eq!(output.references[0].id, "tbl-a");
    assert!(!output.html.contains("tbl-a\"></td>"), "{}", output.html);
}

#[test]
fn a_reference_becomes_a_link() {
    let html = "<h1 id=\"sec-a\">A</h1><p>See @sec-a.</p>";
    let output = transform_cross_references(html, &options());
    assert!(
        output.html.contains("<a class=\"ox-xref ox-xref-section\" href=\"#sec-a\""),
        "{}",
        output.html
    );
    assert!(output.html.contains(">Section 1</a>"), "{}", output.html);
}

#[test]
fn the_prefix_character_is_consumed_so_adjacent_references_do_not_both_match() {
    let html = "<h1 id=\"sec-a\">A</h1><h2 id=\"sec-b\">B</h2><p>@sec-a@sec-b</p>";
    let output = transform_cross_references(html, &options());
    assert_eq!(output.html.matches("ox-xref-section\"").count(), 1, "{}", output.html);
}

#[test]
fn an_identifier_ending_in_a_dash_is_not_a_reference() {
    // `-` is not a word character, so the trailing boundary needs one after it.
    let html = "<h1 id=\"sec-a\">A</h1><p>@sec- alone</p>";
    let output = transform_cross_references(html, &warn_options());
    assert!(output.html.contains("@sec- alone"), "{}", output.html);
}

#[test]
fn verbatim_elements_are_left_alone() {
    let html = "<h1 id=\"sec-a\">A</h1><p><code>@sec-a</code> @sec-a</p>";
    let output = transform_cross_references(html, &options());
    assert!(output.html.contains("<code>@sec-a</code>"), "{}", output.html);
    assert_eq!(output.html.matches("ox-xref-section\"").count(), 1, "{}", output.html);
}

#[test]
fn address_is_not_an_anchor() {
    let html = "<h1 id=\"sec-a\">A</h1><address>@sec-a</address>";
    let output = transform_cross_references(html, &options());
    assert!(output.html.contains("ox-xref-section"), "{}", output.html);
}

#[test]
fn citation_groups_belong_to_the_citation_pass() {
    let html = "<h1 id=\"sec-a\">A</h1><p>[@smith2020] @sec-a</p>";
    let output = transform_cross_references(html, &options());
    assert!(output.html.contains("[@smith2020]"), "{}", output.html);
    assert!(output.html.contains("ox-xref-section"), "{}", output.html);
}

#[test]
fn a_missing_target_is_reported_not_raised() {
    let output = transform_cross_references("<p>@fig-nope</p>", &options());
    assert_eq!(output.diagnostics.len(), 1);
    assert_eq!(output.diagnostics[0].policy, FailureMode::Error);
    assert!(output.diagnostics[0].message.contains("missing"), "{output:?}");
    // The text is left as written so the author can see what failed.
    assert!(output.html.contains("@fig-nope"), "{}", output.html);
}

#[test]
fn a_kind_mismatch_is_reported() {
    let html = "<h1 id=\"fig-a\">A</h1><p>@fig-a</p>";
    let output = transform_cross_references(html, &options());
    assert!(
        output.diagnostics.iter().any(|d| d.message.contains("expects figure but found section")),
        "{output:?}"
    );
}

#[test]
fn a_duplicate_id_is_reported_and_the_first_wins() {
    let html = "<h1 id=\"sec-a\">First</h1><h2 id=\"sec-a\">Second</h2>";
    let output = transform_cross_references(html, &options());
    assert_eq!(output.references.len(), 1);
    assert_eq!(output.references[0].title.as_deref(), Some("First"));
    assert!(output.diagnostics.iter().any(|d| d.message.contains("duplicate")), "{output:?}");
}

#[test]
fn an_untracked_id_is_neither_numbered_nor_linked() {
    let html = "<h1 id=\"intro\">A</h1><p>@intro</p>";
    let output = transform_cross_references(html, &options());
    assert!(output.references.is_empty());
    assert!(output.html.contains("@intro"), "{}", output.html);
    assert!(output.diagnostics.is_empty(), "{output:?}");
}

#[test]
fn the_header_anchor_is_not_part_of_the_title() {
    let html = "<h1 id=\"sec-a\">Alpha<a class=\"header-anchor\" href=\"#sec-a\">#</a></h1>";
    let output = transform_cross_references(html, &options());
    assert_eq!(output.references[0].title.as_deref(), Some("Alpha"));
}

#[test]
fn an_already_annotated_image_keeps_its_number() {
    let html = "<img id=\"fig-a\" data-ox-xref-kind=\"figure\" src=\"a.png\"><img id=\"fig-b\" src=\"b.png\">";
    let output = transform_cross_references(html, &options());
    // Only the second is numbered, and it starts the sequence.
    assert_eq!(output.references.len(), 1);
    assert_eq!(output.references[0].id, "fig-b");
    assert_eq!(output.references[0].number, "1");
}

#[test]
fn an_id_needing_escaping_is_percent_encoded_in_the_href() {
    let html = "<h1 id=\"sec-a b\">A</h1>";
    let output = transform_cross_references(html, &options());
    assert_eq!(output.references[0].href, "#sec-a%20b");
}
