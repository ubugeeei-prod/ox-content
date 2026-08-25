//! Hostile inputs must return `Result` errors or a document, never abort.

use std::panic::{AssertUnwindSafe, catch_unwind};

use ox_content_allocator::Allocator;
use ox_content_parser::{ParseError, Parser, ParserOptions};

fn parse_or_err(source: &str, options: ParserOptions) -> Result<(), String> {
    let allocator = Allocator::new();
    Parser::with_options(&allocator, source, options)
        .parse()
        .map(|_| ())
        .map_err(|error| error.to_string())
}

#[test]
fn malformed_markdown_does_not_abort() {
    let options = ParserOptions::gfm();
    let cases = [
        "",
        "*",
        "**",
        "[",
        "](",
        "![",
        "`",
        "```",
        "|",
        "> ",
        "- \n- \n- ",
        "<script>",
        "\\",
        "\u{1F600}***\u{3042}",
        &"> ".repeat(8),
        &format!("{}emphasis*", "*".repeat(32)),
    ];

    for source in cases {
        let outcome = catch_unwind(AssertUnwindSafe(|| parse_or_err(source, options.clone())));
        assert!(outcome.is_ok(), "parser aborted on {source:?}");
    }
}

#[test]
fn nesting_limit_returns_an_error_instead_of_aborting() {
    let source = "> ".repeat(120) + "too deep";
    let allocator = Allocator::new();
    let error = Parser::with_options(&allocator, &source, ParserOptions::gfm())
        .parse()
        .expect_err("deeply nested quotes should fail closed");
    assert!(matches!(error, ParseError::NestingTooDeep { max_depth: 100, .. }));
}
