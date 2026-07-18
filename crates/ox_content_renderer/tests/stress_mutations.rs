//! Mutation stress pass over the parse → render pipeline.
//!
//! The spec suites pin *correctness* for well-formed documents; this one
//! guards *robustness* for malformed ones. Every CommonMark spec example
//! is mutated deterministically (truncated at char boundaries, prefixed
//! and suffixed with syntax markers, whitespace-folded) and pushed
//! through the pipeline across the parser/renderer option matrix.
//!
//! Nothing about the output is asserted — the contract is only that the
//! pipeline terminates without panicking. That is worth a dedicated test
//! because the `fuzz/` targets need a nightly toolchain and are never run
//! in CI, so an infinite loop like the whitespace-only-input hang (a
//! two-byte document that spun forever) had no automated guard.
//!
//! The whole batch runs on a worker thread with a wall-clock budget: a
//! hang shows up as the budget expiring, a panic as the thread dying.

#[path = "spec_support/spec_txt.rs"]
mod spec_txt;

use std::sync::mpsc;
use std::time::{Duration, Instant};

use ox_content_allocator::Allocator;
use ox_content_parser::{Parser, ParserOptions};
use ox_content_renderer::{HtmlRenderer, HtmlRendererOptions};
use spec_txt::parse_spec;

const SPEC: &str = include_str!("spec_fixtures/commonmark-0.31.2-spec.txt");

/// Generous enough that a slow debug-build machine never trips it, tight
/// enough that an infinite loop is caught in one test run. The batch
/// takes low single-digit seconds in practice.
const BUDGET: Duration = Duration::from_secs(120);

/// Syntax markers spliced into the seeds. Each opens a construct whose
/// scanner has to cope with the closer being absent or misplaced.
const MARKERS: [&str; 16] =
    ["[", "![", "](", "*", "_", "`", "~~", "<", "&#", "[^", "> ", "- ", "|", "\\", "    ", "\t"];

fn options(variant: u8) -> (ParserOptions, HtmlRendererOptions) {
    let parser = if variant & 1 == 0 { ParserOptions::default() } else { ParserOptions::gfm() };
    let mut renderer = HtmlRendererOptions::new();
    renderer.sanitize = variant & 0b10 != 0;
    renderer.xhtml = variant & 0b100 != 0;
    renderer.convert_md_links = variant & 0b1000 != 0;
    renderer.disallow_raw_html = variant & 0b1_0000 != 0;
    (parser, renderer)
}

fn mutate(source: &str, out: &mut Vec<String>) {
    let len = source.len();
    for cut in [len / 4, len / 2, len * 3 / 4, len.saturating_sub(1)] {
        if source.is_char_boundary(cut) {
            out.push(source[..cut].to_string());
        }
    }
    let middle = len / 2;
    for marker in MARKERS {
        out.push([marker, source].concat());
        out.push([source, marker].concat());
        if source.is_char_boundary(middle) {
            out.push([&source[..middle], marker, &source[middle..]].concat());
        }
    }
    // Folding newlines and spaces reshapes block structure without
    // changing the character inventory.
    out.push(source.replace('\n', ""));
    out.push(source.replace(' ', "\t"));
}

fn run_batch() -> usize {
    let examples = parse_spec(SPEC);
    let mut cases = 0usize;
    let mut inputs = Vec::new();

    for (index, example) in examples.iter().enumerate() {
        inputs.clear();
        inputs.push(example.markdown.clone());
        mutate(&example.markdown, &mut inputs);

        for input in &inputs {
            // Rotate the option matrix by example so every variant is
            // exercised without multiplying the case count by 32.
            let variant = (index % 32) as u8;
            let (parser_options, renderer_options) = options(variant);
            let allocator = Allocator::new();
            let parsed = Parser::with_options(&allocator, input, parser_options).parse();
            if let Ok(ref document) = parsed {
                let _ = HtmlRenderer::with_options(renderer_options).render(document);
            }
            cases += 1;
        }
    }

    cases
}

#[test]
fn mutated_spec_examples_never_panic_or_hang() {
    let (sender, receiver) = mpsc::channel();
    let worker = std::thread::Builder::new()
        // Deeply nested mutations recurse through block parsing; give the
        // worker a roomy stack so this measures the parser, not the
        // default thread stack size.
        .stack_size(32 * 1024 * 1024)
        .spawn(move || {
            let cases = run_batch();
            let _ = sender.send(cases);
        })
        .expect("worker thread spawns");

    let started = Instant::now();
    let cases = receiver.recv_timeout(BUDGET).unwrap_or_else(|_| {
        panic!(
            "parse+render did not finish within {BUDGET:?} — a mutated input is \
             most likely hanging the parser (the batch is normally seconds)"
        )
    });

    // Propagate a panic from the worker with its original message.
    worker.join().expect("no mutated input may panic the pipeline");

    // Assert the batch size rather than printing it (the workspace lints
    // ban stdout/stderr writes): if a future change shrinks the seed set
    // or the mutation table, the coverage loss fails the test instead of
    // passing quietly.
    assert!(
        cases > 30_000,
        "expected the full mutation batch, ran only {cases} cases in {:?}",
        started.elapsed()
    );
}

#[test]
fn whitespace_only_inputs_terminate() {
    // The exact shapes that used to spin forever: a document of spaces
    // and/or tabs with no trailing newline.
    for source in ["  ", "\t", "  \t", "   \t", "    \t", " \t \t", "\t\t"] {
        let allocator = Allocator::new();
        let document = Parser::with_options(&allocator, source, ParserOptions::gfm())
            .parse()
            .expect("whitespace-only input parses");
        let html = HtmlRenderer::new().render(&document);
        assert!(html.trim().is_empty(), "expected empty output for {source:?}, got {html:?}");
    }
}
