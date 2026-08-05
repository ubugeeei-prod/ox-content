//! CommonMark conformance scoring for the native competitor engines, plus the
//! normalization filter the JS runner scores its own engines through.
//!
//! Every engine is compared to the spec after both sides pass through
//! `normalize_html`, the same normalizer the in-repo conformance suite uses. A
//! byte-exact comparison would not answer the question the benchmark tables
//! ask. It fails on rendering-irrelevant serialization choices — entity
//! spelling, attribute quoting, `<br />` vs `<br>`, whitespace between block
//! tags — and on the slug `id` ox-content adds to every heading. Scoring those
//! as conformance gaps would rank engines by HTML spelling rather than by
//! whether they implement CommonMark.
//!
//! The normalizer is `#[path]`-included from the renderer's test support
//! directory, the same way `spec_commonmark.rs`, `spec_gfm.rs`, and
//! `stress_mutations.rs` include it. Sharing the file is the point: the number
//! published in the benchmark tables has to be produced by the same rules the
//! conformance suite is reviewed against.

#[path = "../../../crates/ox_content_renderer/tests/spec_support/normalize.rs"]
mod normalize;

use std::fmt::Write as _;
use std::io::{Read as _, Write as _};

use normalize::normalize_html;
use ox_content_allocator::Allocator;
use ox_content_parser::{Parser, ParserOptions};
use ox_content_renderer::{HtmlRenderer, HtmlRendererOptions};
use pulldown_cmark::{html, Parser as PulldownParser};

/// One conformance example extracted from a spec fixture.
struct SpecExample {
    markdown: String,
    html: String,
}

/// A named engine under test: Markdown in, HTML out.
type Engine = (&'static str, fn(&str) -> String);

const FENCE: &str = "````````````````````````````````";

/// Mirrors the spec loaders in `spec_support/spec_txt.rs` and `spec-txt.mjs`.
fn parse_spec(text: &str) -> Vec<SpecExample> {
    let mut examples = Vec::new();
    let mut lines = text.lines();

    while let Some(line) = lines.next() {
        let is_example_fence = line
            .strip_prefix(FENCE)
            .is_some_and(|rest| rest.trim() == "example" || rest.trim().starts_with("example "));
        if !is_example_fence {
            continue;
        }

        let mut markdown = String::new();
        let mut html = String::new();
        let mut in_html = false;
        for body_line in lines.by_ref() {
            if body_line.starts_with(FENCE) {
                break;
            }
            if !in_html && body_line == "." {
                in_html = true;
                continue;
            }
            let target = if in_html { &mut html } else { &mut markdown };
            target.push_str(&body_line.replace('→', "\t"));
            target.push('\n');
        }

        examples.push(SpecExample { markdown, html });
    }

    examples
}

/// Renders with ox-content's core (non-GFM) profile.
///
/// URL autolinking of plain text is a builtin rather than CommonMark behavior,
/// so it is turned off — the same adjustment the conformance suite makes.
fn render_ox_content(markdown: &str) -> String {
    let allocator = Allocator::new();
    let parser = Parser::with_options(&allocator, markdown, ParserOptions::default());
    let Ok(document) = parser.parse() else {
        return String::new();
    };
    let mut options = HtmlRendererOptions::new();
    options.autolink_urls = false;
    HtmlRenderer::with_options(options).render(&document)
}

fn render_pulldown(markdown: &str) -> String {
    let mut output = String::new();
    html::push_html(&mut output, PulldownParser::new(markdown));
    output
}

/// Renders through Grok Build's option set, the same configuration its
/// benchmark rows are measured under. Those options turn on GFM extensions, so
/// this scores the engine as the benchmark actually runs it rather than under a
/// CommonMark-only configuration it does not expose.
fn render_grok(markdown: &str) -> String {
    let mut output = String::new();
    html::push_html(
        &mut output,
        PulldownParser::new_ext(markdown, xai_grok_markdown_core::parser_options()),
    );
    output
}

/// Scores every native engine and emits `{"results":[{name,passed,total}, ...]}`.
///
/// A panicking engine scores the example as a failure instead of aborting:
/// refusing to parse an input the spec defines is a conformance result, not a
/// harness error.
pub fn run(spec_path: &str) -> Result<String, String> {
    let text = std::fs::read_to_string(spec_path)
        .map_err(|error| format!("failed to read {spec_path}: {error}"))?;
    let examples = parse_spec(&text);
    let total = examples.len();
    let expected: Vec<String> =
        examples.iter().map(|example| normalize_html(&example.html)).collect();

    let engines: [Engine; 3] = [
        ("ox-content (native)", render_ox_content),
        ("pulldown-cmark", render_pulldown),
        ("xai-grok-markdown-core (Grok Build)", render_grok),
    ];

    let mut out = String::from("{\"results\":[");
    for (index, (name, render)) in engines.into_iter().enumerate() {
        let passed = examples
            .iter()
            .zip(&expected)
            .filter(|(example, expected)| {
                let markdown = example.markdown.clone();
                std::panic::catch_unwind(move || normalize_html(&render(&markdown)))
                    .is_ok_and(|actual| actual == **expected)
            })
            .count();

        if index > 0 {
            out.push(',');
        }
        let _ = write!(out, "{{\"name\":\"{name}\",\"passed\":{passed},\"total\":{total}}}");
    }
    out.push_str("]}");

    Ok(out)
}

/// Normalization filter for the JS conformance runner.
///
/// Reads a stream of `<byte-length>\n<bytes>` records from stdin and writes the
/// normalized form of each back in the same framing. Length prefixes keep the
/// protocol independent of the HTML content, which contains newlines and every
/// kind of quote. Scoring stays on the JS side; only the equivalence rule
/// crosses the process boundary, so both halves of the sweep are judged
/// identically.
pub fn normalize_filter() -> Result<(), String> {
    let mut input = String::new();
    std::io::stdin()
        .read_to_string(&mut input)
        .map_err(|error| format!("failed to read stdin: {error}"))?;

    let stdout = std::io::stdout();
    let mut out = std::io::BufWriter::new(stdout.lock());
    let bytes = input.as_bytes();
    let mut cursor = 0;

    while cursor < bytes.len() {
        let Some(newline) = input[cursor..].find('\n') else {
            break;
        };
        let header_end = cursor + newline;
        let length: usize = input[cursor..header_end]
            .trim()
            .parse()
            .map_err(|_| format!("invalid record length: {}", &input[cursor..header_end]))?;

        let body_start = header_end + 1;
        let body_end = body_start + length;
        if body_end > bytes.len() {
            return Err(String::from("truncated record body"));
        }

        let normalized = normalize_html(&input[body_start..body_end]);
        writeln!(out, "{}", normalized.len()).map_err(|error| error.to_string())?;
        out.write_all(normalized.as_bytes()).map_err(|error| error.to_string())?;

        cursor = body_end;
    }

    out.flush().map_err(|error| error.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    const SPEC_PATH: &str = concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../crates/ox_content_renderer/tests/spec_fixtures/commonmark-0.31.2-spec.txt"
    );

    #[test]
    fn parses_examples_and_decodes_tabs() {
        let spec = format!("{FENCE} example\na→b\n.\n<p>a\tb</p>\n{FENCE}\n");
        let examples = parse_spec(&spec);
        assert_eq!(examples.len(), 1);
        assert_eq!(examples[0].markdown, "a\tb\n");
        assert_eq!(examples[0].html, "<p>a\tb</p>\n");
    }

    #[test]
    fn reads_the_whole_vendored_spec() {
        let text = std::fs::read_to_string(SPEC_PATH).expect("vendored spec");
        assert_eq!(parse_spec(&text).len(), 652);
    }

    #[test]
    fn scores_every_engine_over_the_vendored_spec() {
        let json = run(SPEC_PATH).expect("conformance run");
        let value: serde_json::Value = serde_json::from_str(&json).expect("valid JSON");
        let results = value["results"].as_array().expect("results array");

        assert_eq!(results.len(), 3);
        for result in results {
            let passed = result["passed"].as_u64().expect("passed count");
            let total = result["total"].as_u64().expect("total count");
            assert_eq!(total, 652);
            assert!(passed <= total);
        }
    }

    #[test]
    fn ox_content_core_mode_matches_every_example() {
        // The claim the docs make about ox-content's own conformance, checked
        // through the same path the published table numbers come from.
        let json = run(SPEC_PATH).expect("conformance run");
        let value: serde_json::Value = serde_json::from_str(&json).expect("valid JSON");
        let ox = value["results"]
            .as_array()
            .expect("results array")
            .iter()
            .find(|row| row["name"] == "ox-content (native)")
            .expect("ox-content row");

        assert_eq!(ox["passed"], ox["total"]);
    }

    #[test]
    fn normalizer_treats_serialization_differences_as_equal() {
        assert_eq!(normalize_html("<p>a<br></p>"), normalize_html("<p>a<br /></p>"));
        assert_ne!(normalize_html("<p>a</p>"), normalize_html("<p>b</p>"));
    }
}
