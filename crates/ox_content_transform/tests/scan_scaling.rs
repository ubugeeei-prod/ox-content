//! Scans that only report absence after walking to the end of their input.
//!
//! Every fix behind this test is the same shape. A pass looks for something
//! — a closing `]`, a `}`, a `>`, a `<!--`, a `://` — and the search reports
//! "not here" only after reading everything left. Run once that is fine. Run
//! once per element of a run, which is what these passes do, and it is
//! O(n²): the input that triggers it is ordinary text, not a crafted
//! document.
//!
//! Measured before the fixes, on one line at 16 KiB against 64 KiB:
//!
//!     [        unclosed link          x16.8    (parser, #1227)
//!     {        unclosed attribute     x19.2    (attrs)
//!     <        unclosed tag           x14.1    (parser)
//!     *[`      code span + bracket    x11.9    (segments)
//!     @[a](b)  text between links     x19.0    (parser)
//!     a@b.com  one autolink per line  x15.1    (parser)
//!
//! The assertion is a ratio, not a time: 4x the input must not cost 8x the
//! work. Linear is about x4 and quadratic about x16, so the budget sits
//! between with room on both sides for a busy runner.

use std::time::Instant;

use ox_content_transform::transformer::MarkdownTransformer;
use ox_content_transform::*;

/// Shapes that reach a scan through a run of the thing it looks for. Each is
/// repeated to fill the input; those ending in a newline become many lines,
/// the rest become one long line.
const SHAPES: &[(&str, &str)] = &[
    ("unclosed link", "[ "),
    ("unclosed image", "![ "),
    ("unclosed image title", "![a](b \" "),
    ("unclosed image dest", "![a](b "),
    ("unclosed attribute", "{ "),
    ("closed attribute", "{a} "),
    ("attribute block", "text{.a #b c=d} "),
    ("unclosed tag", "< "),
    ("triple angle", "<<< "),
    ("code import", "<<< @/f.ts "),
    ("code span and bracket", "*[` "),
    ("code span", "`code` "),
    ("delimiter soup", "*[`~_$^{ "),
    ("emphasis and link", "@[a](b) \n"),
    ("email per line", "a@b.com \n"),
    ("www per line", "www.example.com \n"),
    ("url per line", "https://example.com/a/b \n"),
    ("bracket per line", "[ \n"),
    ("brace per line", "{ \n"),
    ("angle per line", "< \n"),
];

/// Below this the run is too short for the clock to say anything useful, and
/// too short for anyone to care either way.
const MEASURABLE: f64 = 0.001;

fn all_features() -> TransformOptions {
    TransformOptions {
        gfm: Some(true),
        mdx: Some(true),
        footnotes: Some(true),
        tables: Some(true),
        strikethrough: Some(true),
        autolinks: Some(true),
        autolink_urls: Some(true),
        attributes: Some(AttrsOptions { enabled: Some(true) }),
        math: Some(MathOptions { enabled: Some(true) }),
        abbreviations: Some(AbbreviationsOptions {
            enabled: Some(true),
            terms: None,
            first_use_only: None,
        }),
        wiki_links: Some(WikiLinkOptions { enabled: Some(true), base_url: None }),
        keyboard_keys: Some(KeyboardKeysOptions {
            enabled: Some(true),
            aliases: None,
            style: None,
        }),
        magic_links: Some(MagicLinkOptions {
            enabled: Some(true),
            aliases: None,
            favicon: None,
            favicon_template: None,
            image_overrides: None,
        }),
        containers: Some(ContainerOptions { enabled: Some(true), types: None }),
        images: Some(ImageOptions { enabled: Some(true), lazy: Some(true) }),
        sanitize: Some(SanitizeOptions::default()),
        ..Default::default()
    }
}

fn repeat_to(unit: &str, bytes: usize) -> String {
    let mut out = String::with_capacity(bytes + unit.len());
    while out.len() < bytes {
        out.push_str(unit);
    }
    out
}

/// Best of three, so one scheduling stall on a busy runner cannot fail the
/// build. A complexity regression shows up in every run.
fn fastest(transformer: &MarkdownTransformer, source: &str) -> f64 {
    (0..3)
        .map(|_| {
            let started = Instant::now();
            let result = transformer.transform(source);
            let elapsed = started.elapsed().as_secs_f64();
            std::hint::black_box(result.html.len());
            elapsed
        })
        .fold(f64::INFINITY, f64::min)
}

#[test]
fn no_shape_costs_quadratic_time() {
    let transformer = MarkdownTransformer::from_options(&all_features());
    let mut failures = Vec::new();

    for (name, unit) in SHAPES {
        let small = fastest(&transformer, &repeat_to(unit, 16 * 1024));
        let large = fastest(&transformer, &repeat_to(unit, 64 * 1024));

        if large < MEASURABLE {
            continue;
        }
        let ratio = large / small.max(1e-9);
        if ratio >= 8.0 {
            failures.push(format!(
                "{name} ({unit:?}): 64 KiB took {large:.4}s against {small:.4}s for 16 KiB \
                 (x{ratio:.1}); linear is about x4, quadratic about x16"
            ));
        }
    }

    assert!(failures.is_empty(), "{}", failures.join("\n"));
}
