//! Native Rust competitor rows for the JS parse/render benchmark harness
//! (`tools/benchmarks/bundle-size/parse-benchmark.mjs`).
//!
//! Mirrors the JS harness protocol exactly: the identical sample document and
//! size multipliers, 5 warmup calls, the per-size iteration counts, and
//! `--runs N` with median selection by ops/sec. Emits a single line of JSON on
//! stdout, `{"parse": {<size>: [row, ...]}, "render": {<size>: [row, ...]}}`,
//! where each row is `{"name", "opsPerSec", "avgMs", "throughputMBs",
//! "samples": [...]}`, the same row shape the JS tables consume.

// This standalone binary sits outside the root cargo workspace, but clippy
// still discovers the repository clippy.toml. Its disallowed std types,
// methods, and macros guard allocator-aware hot paths in the parser crates;
// a benchmark runner whose only output is one JSON string on stdout is the
// "explicit output buffer / API boundary" case those rules carve out, so opt
// out wholesale here.
#![allow(clippy::disallowed_macros, clippy::disallowed_methods, clippy::disallowed_types)]

mod bench;
mod cli;
mod conformance;
mod json;
mod sample;

use std::hint::black_box;
use std::process::ExitCode;

use pulldown_cmark::{html, Parser};

use crate::bench::bench;
use crate::cli::{parse_args, print_usage, CliAction};
use crate::json::{render_json, SuiteResults};
use crate::sample::SAMPLE_MARKDOWN;

/// `(size name, sample repeats, timed iterations)` in harness order. Matches
/// the JS sizes (small/medium/large/huge = 1/10/100/2150 repeats joined with
/// `"\n\n"`) and the per-size iteration counts (100/50/20/5).
const SIZES: [(&str, usize, u32); 4] =
    [("small", 1, 100), ("medium", 10, 50), ("large", 100, 20), ("huge", 2150, 5)];

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().skip(1).collect();
    match parse_args(&args) {
        Ok(CliAction::Help) => {
            print_usage();
            ExitCode::SUCCESS
        }
        Ok(CliAction::Run { runs }) => {
            let results = run_benchmarks(&SIZES, runs);
            println!("{}", render_json(&results));
            ExitCode::SUCCESS
        }
        Ok(CliAction::Normalize) => match conformance::normalize_filter() {
            Ok(()) => ExitCode::SUCCESS,
            Err(message) => {
                eprintln!("{message}");
                ExitCode::FAILURE
            }
        },
        Ok(CliAction::Conformance { spec_path }) => match conformance::run(&spec_path) {
            Ok(json) => {
                println!("{json}");
                ExitCode::SUCCESS
            }
            Err(message) => {
                eprintln!("{message}");
                ExitCode::FAILURE
            }
        },
        Err(message) => {
            eprintln!("{message}");
            ExitCode::FAILURE
        }
    }
}

/// Drain Grok Build's exact parse path: `offset_events` (pulldown-cmark with
/// Grok's option set, single-tilde strikethrough demoted).
fn drain_grok_events(input: &str) {
    for event in xai_grok_markdown_core::offset_events(input) {
        black_box(event);
    }
}

/// Drain plain pulldown-cmark under the same option set, so the delta against
/// the Grok row isolates the demotion filter rather than differing options.
fn drain_pulldown_events(input: &str) {
    for event in Parser::new_ext(input, xai_grok_markdown_core::parser_options()) {
        black_box(event);
    }
}

/// Parse + render to HTML the way a typical pulldown-cmark consumer does:
/// a fresh output `String` per call.
fn render_pulldown_html(input: &str) -> String {
    let mut out = String::new();
    html::push_html(&mut out, Parser::new_ext(input, xai_grok_markdown_core::parser_options()));
    out
}

/// ox-content's own core, called directly (no napi boundary, no mdast
/// serialization): a full arena parse producing the AST. This is a heavier
/// job than the event-draining rows above — pulldown streams events without
/// materializing a tree — which is exactly the difference the row exists to
/// show next to `@ox-content/napi`.
fn ox_content_parse(input: &str) {
    let allocator = ox_content_allocator::Allocator::for_source_len(input.len());
    let parser = ox_content_parser::Parser::new(&allocator, input);
    let document = parser.parse().expect("benchmark sample must parse");
    black_box(&document);
}

/// ox-content parse + HTML render with the same defaults the
/// `@ox-content/napi` `parseAndRender` row uses (default parser options,
/// `HtmlRenderer::new()`), minus the napi string hand-off.
fn ox_content_render_html(input: &str) -> String {
    let allocator = ox_content_allocator::Allocator::for_source_len(input.len());
    let parser = ox_content_parser::Parser::new(&allocator, input);
    let document = parser.parse().expect("benchmark sample must parse");
    let mut renderer = ox_content_renderer::HtmlRenderer::new();
    renderer.render(&document)
}

/// Build the arena AST without rendering, matching the native parse row.
fn ferromark_parse(input: &str) {
    let allocator = ferromark::Allocator::for_source_len(input.len());
    let document =
        ferromark::Parser::new(&allocator, input).parse().expect("benchmark sample must parse");
    black_box(&document);
}

/// Parse and render HTML with the default settings.
fn render_ferromark_html(input: &str) -> String {
    ferromark::to_html(input).expect("benchmark sample must render")
}

fn run_benchmarks(sizes: &[(&'static str, usize, u32)], runs: u32) -> SuiteResults {
    let mut parse = Vec::new();
    let mut render = Vec::new();
    for &(size_name, repeats, iterations) in sizes {
        let content = vec![SAMPLE_MARKDOWN; repeats].join("\n\n");
        let bytes = content.len();
        parse.push((
            size_name,
            vec![
                bench(
                    "ox-content (native)",
                    || ox_content_parse(&content),
                    iterations,
                    runs,
                    bytes,
                ),
                bench("ferromark", || ferromark_parse(&content), iterations, runs, bytes),
                bench(
                    "xai-grok-markdown-core (Grok Build)",
                    || drain_grok_events(&content),
                    iterations,
                    runs,
                    bytes,
                ),
                bench(
                    "pulldown-cmark",
                    || drain_pulldown_events(&content),
                    iterations,
                    runs,
                    bytes,
                ),
            ],
        ));
        render.push((
            size_name,
            vec![
                bench(
                    "ox-content (native)",
                    || {
                        black_box(ox_content_render_html(&content));
                    },
                    iterations,
                    runs,
                    bytes,
                ),
                bench(
                    "ferromark",
                    || {
                        black_box(render_ferromark_html(&content));
                    },
                    iterations,
                    runs,
                    bytes,
                ),
                bench(
                    "pulldown-cmark + push_html",
                    || {
                        black_box(render_pulldown_html(&content));
                    },
                    iterations,
                    runs,
                    bytes,
                ),
            ],
        ));
    }
    SuiteResults { parse, render }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pulldown_cmark::{Event, Tag};

    #[test]
    fn arena_reserve_covers_node_dense_sample() {
        let input = vec![SAMPLE_MARKDOWN; 100].join("\n\n");
        let allocator = ox_content_allocator::Allocator::for_source_len(input.len());
        let document = ox_content_parser::Parser::new(&allocator, &input).parse().unwrap();
        assert!(!document.children.is_empty());
        assert!(allocator.bump().allocated_bytes() < input.len() * 12);
    }

    #[test]
    fn renderers_agree_on_benchmark_sample() {
        for repeats in [1, 10] {
            let input = vec![SAMPLE_MARKDOWN; repeats].join("\n\n");
            let arena = ox_content_allocator::Allocator::for_source_len(input.len());
            let document = ox_content_parser::Parser::new(&arena, &input).parse().unwrap();
            let ox = ox_content_renderer::HtmlRenderer::new().render(&document);
            let other = render_ferromark_html(&input);
            assert_eq!(ox, other, "rendered output differs for {repeats} repeats");
        }
    }

    fn grok_strike_starts(text: &str) -> usize {
        xai_grok_markdown_core::offset_events(text)
            .filter(|(event, _)| matches!(event, Event::Start(Tag::Strikethrough)))
            .count()
    }

    #[test]
    fn grok_event_stream_demotes_single_tilde_strikethrough() {
        assert_eq!(grok_strike_starts("~x~"), 0);
        assert_eq!(grok_strike_starts("~~x~~"), 1);
        // Plain pulldown-cmark under the same options DOES strike single-tilde
        // pairs; the demotion is the wrapper's observable difference.
        let plain = Parser::new_ext("~x~", xai_grok_markdown_core::parser_options())
            .filter(|event| matches!(event, Event::Start(Tag::Strikethrough)))
            .count();
        assert_eq!(plain, 1);
    }

    #[test]
    fn ox_content_native_render_matches_sample_expectations() {
        let allocator = ox_content_allocator::Allocator::for_source_len(SAMPLE_MARKDOWN.len());
        let parser = ox_content_parser::Parser::new(&allocator, SAMPLE_MARKDOWN);
        let document = parser.parse().expect("sample must parse");
        let mut renderer = ox_content_renderer::HtmlRenderer::new();
        let html = renderer.render(&document);
        assert!(html.contains("<h1"));
        // Default parser options (matching the @ox-content/napi rows, which
        // also pass no options) leave the GFM table extension off — unlike
        // the pulldown rows, whose Grok option set enables tables. The row
        // comparison note in tools/benchmarks/README.md documents this.
        assert!(!html.contains("<table"));
    }

    #[test]
    fn push_html_renders_sample_non_empty() {
        let rendered = render_pulldown_html(SAMPLE_MARKDOWN);
        assert!(!rendered.is_empty());
        assert!(rendered.contains("<h1"));
        // Tables come from the Grok option set (ENABLE_TABLES), so their
        // presence also pins that the options are actually applied.
        assert!(rendered.contains("<table"));
    }

    #[test]
    fn json_output_is_valid_and_has_expected_shape() {
        let sizes = [("tiny", 1, 1)];
        let json = render_json(&run_benchmarks(&sizes, 2));
        let value: serde_json::Value =
            serde_json::from_str(&json).expect("benchmark output must be valid JSON");

        let parse_rows = value["parse"]["tiny"].as_array().expect("parse rows");
        let render_rows = value["render"]["tiny"].as_array().expect("render rows");
        let parse_names: Vec<_> = parse_rows.iter().map(|row| row["name"].as_str()).collect();
        assert_eq!(
            parse_names,
            [
                Some("ox-content (native)"),
                Some("ferromark"),
                Some("xai-grok-markdown-core (Grok Build)"),
                Some("pulldown-cmark")
            ]
        );
        let render_names: Vec<_> = render_rows.iter().map(|row| row["name"].as_str()).collect();
        assert_eq!(
            render_names,
            [Some("ox-content (native)"), Some("ferromark"), Some("pulldown-cmark + push_html")]
        );

        for row in parse_rows.iter().chain(render_rows) {
            for field in ["opsPerSec", "avgMs", "throughputMBs"] {
                let number = row[field].as_f64().expect("numeric field");
                assert!(number.is_finite() && number > 0.0, "{field} must be a positive number");
            }
            let samples = row["samples"].as_array().expect("samples array");
            assert_eq!(samples.len(), 2, "one sample per run");
            for sample in samples {
                assert!(sample["opsPerSec"].as_f64().is_some());
            }
        }
    }

    #[test]
    fn sample_matches_js_harness_byte_for_byte() {
        // The whole protocol hangs on both harnesses timing the same bytes:
        // extract the template literal from the sibling Bun script and compare
        // it to our constant after undoing its only escape (`\`` -> `` ` ``).
        let js = std::fs::read_to_string(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../bundle-size/parse-benchmark-bun.mjs"
        ))
        .expect("sibling parse-benchmark-bun.mjs must exist");
        let start_marker = "const sampleMarkdown = `";
        let start = js.find(start_marker).expect("sample template start") + start_marker.len();
        // Inside the template every backtick is escaped, so the first raw
        // "`;" is the closing delimiter.
        let end = start + js[start..].find("`;").expect("sample template end");
        let sample = js[start..end].replace("\\`", "`");
        assert_eq!(sample, SAMPLE_MARKDOWN);
        assert!(SAMPLE_MARKDOWN.is_ascii(), "byte length must equal JS string length");
    }
}
