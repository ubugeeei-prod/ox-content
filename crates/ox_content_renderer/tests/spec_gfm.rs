//! GFM extension conformance suite (tables, task list items,
//! strikethrough, autolinks).
//!
//! Runs the extension examples extracted from the GitHub Flavored
//! Markdown spec through parse+render with the GFM profile, comparing
//! via the same normalizer as the CommonMark suite. Failures are pinned
//! in `spec_fixtures/gfm-known-failures.txt` with the same ratchet
//! semantics: regressions fail, and so do stale baseline entries.
//! Regenerate with:
//!
//! ```text
//! UPDATE_SPEC_BASELINE=1 cargo test -p ox_content_renderer --test spec_gfm
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

const SPEC: &str = include_str!("spec_fixtures/gfm-extensions-spec.txt");
const BASELINE: &str = include_str!("spec_fixtures/gfm-known-failures.txt");

fn render(markdown: &str, section: &str) -> String {
    let allocator = Allocator::new();
    let mut renderer_options = HtmlRendererOptions::new();
    renderer_options.autolink_urls = false;
    // The tagfilter extension is opt-in on the renderer (raw HTML normally
    // passes through), so enable it for the section that specifies it.
    renderer_options.disallow_raw_html = section.starts_with("Disallowed Raw HTML");
    let parser = Parser::with_options(&allocator, markdown, ParserOptions::gfm());
    let parsed = parser.parse();
    let rendered = match parsed {
        Ok(ref document) => HtmlRenderer::with_options(renderer_options).render(document),
        Err(ref error) => format!("<!-- PARSE ERROR: {error:?} -->"),
    };
    rendered
}

fn failures(examples: &[SpecExample]) -> Vec<(usize, String)> {
    examples
        .iter()
        .filter_map(|example| {
            let actual = render(&example.markdown, &example.section);
            (normalize_html(&actual) != normalize_html(&example.html)).then(|| {
                let mut detail = String::new();
                let _ = writeln!(detail, "=== example {} ({})", example.number, example.section);
                let _ = writeln!(detail, "--- markdown\n{}", example.markdown);
                let _ = writeln!(detail, "--- expected\n{}", example.html);
                let _ = writeln!(detail, "--- actual\n{actual}");
                (example.number, detail)
            })
        })
        .collect()
}

fn baseline_numbers() -> Vec<usize> {
    BASELINE
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty() && !line.starts_with('#'))
        .map(|line| {
            line.split_whitespace()
                .next()
                .and_then(|token| token.parse().ok())
                .unwrap_or_else(|| panic!("malformed baseline line: {line:?}"))
        })
        .collect()
}

#[test]
fn gfm_extension_conformance() {
    let examples = parse_spec(SPEC);
    assert_eq!(examples.len(), 24, "expected the vendored GFM extension examples");

    let failing = failures(&examples);

    if std::env::var_os("UPDATE_SPEC_BASELINE").is_some() {
        let mut content = String::from(
            "# GFM extension examples that ox-content does not render per spec yet.\n\
             # Format: <example-number>\n\
             # Regenerate: UPDATE_SPEC_BASELINE=1 cargo test -p ox_content_renderer --test spec_gfm\n",
        );
        for (number, _) in &failing {
            let _ = writeln!(content, "{number}");
        }
        let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("tests/spec_fixtures/gfm-known-failures.txt");
        std::fs::write(path, content).expect("baseline file is writable");
        return;
    }

    let known = baseline_numbers();
    let mut report = String::new();
    for (number, detail) in &failing {
        if !known.contains(number) {
            let _ = writeln!(report, "regressed:\n{detail}");
        }
    }
    for number in &known {
        if !failing.iter().any(|(failing_number, _)| failing_number == number) {
            let _ = writeln!(
                report,
                "baseline example {number} now passes — remove it from gfm-known-failures.txt"
            );
        }
    }

    assert!(
        report.is_empty(),
        "GFM extension conformance drifted ({} of {} examples fail).\n{}",
        failing.len(),
        examples.len(),
        report
    );
}
