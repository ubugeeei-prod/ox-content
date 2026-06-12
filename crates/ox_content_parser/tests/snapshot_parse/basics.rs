use ox_content_parser::ParserOptions;

use super::check;

// --- Whitespace, blank input, trivial documents ---

#[test]
fn snapshot_empty_document() {
    check("empty_document", "", ParserOptions::default());
}

#[test]
fn snapshot_whitespace_only_document() {
    check("whitespace_only_document", "\n  \n\t\n   \n", ParserOptions::default());
}

#[test]
fn snapshot_single_paragraph() {
    check("single_paragraph", "Just one paragraph.", ParserOptions::default());
}

// --- Headings ---

#[test]
fn snapshot_atx_headings_all_depths() {
    check(
        "atx_headings_all_depths",
        "# h1\n## h2\n### h3\n#### h4\n##### h5\n###### h6\n",
        ParserOptions::default(),
    );
}

#[test]
fn snapshot_atx_heading_with_trailing_hashes() {
    check(
        "atx_heading_with_trailing_hashes",
        "## Title ###\n## Edge  ####  \n",
        ParserOptions::default(),
    );
}

#[test]
fn snapshot_atx_heading_requires_space() {
    check("atx_heading_requires_space", "#NoSpace\n## With space\n", ParserOptions::default());
}

#[test]
fn snapshot_atx_heading_too_many_hashes() {
    check("atx_heading_too_many_hashes", "####### too many\n", ParserOptions::default());
}

// --- Thematic breaks ---

#[test]
fn snapshot_thematic_break_variants() {
    check(
        "thematic_break_variants",
        "---\n\n___\n\n***\n\n  ---  \n\n - - -\n\n* * *\n",
        ParserOptions::default(),
    );
}

#[test]
fn snapshot_thematic_break_invalid_mixed_markers() {
    check("thematic_break_invalid_mixed_markers", "- * -\n", ParserOptions::default());
}
