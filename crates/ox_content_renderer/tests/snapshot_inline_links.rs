#[path = "support/snapshot.rs"]
mod snapshot_support;

use ox_content_parser::ParserOptions;
use ox_content_renderer::HtmlRendererOptions;
use snapshot_support::check;

#[test]
fn html_inline_emphasis_strong_combined() {
    check(
        "inline_emphasis_strong_combined",
        "*it* **bold** ***both*** `code`\n",
        ParserOptions::default(),
        HtmlRendererOptions::default(),
    );
}

#[test]
fn html_inline_strikethrough_gfm() {
    check(
        "inline_strikethrough_gfm",
        "~~gone~~ kept\n",
        ParserOptions::gfm(),
        HtmlRendererOptions::default(),
    );
}

#[test]
fn html_inline_hard_break_backslash() {
    check(
        "inline_hard_break_backslash",
        "line 1\\\nline 2\n",
        ParserOptions::default(),
        HtmlRendererOptions::default(),
    );
}

#[test]
fn html_inline_escaped_punctuation() {
    check(
        "inline_escaped_punctuation",
        "\\*not italic\\*\n",
        ParserOptions::default(),
        HtmlRendererOptions::default(),
    );
}

#[test]
fn html_inline_special_chars_are_escaped() {
    check(
        "inline_special_chars_are_escaped",
        "5 < 6 & 7 > 4, quote: \"hi\" 'bye'\n",
        ParserOptions::default(),
        HtmlRendererOptions::default(),
    );
}

// --- Links and images ---

#[test]
fn html_external_link_adds_security_attrs() {
    check(
        "external_link_adds_security_attrs",
        "[site](https://example.com)\n",
        ParserOptions::default(),
        HtmlRendererOptions::default(),
    );
}

#[test]
fn html_relative_link_has_no_external_attrs() {
    check(
        "relative_link_has_no_external_attrs",
        "[guide](./guide.md)\n",
        ParserOptions::default(),
        HtmlRendererOptions::default(),
    );
}

#[test]
fn html_link_with_title() {
    check(
        "link_with_title",
        "[home](https://example.com \"Title\")\n",
        ParserOptions::default(),
        HtmlRendererOptions::default(),
    );
}

#[test]
fn html_image_basic() {
    check(
        "image_basic",
        "![alt](./logo.png)\n",
        ParserOptions::default(),
        HtmlRendererOptions::default(),
    );
}

#[test]
fn html_image_xhtml_self_closes() {
    check(
        "image_xhtml_self_closes",
        "![logo](/logo.svg)\n",
        ParserOptions::default(),
        HtmlRendererOptions { xhtml: true, ..HtmlRendererOptions::default() },
    );
}

// --- Sanitization ---
