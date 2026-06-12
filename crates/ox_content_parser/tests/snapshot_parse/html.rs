use ox_content_parser::ParserOptions;

use super::check;

// --- Raw HTML ---

#[test]
fn snapshot_html_block_div() {
    check("html_block_div", "<div>\nraw html line\n</div>\n\nAfter\n", ParserOptions::default());
}

#[test]
fn snapshot_html_block_details() {
    check(
        "html_block_details",
        "<details id=\"a\">\n<summary>S</summary>\n<p>Body</p>\n</details>\n\nAfter\n",
        ParserOptions::default(),
    );
}

#[test]
fn snapshot_html_type6_details_resumes_markdown_after_blank() {
    check(
        "html_type6_details_resumes_markdown_after_blank",
        "<details>\n\n<summary>Click</summary>\n\n**bold**\n\n- list\n\n</details>\n",
        ParserOptions::default(),
    );
}

#[test]
fn snapshot_inline_raw_html_in_paragraph() {
    check(
        "inline_raw_html_in_paragraph",
        "before <span class=\"x\">middle</span> after\n",
        ParserOptions::default(),
    );
}
