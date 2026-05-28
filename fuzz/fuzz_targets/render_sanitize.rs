#![no_main]

//! Fuzz target: sanitize=true rendering must never panic and must never emit
//! `javascript:` (case-insensitive) into the output. The second property is
//! the actual sanitizer contract — if it ever fails, libFuzzer surfaces a
//! reproducer.

use libfuzzer_sys::fuzz_target;
use ox_content_allocator::Allocator;
use ox_content_parser::{Parser, ParserOptions};
use ox_content_renderer::{HtmlRenderer, HtmlRendererOptions};

fuzz_target!(|data: &[u8]| {
    let Ok(source) = std::str::from_utf8(data) else {
        return;
    };
    let allocator = Allocator::new();
    let Ok(doc) = Parser::with_options(&allocator, source, ParserOptions::gfm()).parse() else {
        return;
    };

    let mut renderer = HtmlRenderer::with_options(HtmlRendererOptions {
        sanitize: true,
        ..HtmlRendererOptions::default()
    });
    let html = renderer.render(&doc);

    let lowered = html.to_ascii_lowercase();
    // Allowed: a literal occurrence inside an escaped sequence like
    // `&lt;javascript:` is fine, what we care about is the dangerous
    // unescaped `href="javascript:` / `src="javascript:` form.
    assert!(
        !lowered.contains("href=\"javascript:"),
        "sanitize emitted href=javascript: href=javascript:; output: {html}"
    );
    assert!(
        !lowered.contains("src=\"javascript:"),
        "sanitize emitted src=javascript: src=javascript:; output: {html}"
    );
});
