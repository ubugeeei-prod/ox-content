//! Native Rust competitor rows for the JS parse/render benchmark harness
//! (`benchmarks/bundle-size/parse-benchmark.mjs`).
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
mod json;

use std::hint::black_box;
use std::process::ExitCode;

use pulldown_cmark::{html, Parser};

use crate::bench::bench;
use crate::json::{render_json, SuiteResults};

/// Byte-for-byte copy of `sampleMarkdown` in `parse-benchmark-bun.mjs`,
/// including the leading and trailing newline. The JS harness derives
/// throughput from `input.length` (UTF-16 code units), which equals the byte
/// length here because the sample is pure ASCII.
const SAMPLE_MARKDOWN: &str = r#"
# Heading 1

This is a paragraph with **bold** and *italic* text.

## Heading 2

- List item 1
- List item 2
  - Nested item
- List item 3

### Code Block

```javascript
function hello() {
  console.log("Hello, World!");
}
```

> This is a blockquote
> with multiple lines

| Header 1 | Header 2 |
|----------|----------|
| Cell 1   | Cell 2   |
| Cell 3   | Cell 4   |

Here's a [link](https://example.com) and an image: ![alt](image.png)

---

Final paragraph with `inline code` and more text.
"#;

/// `(size name, sample repeats, timed iterations)` in harness order. Matches
/// the JS sizes (small/medium/large/huge = 1/10/100/2150 repeats joined with
/// `"\n\n"`) and the per-size iteration counts (100/50/20/5).
const SIZES: [(&str, usize, u32); 4] =
    [("small", 1, 100), ("medium", 10, 50), ("large", 100, 20), ("huge", 2150, 5)];

enum CliAction {
    Run { runs: u32 },
    Help,
}

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
        Err(message) => {
            eprintln!("{message}");
            ExitCode::FAILURE
        }
    }
}

fn parse_args(args: &[String]) -> Result<CliAction, String> {
    let mut runs = 1u32;
    let mut index = 0;
    while index < args.len() {
        let arg = &args[index];
        if arg == "--runs" {
            index += 1;
            let value =
                args.get(index).ok_or_else(|| "--runs requires a positive integer".to_string())?;
            runs = parse_positive_integer(value)?;
        } else if let Some(value) = arg.strip_prefix("--runs=") {
            runs = parse_positive_integer(value)?;
        } else if arg == "--help" || arg == "-h" {
            return Ok(CliAction::Help);
        } else {
            return Err(format!("Unknown argument: {arg}"));
        }
        index += 1;
    }
    Ok(CliAction::Run { runs })
}

/// Positive-integer parsing with the JS harness' strictness: the canonical
/// re-rendering must equal the input, so `+5`, `05`, or `5x` are rejected.
fn parse_positive_integer(value: &str) -> Result<u32, String> {
    match value.parse::<u32>() {
        Ok(parsed) if parsed >= 1 && parsed.to_string() == value => Ok(parsed),
        _ => Err(format!("--runs requires a positive integer, got `{value}`")),
    }
}

fn print_usage() {
    println!(
        "Usage: ox-content-native-competitors [--runs <count>]

Options:
  --runs <count> Use the median result from repeated runs
  -h, --help     Show this help message"
    );
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
            vec![bench(
                "pulldown-cmark + push_html",
                || {
                    black_box(render_pulldown_html(&content));
                },
                iterations,
                runs,
                bytes,
            )],
        ));
    }
    SuiteResults { parse, render }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pulldown_cmark::{Event, Tag};

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
            [Some("xai-grok-markdown-core (Grok Build)"), Some("pulldown-cmark")]
        );
        let render_names: Vec<_> = render_rows.iter().map(|row| row["name"].as_str()).collect();
        assert_eq!(render_names, [Some("pulldown-cmark + push_html")]);

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
