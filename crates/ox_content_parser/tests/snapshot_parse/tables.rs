use ox_content_parser::ParserOptions;

use super::check;

// --- Tables (GFM) ---

#[test]
fn snapshot_table_alignment_variants() {
    check(
        "table_alignment_variants",
        "| a | b | c |\n| :-- | :-: | --: |\n| 1 | 2 | 3 |\n",
        ParserOptions::gfm(),
    );
}

#[test]
fn snapshot_table_with_inline_formatting() {
    check(
        "table_with_inline_formatting",
        "| name | status |\n| ---- | ------ |\n| **bold** | *italic* |\n| `code` | ~~old~~ |\n",
        ParserOptions::gfm(),
    );
}

// --- Definitions / footnotes ---

#[test]
fn snapshot_reference_link_definition() {
    check(
        "reference_link_definition",
        "[link][ref]\n\n[ref]: https://example.com \"Title\"\n",
        ParserOptions::default(),
    );
}

#[test]
fn snapshot_footnote_reference_and_definition() {
    let mut options = ParserOptions::gfm();
    options.footnotes = true;
    check("footnote_reference_and_definition", "See[^1].\n\n[^1]: The footnote body.\n", options);
}
