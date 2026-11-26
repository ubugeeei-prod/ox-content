//! Inline constructs that nest — link text, image alt text, emphasis — used
//! to be parsed by recursing once per level, and a bracket that turned out
//! not to be a link had its text parsed again. A few kilobytes of brackets
//! therefore ran out of stack (which aborts the process instead of
//! unwinding) and ran in cubic time before that.
//!
//! These tests pin the three halves of the fix: deep inline nesting fails
//! with `NestingTooDeep` instead of overflowing, the work grows linearly
//! with the input, and the parse is unchanged for everything in between.

use std::sync::mpsc;
use std::thread;
use std::time::{Duration, Instant};

use ox_content_allocator::Allocator;
use ox_content_parser::{ParseError, Parser, ParserOptions};
use ox_content_renderer::HtmlRenderer;

/// Generous enough that a slow shared runner never trips it, and far below
/// what the old paths needed at these sizes.
const BUDGET: Duration = Duration::from_secs(30);

/// The default main-thread stack of most platforms is 8 MiB, but spawned
/// threads (and many embedders) get 2 MiB. Every case runs on one.
const SMALL_STACK: usize = 2 * 1024 * 1024;

fn render(source: &str, options: ParserOptions) -> String {
    let allocator = Allocator::new();
    let document =
        Parser::with_options(&allocator, source, options).parse().expect("source should parse");
    HtmlRenderer::new().render(&document).trim().to_string()
}

/// Parses on a small-stack worker thread so a regression fails the suite
/// instead of hanging it or taking the test binary down with an overflow
/// that would hide which case caused it. Best of two runs.
fn parse_timed(source: &str, options: ParserOptions) -> (Result<(), String>, Duration) {
    let mut best = (Err("not run".to_owned()), BUDGET);
    for _ in 0..2 {
        let owned = source.to_owned();
        let options = options.clone();
        let (sender, receiver) = mpsc::channel();
        thread::Builder::new()
            .stack_size(SMALL_STACK)
            .spawn(move || {
                let started = Instant::now();
                let allocator = Allocator::new();
                let result = Parser::with_options(&allocator, &owned, options)
                    .parse()
                    .map(|_| ())
                    .map_err(|error| match error {
                        ParseError::NestingTooDeep { .. } => "too deep".to_owned(),
                        other => format!("{other}"),
                    });
                let _ = sender.send((result, started.elapsed()));
            })
            .expect("spawn parser thread");
        let (result, elapsed) =
            receiver.recv_timeout(BUDGET).expect("parse should finish in bounded time");
        if elapsed <= best.1 {
            best = (result, elapsed);
        }
    }
    best
}

/// Below this the larger parse is fast enough that no quadratic path is
/// left — the old paths took seconds to minutes at these sizes — and a
/// ratio between such small times is mostly scheduler noise.
const NOISE_FLOOR: Duration = Duration::from_millis(250);

fn assert_linear(label: &str, small: Duration, large: Duration) {
    let ratio = large.as_secs_f64() / small.as_secs_f64().max(1e-4);
    assert!(
        ratio < 10.0 || large < NOISE_FLOOR,
        "{label}: 4x input took x{ratio:.1} ({small:?} -> {large:?})"
    );
}

fn nested(open: &str, middle: &str, close: &str, depth: usize) -> String {
    open.repeat(depth) + middle + &close.repeat(depth)
}

fn smart() -> ParserOptions {
    ParserOptions { smart_punctuation: true, ..ParserOptions::default() }
}

fn wiki() -> ParserOptions {
    ParserOptions { wiki_links: true, ..ParserOptions::default() }
}

#[test]
fn deeply_nested_inline_constructs_do_not_overflow_the_stack() {
    // Each of these used to abort the process at well under the depth used
    // here. Brackets that never become links are plain text and parse; the
    // rest nest real constructs and fail closed.
    let cases: [(&str, String, ParserOptions, bool); 9] = [
        ("brackets", nested("[", "", "]", 20_000), ParserOptions::default(), true),
        ("footnote brackets", nested("[^", "", "]", 20_000), ParserOptions::gfm(), true),
        ("double brackets", nested("[[", "", "]]", 20_000), wiki(), true),
        ("inline links", nested("[", "a", "](u)", 20_000), ParserOptions::default(), true),
        (
            "reference links",
            nested("[", "a", "][r]", 20_000) + "\n\n[r]: /r",
            ParserOptions::default(),
            true,
        ),
        ("images", nested("![", "a", "](u)", 20_000), ParserOptions::default(), false),
        ("bare images", nested("![", "a", "]", 20_000), ParserOptions::default(), true),
        ("emphasis", nested("*", "a", "*", 50_000), ParserOptions::default(), false),
        ("smart emphasis", nested("*", "a", "*", 50_000), smart(), false),
    ];
    for (label, source, options, parses) in cases {
        let (result, _) = parse_timed(&source, options);
        if parses {
            assert_eq!(result, Ok(()), "{label}");
        } else {
            assert_eq!(result, Err("too deep".to_owned()), "{label}");
        }
    }
}

#[test]
fn nesting_within_the_cap_still_parses() {
    assert_eq!(
        render(&nested("![", "a", "](u)", 3), ParserOptions::default()),
        r#"<p><img src="u" alt="a"></p>"#
    );
    assert!(render(&nested("*a ", "x", " a*", 90), ParserOptions::default()).contains("<em>"));
    let (result, _) = parse_timed(&nested("![", "a", "](u)", 90), ParserOptions::default());
    assert_eq!(result, Ok(()));
}

#[test]
fn the_inline_cap_follows_max_nesting_depth() {
    let tight = ParserOptions { max_nesting_depth: 4, ..ParserOptions::default() };
    assert_eq!(
        parse_timed(&nested("*a ", "x", " a*", 8), tight.clone()).0,
        Err("too deep".to_owned())
    );
    assert_eq!(parse_timed(&nested("*a ", "x", " a*", 2), tight).0, Ok(()));

    // `0` still means unlimited for documents that are merely deep.
    let unlimited = ParserOptions { max_nesting_depth: 0, ..ParserOptions::default() };
    assert_eq!(parse_timed(&nested("*a ", "x", " a*", 400), unlimited).0, Ok(()));
}

#[test]
fn nested_bracket_cost_grows_linearly() {
    // 4x the input. Linear costs about 4x; every one of these was at least
    // quadratic (16x) and most cubic (64x) before.
    type Shape = (&'static str, fn(usize) -> String, ParserOptions);
    let shapes: [Shape; 7] = [
        ("[]", |n| nested("[", "", "]", n), ParserOptions::default()),
        ("[a](u)", |n| nested("[", "a", "](u)", n), ParserOptions::default()),
        ("[a][r]", |n| nested("[", "a", "][a]", n), ParserOptions::default()),
        ("[^]", |n| nested("[^", "", "]", n), ParserOptions::default()),
        ("[[]]", |n| nested("[[", "", "]]", n), ParserOptions::default()),
        ("wiki [[]]", |n| nested("[[", "", "]]", n), wiki()),
        ("![]", |n| nested("![", "a", "]", n), ParserOptions::default()),
    ];
    for (label, build, options) in shapes {
        let (small_result, small) = parse_timed(&build(2_000), options.clone());
        let (large_result, large) = parse_timed(&build(8_000), options);
        assert_eq!(small_result, Ok(()), "{label}");
        assert_eq!(large_result, Ok(()), "{label}");
        assert_linear(label, small, large);
    }
}

#[test]
fn unclosed_link_tails_cost_linear_time() {
    // A `](` or `][` that never closes used to send every bracket in front
    // of it to the end of the content.
    for unit in ["[a](", "[a][", "![a](b", "[a](<", "[a](u \"", "[a]((("] {
        let (_, small) =
            parse_timed(&unit.repeat(32 * 1024 / unit.len()), ParserOptions::default());
        let (_, large) =
            parse_timed(&unit.repeat(128 * 1024 / unit.len()), ParserOptions::default());
        assert_linear(unit, small, large);
    }
}

#[test]
fn only_the_innermost_link_survives_at_any_depth() {
    assert_eq!(
        render("[a [b [c](/c) d](/b) e](/a)", ParserOptions::default()),
        r#"<p>[a [b <a href="/c">c</a> d](/b) e](/a)</p>"#
    );
    assert_eq!(
        render("[*a [b](/b)*](/a)", ParserOptions::default()),
        r#"<p>[<em>a <a href="/b">b</a></em>](/a)</p>"#
    );
    // Emphasis opened inside rejected link text pairs across its `]`.
    assert_eq!(
        render("*[a [b](/b)* c](/a)", ParserOptions::default()),
        r#"<p><em>[a <a href="/b">b</a></em> c](/a)</p>"#
    );
    assert_eq!(
        render("[a ![b [c](/c)](/i)](/a)", ParserOptions::default()),
        r#"<p><a href="/a">a <img src="/i" alt="b c"></a></p>"#
    );
}

#[test]
fn link_text_keeps_its_own_boundaries() {
    // Constructs inside link text cannot see past its `]`, and constructs
    // outside it are not split by it.
    assert_eq!(
        render("[a `b](/u)` c](/v)", ParserOptions::default()),
        r#"<p><a href="/v">a <code>b](/u)</code> c</a></p>"#
    );
    assert_eq!(render("[*a](/u)*", ParserOptions::default()), r#"<p><a href="/u">*a</a>*</p>"#);
    assert_eq!(
        render("[a [b] c](/u)", ParserOptions::default()),
        r#"<p><a href="/u">a [b] c</a></p>"#
    );
}

#[test]
fn wiki_links_follow_the_same_rule() {
    assert_eq!(render("[[Page]]", wiki()), r#"<p><a href="Page">Page</a></p>"#);
    assert_eq!(render("[[Page|a *b*]]", wiki()), r#"<p><a href="Page">a <em>b</em></a></p>"#);
    assert_eq!(render("[[Page|[a](/u)]]", wiki()), r#"<p>[[Page|<a href="/u">a</a>]]</p>"#);
    assert_eq!(render("[[a [[b]] c]]", wiki()), r#"<p><a href="a%20[[b">a [[b</a> c]]</p>"#);
}

#[test]
fn destination_parentheses_nest_up_to_a_limit() {
    let link = |depth: usize| format!("[a]({}u{})", "(".repeat(depth), ")".repeat(depth));
    assert_eq!(
        render(&link(32), ParserOptions::default()),
        format!(r#"<p><a href="{}u{}">a</a></p>"#, "(".repeat(32), ")".repeat(32))
    );
    assert!(!render(&link(33), ParserOptions::default()).contains("<a "));
    assert_eq!(render("[a](b(c)d)", ParserOptions::default()), r#"<p><a href="b(c)d">a</a></p>"#);
}

#[test]
fn angle_autolinks_end_at_the_first_space() {
    const BLANK: &str = r#" target="_blank" rel="noopener noreferrer""#;
    assert_eq!(
        render("<http://a.b> <x y> <a@b.c>", ParserOptions::default()),
        format!(
            r#"<p><a href="http://a.b"{BLANK}>http://a.b</a> <x y> <a href="mailto:a@b.c">a@b.c</a></p>"#
        )
    );
    assert_eq!(
        render("<http://a b>", ParserOptions::default()),
        format!(r#"<p>&lt;<a href="http://a"{BLANK}>http://a</a> b&gt;</p>"#)
    );
    assert_eq!(
        render("<a <http://a.b>", ParserOptions::default()),
        format!(r#"<p>&lt;a <a href="http://a.b"{BLANK}>http://a.b</a></p>"#)
    );
}
