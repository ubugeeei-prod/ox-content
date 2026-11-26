//! Inline raw HTML: a comment, processing instruction or CDATA section is
//! raw HTML only when its own terminator follows, and each opener found
//! that out by searching to the end of the content. A run of openers with
//! nothing to end them was O(n²): 256 KiB of `<!-- ` took 3.8 s, of `<? `
//! 6.4 s, of `<![CDATA[ ` 0.5 s, in a paragraph with a `>` at its end.
//!
//! These tests pin the cost back to linear and pin the output on both sides
//! of the remembered search, which only content longer than 256 bytes uses.

use ox_content_allocator::Allocator;
use ox_content_parser::{Parser, ParserOptions};
use ox_content_renderer::HtmlRenderer;

#[path = "support/scaling.rs"]
mod scaling;

use scaling::Part::{Fill, Once};

#[test]
fn openers_with_nothing_to_end_them_cost_linear_time() {
    // The `>` at the end gets every `<` past the "no `>` at all" check.
    scaling::assert_linear(
        &ParserOptions::default(),
        &[
            &[Once("x "), Fill("<!-- "), Once(">")],
            &[Once("x "), Fill("<? "), Once(">")],
            &[Once("x "), Fill("<![CDATA[ "), Once(">")],
            &[Once("x "), Fill("<!-- <? <![CDATA[ "), Once(">")],
            &[Once("x [ "), Fill("<!-- "), Once(">")],
            &[Once("x "), Fill("<!-- --> <!-- "), Once(">")],
        ],
    );
}

fn render(source: &str) -> String {
    let allocator = Allocator::new();
    let document = Parser::with_options(&allocator, source, ParserOptions::default())
        .parse()
        .expect("source should parse");
    HtmlRenderer::new().render(&document).trim().to_string()
}

#[test]
fn terminated_bodies_open_and_close_as_they_did() {
    // Each case is checked as written, and again behind enough text to take
    // the remembered search.
    let padding = "p ".repeat(200);
    for (source, expected) in [
        ("a <!-- b --> c", "a <!-- b --> c"),
        ("a <!-- b -- c", "a &lt;!-- b -- c"),
        ("a <!-- b --> <!-- c", "a <!-- b --> &lt;!-- c"),
        ("a <!-- b <!-- c -->", "a <!-- b <!-- c -->"),
        ("a --> <!-- b", "a --&gt; &lt;!-- b"),
        ("a <!--> b", "a <!--> b"),
        ("a <!---> b", "a <!---> b"),
        ("a <!---->", "a <!---->"),
        ("a <? b ?> c", "a <? b ?> c"),
        ("a <? b > c", "a &lt;? b &gt; c"),
        ("a ?> <? b >", "a ?&gt; &lt;? b &gt;"),
        ("a <?> b ?>", "a <?> b ?>"),
        ("a <![CDATA[ b ]]> c", "a <![CDATA[ b ]]> c"),
        ("a <![CDATA[ b ]> c", "a &lt;![CDATA[ b ]&gt; c"),
        ("a ]]> <![CDATA[ b >", "a ]]&gt; &lt;![CDATA[ b &gt;"),
        ("a <!X b> c", "a <!X b> c"),
        ("[a <!-- b](c) -->", "[a <!-- b](c) -->"),
        ("[a <!-- b](c)", "<a href=\"c\">a &lt;!-- b</a>"),
    ] {
        assert_eq!(render(source), format!("<p>{expected}</p>"), "for {source:?}");
        let long = format!("{padding}{source}");
        assert_eq!(render(&long), format!("<p>{padding}{expected}</p>"), "for {long:?}");
    }
}
