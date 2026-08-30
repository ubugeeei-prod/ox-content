//! A single long line used to cost O(n²).
//!
//! Every scan in the masker and the tokenizer converted each match's byte
//! offset to a character offset by counting from the front of the line, so a
//! line with k matches over n bytes did O(n·k) work. One 1 MiB line took
//! seconds; four of them took a minute. Markdown that is not hard-wrapped —
//! generated API docs, a paragraph written as one line, a long table row —
//! hits this on ordinary content.
//!
//! These tests pin the cost back to linear and pin the diagnostic columns
//! that the conversion feeds, because a cursor that drifts would move every
//! reported column on a multi-byte line.

use std::time::Instant;

use ox_content_markdown_lint::{MarkdownLintOptions, lint_markdown};

fn options(languages: &[&str]) -> MarkdownLintOptions {
    MarkdownLintOptions {
        languages: Some(languages.iter().map(ToString::to_string).collect()),
        ..Default::default()
    }
}

fn one_line(unit: &str, bytes: usize) -> String {
    let mut out = String::with_capacity(bytes + unit.len());
    while out.len() < bytes {
        out.push_str(unit);
    }
    out
}

/// Best of three, so one scheduling stall on a busy runner cannot fail the
/// build. A complexity regression shows up in every run, not just the slowest.
fn fastest_lint(source: &str, options: &MarkdownLintOptions) -> f64 {
    (0..3)
        .map(|_| {
            let started = Instant::now();
            let result = lint_markdown(source, Some(options.clone()));
            let elapsed = started.elapsed().as_secs_f64();
            std::hint::black_box(result.diagnostics.len());
            elapsed
        })
        .fold(f64::INFINITY, f64::min)
}

#[test]
fn a_long_line_costs_linear_time() {
    // Both units pack many matches into few bytes and cost little per byte
    // otherwise, so the quadratic term stands furthest above the noise. Before
    // the fix these grew x13.3 and x19.1 for 4x the input; after it, x3.9 and
    // x4.6. The budget sits between, with roughly 2x of room on either side.
    for (name, unit, languages) in
        [("short urls", "http://a.b/x ", &["en"][..]), ("footnotes", "[^あ] ", &["en", "ja"][..])]
    {
        let options = options(languages);
        // Warm the lazily-built dictionary before timing anything.
        let _ = lint_markdown("warm up", Some(options.clone()));

        let small = fastest_lint(&one_line(unit, 256 * 1024), &options);
        let large = fastest_lint(&one_line(unit, 1024 * 1024), &options);

        // 4x the input. Linear costs about 4x the time; quadratic costs 16x.
        assert!(
            large < small * 8.0,
            "{name}: 1 MiB took {large:.4}s against {small:.4}s for 256 KiB \
             (x{:.1}); linear is about x4, quadratic about x16",
            large / small
        );
    }
}

#[test]
fn columns_stay_character_offsets_on_long_multibyte_lines() {
    // The cursor replaced a full recount per match. If it drifts, every column
    // on a multi-byte line moves — and it would only drift once the line is
    // long enough for the cursor to have advanced, so short cases cannot catch
    // it on their own.
    let options = options(&["en"]);
    let word = "wrrrrong";

    for unit in
        ["日本語のテキストです。", "café naïve ", "<span>日本語</span> ", "a\u{0301}e\u{0301} "]
    {
        for repeats in [1usize, 4, 64, 4096] {
            let prefix = unit.repeat(repeats);
            let source = format!("{prefix}{word}");
            let result = lint_markdown(&source, Some(options.clone()));

            let found = result
                .diagnostics
                .iter()
                .find(|diagnostic| diagnostic.message.contains(word))
                .unwrap_or_else(|| panic!("no diagnostic for {word:?} after {repeats} x {unit:?}"));

            let expected_column = prefix.chars().count() + 1;
            assert_eq!(found.column as usize, expected_column, "column after {repeats} x {unit:?}");
            assert_eq!(
                found.end_column as usize,
                expected_column + word.chars().count(),
                "end column after {repeats} x {unit:?}"
            );
        }
    }
}

#[test]
fn masking_stays_aligned_on_long_multibyte_lines() {
    // The same conversion drives `blank_range`, so a drifting cursor blanks
    // the wrong span: the masked line has to keep the source's length and keep
    // the visible words visible.
    let options = options(&["en", "ja"]);

    for unit in [
        "日本語 [リンク](https://example.com/日本) テキスト ",
        "見出し <span class=\"あ\">日本語</span> あと ",
        "1. 項目 [^脚注] おわり ",
    ] {
        for repeats in [1usize, 4, 64, 2048] {
            let source = unit.repeat(repeats);
            let masked = lint_markdown(&source, Some(options.clone())).masked_document;

            assert_eq!(
                masked.chars().count(),
                source.chars().count(),
                "masked length changed for {repeats} x {unit:?}"
            );
            assert!(
                masked.contains("テキスト") || masked.contains("日本語") || masked.contains("項目"),
                "masking swallowed the visible text of {repeats} x {unit:?}"
            );
        }
    }
}
