#![no_main]

//! Fuzz target: parser with GFM extensions (tables, task lists, strikethrough,
//! footnotes, autolinks) enabled must never panic.

use libfuzzer_sys::fuzz_target;
use ox_content_allocator::Allocator;
use ox_content_parser::{Parser, ParserOptions};

fuzz_target!(|data: &[u8]| {
    let Ok(source) = std::str::from_utf8(data) else {
        return;
    };
    let allocator = Allocator::new();
    let _ = Parser::with_options(&allocator, source, ParserOptions::gfm()).parse();
});
