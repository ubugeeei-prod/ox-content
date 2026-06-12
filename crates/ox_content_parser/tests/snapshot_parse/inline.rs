use ox_content_parser::ParserOptions;

use super::check;

// --- Inline content ---

#[test]
fn snapshot_inline_emphasis_strong_combined() {
    check(
        "inline_emphasis_strong_combined",
        "An *italic* word and a **bold** one and a ***both*** one.\n",
        ParserOptions::default(),
    );
}

#[test]
fn snapshot_inline_underscore_emphasis() {
    check(
        "inline_underscore_emphasis",
        "_italic_ and __bold__ and ___both___.\n",
        ParserOptions::default(),
    );
}

#[test]
fn snapshot_inline_strikethrough_gfm() {
    check("inline_strikethrough_gfm", "~~gone~~ but kept.\n", ParserOptions::gfm());
}

#[test]
fn snapshot_inline_strikethrough_unmatched() {
    check("inline_strikethrough_unmatched", "~~open\n", ParserOptions::gfm());
}

#[test]
fn snapshot_inline_code_basic() {
    check("inline_code_basic", "Use `let x = 1;` to declare a value.\n", ParserOptions::default());
}

#[test]
fn snapshot_inline_code_with_html_literal() {
    check(
        "inline_code_with_html_literal",
        "Show `<input type=\"checkbox\">` literally.\n",
        ParserOptions::default(),
    );
}

#[test]
fn snapshot_inline_hard_break_backslash() {
    check("inline_hard_break_backslash", "line 1\\\nline 2\n", ParserOptions::default());
}

#[test]
fn snapshot_inline_escaped_punctuation() {
    check(
        "inline_escaped_punctuation",
        "\\*not italic\\* and \\_not underscore\\_\n",
        ParserOptions::default(),
    );
}

// --- Links and images ---

#[test]
fn snapshot_inline_link_simple() {
    check("inline_link_simple", "See [the site](https://example.com).\n", ParserOptions::default());
}

#[test]
fn snapshot_inline_link_with_title() {
    check(
        "inline_link_with_title",
        "See [home](https://example.com \"Example Home\").\n",
        ParserOptions::default(),
    );
}

#[test]
fn snapshot_inline_link_nested_parentheses() {
    check(
        "inline_link_nested_parentheses",
        "[docs](https://example.com/a(b)c)\n",
        ParserOptions::default(),
    );
}

#[test]
fn snapshot_image_simple() {
    check("image_simple", "![alt](./logo.png)\n", ParserOptions::default());
}

#[test]
fn snapshot_image_with_title() {
    check("image_with_title", "![alt](./logo.png \"A logo\")\n", ParserOptions::default());
}

#[test]
fn snapshot_image_nested_parentheses() {
    check("image_nested_parentheses", "![diagram](./img(test).png)\n", ParserOptions::default());
}
