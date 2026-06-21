//! Exact, full-document snapshot tests for the parser.
//!
//! Unlike `edge_cases.rs`, these tests do not poke at individual fields with
//! `assert!`/`contains` — every case captures the entire AST as a
//! deterministic, indented tree string and pins it with `insta::assert_snapshot!`.
//! Spans are included so any structural drift is visible in the diff.

use ox_content_allocator::Allocator;
use ox_content_parser::{Parser, ParserOptions};

#[path = "snapshot_parse/basics.rs"]
mod basics;
#[path = "snapshot_parse/blocks.rs"]
mod blocks;
#[path = "snapshot_parse/html.rs"]
mod html;
#[path = "snapshot_parse/inline.rs"]
mod inline;
#[path = "snapshot_parse/lists.rs"]
mod lists;
#[path = "snapshot_parse/mixed.rs"]
mod mixed;
#[path = "support/pretty.rs"]
mod pretty;
#[path = "snapshot_parse/tables.rs"]
mod tables;

fn parse_to_snapshot(source: &str, options: ParserOptions) -> String {
    let allocator = Allocator::new();
    let doc = Parser::with_options(&allocator, source, options)
        .parse()
        .expect("parser should not fail on snapshot fixtures");
    let mut out = String::new();
    pretty::format_document(&doc, source, &mut out);
    out
}

fn check(name: &str, source: &str, options: ParserOptions) {
    let snapshot = parse_to_snapshot(source, options);
    insta::with_settings!({
        snapshot_path => "snapshots/parser",
        prepend_module_to_snapshot => false,
        description => source.to_string(),
        omit_expression => true,
    }, {
        insta::assert_snapshot!(name, snapshot);
    });
}
