//! An opening `$` is math only when something closes it, and each opener
//! found that out by scanning forward on its own — to the end of the block
//! for an inline `$`, to the end of the source for a `$$` line. A run of
//! openers with nothing to close them therefore cost O(n²): 125 KB of `$a `
//! took 3.6 s and of `$$a` lines 19 s. That is ordinary text: prices (`$5
//! and $10`), shell variables, a template placeholder.
//!
//! These tests pin the cost back to linear, pin the output at the open and
//! close rules, and hold the remembered searches to the old scan — kept in
//! `support/inline_math_reference.rs` — over generated text.

use std::fmt::Write as _;

use ox_content_allocator::Allocator;
use ox_content_ast::Node;
use ox_content_parser::{Parser, ParserOptions};
use ox_content_renderer::HtmlRenderer;

#[path = "support/inline_math_reference.rs"]
mod reference;
#[path = "support/scaling.rs"]
mod scaling;

use scaling::Part::{self, Fill, Once};

fn math() -> ParserOptions {
    ParserOptions { math: true, ..ParserOptions::default() }
}

fn assert_linear(shapes: &[&[Part]]) {
    scaling::assert_linear(&math(), shapes);
}

#[test]
fn openers_with_nothing_to_close_them_cost_linear_time() {
    // No `$` here can close: each follows a space, is escaped, is followed
    // by a digit, or is one dollar short of its opener.
    assert_linear(&[
        &[Fill("$a ")],
        &[Fill("$$a ")],
        &[Fill("$1 ")],
        &[Fill("$a\\$ ")],
        &[Fill("$$a "), Once("x$$5")],
        &[Fill("a price of $5 or $10 is fine ")],
    ]);
}

#[test]
fn closers_hidden_in_code_spans_cost_linear_time() {
    // A `$` that could close sits inside a code span for every opener, so
    // "is there a closing `$` further on" is true and still no use. In the
    // last shape the escaped backticks make the openers alternate between
    // two readings of where the code spans are.
    assert_linear(&[
        &[Fill("$a` ")],
        &[Fill("$a `b$` ")],
        &[Fill("$$a `b$$` ")],
        &[Fill("$a "), Once("`x$`")],
        &[Fill("$a \\` "), Once("``b$``")],
    ]);
}

#[test]
fn digit_prefixed_openers_that_stay_literal_cost_linear_time() {
    // `$5 ... x$` closes, but only counts as math with a `*` or `_` inside:
    // the closer is found and the opener stays literal all the same.
    assert_linear(&[
        &[Fill("$1 "), Once("x$")],
        &[Fill("$1 "), Once("x$ *")],
        &[Fill("$$1 "), Once("x$$ *")],
    ]);
}

#[test]
fn a_run_of_dollars_costs_linear_time() {
    assert_linear(&[&[Once("a "), Fill("$")], &[Once("a $x"), Fill("$"), Once("9")]]);
}

#[test]
fn math_that_does_close_stays_linear() {
    assert_linear(&[
        &[Fill("$a$ ")],
        &[Fill("a$ ")],
        &[Fill("$a $1")],
        &[Once("x $"), Fill("a $ "), Once("b$")],
        &[Fill("$$\n")],
    ]);
}

#[test]
fn display_openers_with_nothing_to_close_them_cost_linear_time() {
    // As a paragraph's continuation lines, as paragraphs of their own, as
    // lines that would interrupt a paragraph, and inside a block quote.
    assert_linear(&[
        &[Fill("$$a\n")],
        &[Fill("$$a\n\n")],
        &[Fill("a\n$$a\n")],
        &[Fill("$$ a $ \n")],
        &[Fill("> $$a\n")],
    ]);
}

fn render(source: &str) -> String {
    let allocator = Allocator::new();
    let document =
        Parser::with_options(&allocator, source, math()).parse().expect("source should parse");
    HtmlRenderer::new().render(&document).trim().to_string()
}

/// Expands the shorthand the expectations below are written in: `⟦tex⟧` is
/// inline math and `⟪tex⟫` a display block, as the renderer writes them.
fn expand(template: &str) -> String {
    let mut out = String::new();
    let mut rest = template;
    while let Some(open) = rest.find(['⟦', '⟪']) {
        let display = rest[open..].starts_with('⟪');
        let inner = open + '⟦'.len_utf8();
        let close =
            inner + rest[inner..].find(if display { '⟫' } else { '⟧' }).expect("closed shorthand");
        let tex = &rest[inner..close];
        let attribute = tex.replace('\n', "&#10;");
        out.push_str(&rest[..open]);
        let written = if display {
            write!(
                out,
                r#"<div class="ox-math ox-math-block" data-ox-tex="{attribute}"><math display="block"><mtext>{tex}</mtext></math></div>"#
            )
        } else {
            write!(
                out,
                r#"<span class="ox-math ox-math-inline" data-ox-tex="{attribute}"><math><mtext>{tex}</mtext></math></span>"#
            )
        };
        written.expect("writing to a string cannot fail");
        rest = &rest[close + '⟧'.len_utf8()..];
    }
    out.push_str(rest);
    out
}

#[test]
fn inline_math_opens_and_closes_as_it_did() {
    for (source, expected) in [
        // What may open and what may close.
        ("$a$", "<p>⟦a⟧</p>"),
        ("$a$ $b$", "<p>⟦a⟧ ⟦b⟧</p>"),
        ("$a $", "<p>$a $</p>"),
        ("$ a$", "<p>$ a$</p>"),
        ("$a$1", "<p>$a$1</p>"),
        ("1$a$", "<p>1$a$</p>"),
        ("a $ b $ c", "<p>a $ b $ c</p>"),
        ("$a", "<p>$a</p>"),
        ("a$", "<p>a$</p>"),
        ("$", "<p>$</p>"),
        ("x $$", "<p>x $$</p>"),
        // A closer needs as many dollars as its opener; only that many are
        // read from a longer run.
        ("$$a$", "<p>$$a$</p>"),
        ("$a$$", "<p>⟦a⟧$</p>"),
        ("x $$a $$b$$", "<p>x ⟦a $$b⟧</p>"),
        ("x $$ a $$", "<p>x $$ a $$</p>"),
        ("a $$$ b", "<p>a $$$ b</p>"),
        ("a $$$$ b", "<p>a ⟦⟧ b</p>"),
        ("a $$$$$ b", "<p>a ⟦⟧$ b</p>"),
        ("a $$$$1 b", "<p>a $$$$1 b</p>"),
        // Escapes and code spans between opener and closer.
        ("\\$a$", "<p>$a$</p>"),
        ("$a\\$ b$", "<p>⟦a\\$ b⟧</p>"),
        ("$a\\\\$ b$", "<p>⟦a\\\\⟧ b$</p>"),
        ("$a `$` b$", "<p>⟦a `$` b⟧</p>"),
        ("$a ` $b$ ` c$", "<p>⟦a ` $b$ ` c⟧</p>"),
        ("$a \\` b$ ` c$", "<p>⟦a \\` b$ ` c⟧</p>"),
        ("$a `b", "<p>$a `b</p>"),
        ("`$a$`", "<p><code>$a$</code></p>"),
        // A digit-prefixed opener is math only around a `*` or `_`.
        ("$1$", "<p>$1$</p>"),
        ("$1*2$", "<p>⟦1*2⟧</p>"),
        ("$5 and $10", "<p>$5 and $10</p>"),
        ("$5 and x$", "<p>$5 and x$</p>"),
        ("$5 *and* x$", "<p>⟦5 *and* x⟧</p>"),
        ("$5 and x$ *y*", "<p>$5 and x$ <em>y</em></p>"),
    ] {
        assert_eq!(render(source), expand(expected), "for {source:?}");
    }
}

#[test]
fn display_math_opens_and_closes_as_it_did() {
    for (source, expected) in [
        ("$$a$$", "⟪a⟫"),
        ("$$ a $$", "⟪ a ⟫"),
        ("$$5 *and* x$$", "⟪5 *and* x⟫"),
        ("$$\na\n$$", "⟪\na\n⟫"),
        ("$$a\n$$", "⟪a\n⟫"),
        ("$$a$$\n", "⟪a⟫"),
        ("$$a\nb$$ \nc", "⟪a\nb⟫\n<p>c</p>"),
        ("$$a\n$$\t \nb", "⟪a\n⟫\n<p>b</p>"),
        ("   $$a\n   $$", "⟪a\n   ⟫"),
        ("x\n$$\na\n$$", "<p>x</p>\n⟪\na\n⟫"),
        ("a\n$$\nb\n$$\nc", "<p>a</p>\n⟪\nb\n⟫\n<p>c</p>"),
        ("$$\n$$\n$$\n$$", "⟪\n⟫\n⟪\n⟫"),
        ("> $$\n> a\n> $$", "<blockquote>\n⟪\na\n⟫\n</blockquote>"),
        // Only a `$$` that ends its line closes, and not an escaped one.
        ("$$ a $ \n$$", "⟪ a $ \n⟫"),
        ("$$a\nb\n$$c\n$$", "⟪a\nb\n$$c\n⟫"),
        ("$$a\n\\$$\n$$", "⟪a\n\\$$\n⟫"),
        ("$$a\n\\\\$$", "⟪a\n\\\\⟫"),
        // Nothing closes the block: the line is a paragraph, whose `$$` may
        // still open inline math.
        ("$$", "<p>$$</p>"),
        ("$$\na", "<p>$$\na</p>"),
        ("$$a\n$$ c", "<p>$$a\n$$ c</p>"),
        ("$$a\nb $$ c", "<p>$$a\nb $$ c</p>"),
        ("$$a\nb$$ c", "<p>⟦a\nb⟧ c</p>"),
    ] {
        assert_eq!(render(source), expand(expected), "for {source:?}");
    }
}

fn collect_inline_math(children: &[Node<'_>], found: &mut Vec<(usize, usize, String)>) {
    for node in children {
        match node {
            Node::InlineMath(node) => {
                found.push((node.span.start as usize, node.span.end as usize, node.value.into()));
            }
            Node::Emphasis(node) => collect_inline_math(&node.children, found),
            Node::Strong(node) => collect_inline_math(&node.children, found),
            _ => {}
        }
    }
}

#[test]
fn remembered_searches_agree_with_the_scan_they_replaced() {
    const ALPHABET: [&str; 14] =
        ["$", "$$", "a", " ", "1", "\\", "`", "``", "*", "_", "b$", "$1 ", "$a ", "x "];
    let mut random = scaling::Random(0x9E37_79B9_7F4A_7C15);
    // Short strings, where the open and close rules are dense, and long
    // ones, where a search is remembered and reused.
    for round in 0..3400 {
        let tokens = if round < 3000 { 1 + random.below(30) } else { 300 + random.below(600) };
        // The `x` at either end keeps the line a paragraph whose content is
        // the whole line, so spans are offsets into `source`.
        let mut source = String::from("x ");
        for _ in 0..tokens {
            source.push_str(ALPHABET[random.below(ALPHABET.len())]);
        }
        source.push_str(" x");

        let allocator = Allocator::new();
        let document =
            Parser::with_options(&allocator, &source, math()).parse().expect("source should parse");
        let mut actual = Vec::new();
        for block in &document.children {
            if let Node::Paragraph(paragraph) = block {
                collect_inline_math(&paragraph.children, &mut actual);
            }
        }
        assert_eq!(actual, reference::inline_math(&source), "for {source:?}");
    }
}
