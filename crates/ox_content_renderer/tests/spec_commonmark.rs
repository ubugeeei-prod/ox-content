//! CommonMark 0.31.2 conformance suite.
//!
//! Every spec example is parsed and rendered twice — once with default
//! (core) parser options and once with the GFM profile — and the output is
//! compared against the spec's expected HTML after both sides pass through
//! the shared normalizer (see `spec_support/normalize.rs` for what is
//! considered equivalent).
//!
//! Examples that do not conform yet are tracked in
//! `spec_fixtures/commonmark-known-failures.txt`. The test fails when a
//! conforming example regresses **or** when a listed example starts
//! passing, so the file always reflects reality and can only shrink.
//! Regenerate it with:
//!
//! ```text
//! UPDATE_SPEC_BASELINE=1 cargo test -p ox_content_renderer --test spec_commonmark
//! ```

#[path = "spec_support/normalize.rs"]
mod normalize;
#[path = "spec_support/spec_txt.rs"]
mod spec_txt;

use std::fmt::Write as _;

use normalize::normalize_html;
use ox_content_allocator::Allocator;
use ox_content_parser::{Parser, ParserOptions};
use ox_content_renderer::{HtmlRenderer, HtmlRendererOptions};
use spec_txt::{parse_spec, SpecExample};

const SPEC: &str = include_str!("spec_fixtures/commonmark-0.31.2-spec.txt");
const BASELINE: &str = include_str!("spec_fixtures/commonmark-known-failures.txt");
const MODES: [&str; 2] = ["core", "gfm"];

fn parser_options(mode: &str) -> ParserOptions {
    match mode {
        "core" => ParserOptions::default(),
        "gfm" => ParserOptions::gfm(),
        other => panic!("unknown mode {other}"),
    }
}

/// Renderer options for spec comparison: URL autolinking of plain text is
/// not part of CommonMark, so the builtin is turned off. Heading ids and
/// external-link attributes stay on and are stripped by the normalizer.
fn renderer_options() -> HtmlRendererOptions {
    let mut options = HtmlRendererOptions::new();
    options.autolink_urls = false;
    options
}

/// Renders one example, converting parser errors and panics into an
/// inline marker string. A panicking parser must show up as a recorded
/// conformance failure instead of killing the whole suite.
fn render(markdown: &str, mode: &'static str) -> String {
    let markdown = markdown.to_string();
    let result = std::panic::catch_unwind(move || {
        let allocator = Allocator::new();
        let parser = Parser::with_options(&allocator, &markdown, parser_options(mode));
        let parsed = parser.parse();
        let rendered = match parsed {
            Ok(ref document) => HtmlRenderer::with_options(renderer_options()).render(document),
            Err(ref error) => format!("<!-- PARSE ERROR: {error:?} -->"),
        };
        rendered
    });
    result.unwrap_or_else(|panic| {
        let message = panic
            .downcast_ref::<String>()
            .map(String::as_str)
            .or_else(|| panic.downcast_ref::<&str>().copied())
            .unwrap_or("opaque panic payload");
        format!("<!-- PANIC: {message} -->")
    })
}

struct Failure {
    mode: &'static str,
    example: usize,
    section: String,
    markdown: String,
    expected: String,
    actual_raw: String,
}

fn run_all(examples: &[SpecExample]) -> Vec<Failure> {
    // Parser panics are caught and recorded per example; silence the
    // default hook so hundreds of expected backtraces don't drown the
    // actual report. The hook is restored before returning.
    let default_hook = std::panic::take_hook();
    std::panic::set_hook(Box::new(|_| {}));
    let failures = collect_failures(examples);
    std::panic::set_hook(default_hook);
    failures
}

fn collect_failures(examples: &[SpecExample]) -> Vec<Failure> {
    let mut failures = Vec::new();
    for mode in MODES {
        for example in examples {
            let actual_raw = render(&example.markdown, mode);
            if normalize_html(&actual_raw) != normalize_html(&example.html) {
                failures.push(Failure {
                    mode,
                    example: example.number,
                    section: example.section.clone(),
                    markdown: example.markdown.clone(),
                    expected: example.html.clone(),
                    actual_raw,
                });
            }
        }
    }
    failures
}

/// Parses baseline lines of the form `mode example-number [comment...]`.
fn parse_baseline(text: &str) -> Vec<(String, usize)> {
    let mut entries = Vec::new();
    for line in text.lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        let mut parts = line.split_whitespace();
        let mode = parts.next().expect("non-empty line has a first token");
        let number = parts
            .next()
            .and_then(|token| token.parse::<usize>().ok())
            .unwrap_or_else(|| panic!("malformed baseline line: {line:?}"));
        assert!(MODES.contains(&mode), "unknown mode in baseline line: {line:?}");
        entries.push((mode.to_string(), number));
    }
    entries
}

fn baseline_path() -> std::path::PathBuf {
    std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/spec_fixtures/commonmark-known-failures.txt")
}

fn write_baseline(failures: &[Failure]) {
    let mut content = String::from(
        "# CommonMark 0.31.2 examples that ox-content does not render per spec yet.\n\
         # Format: <mode> <example-number> <section>\n\
         # Regenerate: UPDATE_SPEC_BASELINE=1 cargo test -p ox_content_renderer --test spec_commonmark\n",
    );
    for failure in failures {
        let _ = writeln!(
            content,
            "{} {} {}",
            failure.mode,
            failure.example,
            failure.section.replace(' ', "-")
        );
    }
    std::fs::write(baseline_path(), content).expect("baseline file is writable");
}

fn describe(failure: &Failure) -> String {
    let mut message = String::new();
    let _ = writeln!(
        message,
        "\n=== {} mode, example {} ({}) ===",
        failure.mode, failure.example, failure.section
    );
    let _ = writeln!(message, "--- markdown\n{}", failure.markdown);
    let _ = writeln!(message, "--- expected (spec)\n{}", failure.expected);
    let _ = writeln!(message, "--- actual (renderer)\n{}", failure.actual_raw);
    let _ = writeln!(message, "--- expected normalized\n{}", normalize_html(&failure.expected));
    let _ = writeln!(message, "--- actual normalized\n{}", normalize_html(&failure.actual_raw));
    message
}

#[test]
fn commonmark_spec_conformance() {
    let examples = parse_spec(SPEC);
    assert_eq!(examples.len(), 652, "CommonMark 0.31.2 ships 652 examples");

    let failures = run_all(&examples);

    if let Some(path) = std::env::var_os("SPEC_REPORT_PATH") {
        let report: String = failures.iter().map(describe).collect();
        std::fs::write(path, report).expect("report path is writable");
    }

    if std::env::var_os("UPDATE_SPEC_BASELINE").is_some() {
        write_baseline(&failures);
        return;
    }

    let known: Vec<(String, usize)> = parse_baseline(BASELINE);
    let mut unexpected_failures = Vec::new();
    for failure in &failures {
        if !known.iter().any(|(mode, number)| mode == failure.mode && *number == failure.example) {
            unexpected_failures.push(failure);
        }
    }
    let unexpected_passes: Vec<&(String, usize)> = known
        .iter()
        .filter(|(mode, number)| {
            !failures
                .iter()
                .any(|failure| failure.mode == mode.as_str() && failure.example == *number)
        })
        .collect();

    let mut report = String::new();
    if !unexpected_failures.is_empty() {
        let _ = writeln!(
            report,
            "{} example(s) regressed (passing before, failing now):",
            unexpected_failures.len()
        );
        for failure in unexpected_failures.iter().take(8) {
            report.push_str(&describe(failure));
        }
        if unexpected_failures.len() > 8 {
            let _ = writeln!(report, "... and {} more", unexpected_failures.len() - 8);
        }
    }
    if !unexpected_passes.is_empty() {
        let _ = writeln!(
            report,
            "\n{} baseline example(s) now pass — remove them from \
             commonmark-known-failures.txt (or regenerate it):",
            unexpected_passes.len()
        );
        for (mode, number) in &unexpected_passes {
            let _ = writeln!(report, "  {mode} {number}");
        }
    }

    assert!(
        report.is_empty(),
        "CommonMark conformance drifted from the baseline \
         ({} of {} runs currently fail).\n{}",
        failures.len(),
        examples.len() * MODES.len(),
        report
    );
}
