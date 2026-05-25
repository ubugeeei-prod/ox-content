#![no_main]

//! Fuzz target: parser with default options must never panic.
//!
//! Inputs are interpreted as UTF-8 only; non-UTF-8 byte sequences are skipped
//! since the parser requires `&str`. Whether `parse()` returns `Ok` or `Err`
//! is irrelevant — the property under test is "no panic on arbitrary text."

use libfuzzer_sys::fuzz_target;
use ox_content_allocator::Allocator;
use ox_content_parser::{Parser, ParserOptions};

fuzz_target!(|data: &[u8]| {
    let Ok(source) = std::str::from_utf8(data) else {
        return;
    };
    let allocator = Allocator::new();
    let _ = Parser::with_options(&allocator, source, ParserOptions::default()).parse();
});
