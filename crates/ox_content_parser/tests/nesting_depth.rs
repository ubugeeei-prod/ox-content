//! `max_nesting_depth` has to bound every construct that re-enters the
//! parser, not just block quotes.
//!
//! A stack overflow aborts the process instead of unwinding, so no caller
//! can recover from one. Each case here nests far past the cap; if the cap
//! stops applying to that construct the test does not fail, it kills the
//! test binary — which is the point.

use ox_content_allocator::Allocator;
use ox_content_ast::Node;
use ox_content_parser::{ParseError, Parser, ParserOptions};

const OVER_LIMIT: usize = 500;

fn parse_result(source: &str, options: ParserOptions) -> Result<(), ParseError> {
    let allocator = Allocator::new();
    Parser::with_options(&allocator, source, options).parse().map(|_| ())
}

fn assert_too_deep(label: &str, source: &str, options: ParserOptions) {
    match parse_result(source, options) {
        Err(ParseError::NestingTooDeep { max_depth: 100, .. }) => {}
        Err(other) => panic!("{label}: expected NestingTooDeep, got {other}"),
        Ok(()) => panic!("{label}: expected NestingTooDeep, parsed instead"),
    }
}

fn nested_lists(depth: usize) -> String {
    let mut source = String::new();
    for level in 0..depth {
        source.push_str(&" ".repeat(level * 2));
        source.push_str("- item\n");
    }
    source
}

fn nested_quotes(depth: usize) -> String {
    "> ".repeat(depth) + "item\n"
}

fn nested_footnote_definitions(depth: usize) -> String {
    let mut source = String::new();
    for level in 0..depth {
        source.push_str(&" ".repeat(level * 4));
        source.push_str("[^");
        source.push_str(&level.to_string());
        source.push_str("]:\n");
    }
    source.push_str(&" ".repeat(depth * 4));
    source.push_str("body\n");
    source
}

fn nested_quotes_around_lists(depth: usize) -> String {
    let mut source = String::new();
    for level in 0..depth {
        source.push_str(&"> ".repeat(level + 1));
        source.push_str("- item\n");
    }
    source
}

#[test]
fn deeply_nested_lists_fail_closed() {
    assert_too_deep("lists", &nested_lists(OVER_LIMIT), ParserOptions::gfm());
}

#[test]
fn deeply_nested_quotes_fail_closed() {
    assert_too_deep("quotes", &nested_quotes(OVER_LIMIT), ParserOptions::gfm());
}

#[test]
fn deeply_nested_footnote_definitions_fail_closed() {
    assert_too_deep("footnotes", &nested_footnote_definitions(OVER_LIMIT), ParserOptions::gfm());
}

#[test]
fn deeply_nested_quotes_around_lists_fail_closed() {
    assert_too_deep("quoted lists", &nested_quotes_around_lists(OVER_LIMIT), ParserOptions::gfm());
}

#[test]
fn the_cap_applies_without_the_gfm_profile() {
    // `ParserOptions::default()` used to leave nesting unlimited, so the
    // plain-CommonMark path could take the host process down.
    assert_eq!(ParserOptions::default().max_nesting_depth, 100);
    assert_too_deep("plain lists", &nested_lists(OVER_LIMIT), ParserOptions::default());
    assert_too_deep("plain quotes", &nested_quotes(OVER_LIMIT), ParserOptions::default());
}

#[test]
fn the_cap_applies_in_mdx_mode() {
    assert_eq!(ParserOptions::mdx().max_nesting_depth, 100);
    assert_too_deep("mdx lists", &nested_lists(OVER_LIMIT), ParserOptions::mdx());
}

#[test]
fn nesting_within_the_cap_still_parses() {
    let allocator = Allocator::new();
    let source = nested_lists(40);
    let document = Parser::with_options(&allocator, &source, ParserOptions::gfm())
        .parse()
        .expect("40 levels are well inside the cap");

    // Walk the whole chain so the depth is measured, not assumed.
    let mut node = document.children.first().expect("one root list");
    let mut levels = 1;
    while let Node::List(list) = node {
        let item = list.children.first().expect("each list has an item");
        match item.children.iter().find(|child| matches!(child, Node::List(_))) {
            Some(inner) => {
                node = inner;
                levels += 1;
            }
            None => break,
        }
    }
    assert_eq!(levels, 40, "every source level should survive as a nested list");
}

#[test]
fn a_custom_cap_is_honored_for_every_construct() {
    let options = ParserOptions { max_nesting_depth: 3, ..ParserOptions::gfm() };
    for (label, source) in [
        ("lists", nested_lists(12)),
        ("quotes", nested_quotes(12)),
        ("footnotes", nested_footnote_definitions(12)),
    ] {
        match parse_result(&source, options.clone()) {
            Err(ParseError::NestingTooDeep { max_depth: 3, .. }) => {}
            other => panic!("{label}: expected the custom cap to bite, got {other:?}"),
        }
    }
    assert!(parse_result(&nested_lists(2), options.clone()).is_ok());
    assert!(parse_result(&nested_quotes(2), options).is_ok());
}

#[test]
fn zero_still_means_unlimited() {
    // Documented escape hatch: shallow input must keep parsing with it.
    let options = ParserOptions { max_nesting_depth: 0, ..ParserOptions::gfm() };
    assert!(parse_result(&nested_lists(120), options.clone()).is_ok());
    assert!(parse_result(&nested_quotes(120), options).is_ok());
}
