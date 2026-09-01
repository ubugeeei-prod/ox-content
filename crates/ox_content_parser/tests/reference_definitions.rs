//! A link reference definition may not contain a blank line, so the parser
//! cuts each candidate at the next one. Scanning for that boundary from
//! every definition made a document that is mostly definitions quadratic;
//! the boundary is now remembered for the run that shares it.
//!
//! These tests pin the shape of the run (the cache must not leak a
//! boundary across a blank line) and the cost of it.

use std::time::{Duration, Instant};

use ox_content_allocator::Allocator;
use ox_content_ast::Node;
use ox_content_parser::{Parser, ParserOptions};
use ox_content_renderer::HtmlRenderer;

fn render(source: &str) -> String {
    let allocator = Allocator::new();
    let document = Parser::with_options(&allocator, source, ParserOptions::gfm())
        .parse()
        .expect("source should parse");
    HtmlRenderer::new().render(&document).trim().to_string()
}

fn definitions(source: &str) -> Vec<(String, String, Option<String>)> {
    let allocator = Allocator::new();
    let document = Parser::with_options(&allocator, source, ParserOptions::gfm())
        .parse()
        .expect("source should parse");
    document
        .children
        .iter()
        .filter_map(|node| match node {
            Node::Definition(definition) => Some((
                definition.identifier.to_string(),
                definition.url.to_string(),
                definition.title.map(str::to_string),
            )),
            _ => None,
        })
        .collect()
}

fn definition_run(count: usize) -> String {
    let mut source = String::new();
    for index in 0..count {
        source.push_str("[r");
        source.push_str(&index.to_string());
        source.push_str("]: /u");
        source.push_str(&index.to_string());
        source.push('\n');
    }
    source
}

fn parse_once(source: &str) {
    let allocator = Allocator::new();
    let parsed = Parser::with_options(&allocator, source, ParserOptions::gfm()).parse();
    assert!(parsed.is_ok(), "source should parse");
}

fn time_parse_batch(source: &str, runs: usize) -> Duration {
    let start = Instant::now();
    for _ in 0..runs {
        parse_once(source);
    }
    start.elapsed()
}

#[derive(Clone, Copy)]
struct TimingSample {
    small_batch: Duration,
    large: Duration,
}

fn fastest_equal_definition_sample(small_source: &str, large_source: &str) -> TimingSample {
    parse_once(small_source);
    parse_once(large_source);

    let mut best = TimingSample { small_batch: Duration::MAX, large: Duration::MAX };
    let mut best_ratio = f64::INFINITY;

    for round in 0..5 {
        let sample = if round % 2 == 0 {
            let small_batch = time_parse_batch(small_source, 4);
            let large = time_parse_batch(large_source, 1);
            TimingSample { small_batch, large }
        } else {
            let large = time_parse_batch(large_source, 1);
            let small_batch = time_parse_batch(small_source, 4);
            TimingSample { small_batch, large }
        };
        let ratio = sample.large.as_secs_f64()
            / sample.small_batch.max(Duration::from_micros(1)).as_secs_f64();
        if ratio < best_ratio {
            best = sample;
            best_ratio = ratio;
        }
    }

    best
}

#[test]
fn every_definition_in_a_long_run_is_collected() {
    let collected = definitions(&definition_run(2_000));
    assert_eq!(collected.len(), 2_000);
    assert_eq!(collected[0], ("r0".into(), "/u0".into(), None));
    assert_eq!(collected[1_999], ("r1999".into(), "/u1999".into(), None));
}

#[test]
fn references_in_a_long_run_still_resolve() {
    let source = definition_run(1_000) + "\n[link][r0] and [link][r999]\n";
    assert_eq!(render(&source), "<p><a href=\"/u0\">link</a> and <a href=\"/u999\">link</a></p>");
}

#[test]
fn a_blank_line_ends_the_remembered_region() {
    // The second run starts past the first boundary, so the cached window
    // must not be reused for it.
    let source = "[a]: /a\n[b]: /b\n\n[c]: /c\n[d]: /d\n\n[x][a][x][b][x][c][x][d]\n";
    assert_eq!(
        definitions(source).iter().map(|(id, url, _)| format!("{id}={url}")).collect::<Vec<_>>(),
        ["a=/a", "b=/b", "c=/c", "d=/d"]
    );
}

#[test]
fn a_definition_after_a_paragraph_gets_a_fresh_region() {
    // Prose between two runs moves the position past the cached window
    // without any definition consuming it.
    let source = "[a]: /a\n\nprose\n\n[b]: /b\n[c]: /c\n\n[x][a] [x][b] [x][c]\n";
    assert_eq!(
        render(source),
        "<p>prose</p>\n<p><a href=\"/a\">x</a> <a href=\"/b\">x</a> <a href=\"/c\">x</a></p>"
    );
}

#[test]
fn a_failed_candidate_does_not_poison_the_definitions_after_it() {
    // `[nope]` has no colon, so the run becomes a paragraph; the real
    // definitions that follow the blank line still have to register.
    let source = "[nope] plain text\n[still]: text\n\n[real]: /real\n\n[x][real]\n";
    assert_eq!(definitions(source), [("real".to_string(), "/real".to_string(), None)]);
    assert!(render(source).contains("<a href=\"/real\">x</a>"));
}

#[test]
fn multiline_definitions_inside_a_run_keep_their_titles() {
    let source = concat!(
        "[a]: /a\n",
        "[b]: /b\n   \"B title\"\n",
        "[c]: /c \"C title\"\n",
        "[d]: /d\n",
        "\n[x][b] [x][c]\n"
    );
    assert_eq!(
        definitions(source),
        [
            ("a".to_string(), "/a".to_string(), None),
            ("b".to_string(), "/b".to_string(), Some("B title".to_string())),
            ("c".to_string(), "/c".to_string(), Some("C title".to_string())),
            ("d".to_string(), "/d".to_string(), None),
        ]
    );
}

#[test]
fn definition_runs_inside_a_block_quote_resolve() {
    // The quote's sub-parser has its own source, so it must not inherit
    // the outer parser's remembered offsets.
    let source = "> [a]: /a\n> [b]: /b\n>\n> [x][a] [y][b]\n";
    assert_eq!(
        render(source),
        "<blockquote>\n<p><a href=\"/a\">x</a> <a href=\"/b\">y</a></p>\n</blockquote>"
    );
}

#[test]
fn a_run_of_definitions_costs_linear_time() {
    // Compare equal bytes of work: four 4,000-definition runs against one
    // 16,000-definition run. Quadratic rescanning makes the large input cost
    // about four times the batch; a linear parser stays close to one.
    let small_source = definition_run(4_000);
    let large_source = definition_run(16_000);
    let sample = fastest_equal_definition_sample(&small_source, &large_source);
    let ratio =
        sample.large.as_secs_f64() / sample.small_batch.max(Duration::from_micros(1)).as_secs_f64();
    assert!(
        ratio < 2.5,
        "16,000 definitions took {large:?} against {small:?} for four 4,000-definition runs \
         (x{ratio:.1}); the rescan is back",
        large = sample.large,
        small = sample.small_batch
    );
}
