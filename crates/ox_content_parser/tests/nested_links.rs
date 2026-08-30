//! CommonMark forbids a link inside a link, so `parse_link` parses the
//! bracket text before it can tell whether the outer bracket is a link at
//! all. When it is not, the bracket stays literal and the caller re-scans
//! the same bytes — so every nesting level used to parse its inner text
//! twice, and `[[[[a](/u)](/u)]...` cost 2^depth.
//!
//! These tests pin both halves of the fix: the run has to finish, and it
//! has to keep producing exactly the nesting the spec asks for.

use std::sync::mpsc;
use std::thread;
use std::time::{Duration, Instant};

use ox_content_allocator::Allocator;
use ox_content_parser::{Parser, ParserOptions};
use ox_content_renderer::HtmlRenderer;

/// Generous enough that a slow shared runner never trips it, and far below
/// what the old exponential path needed: at depth 64 that path was 2^44
/// times the depth-20 cost, which already measured 45 ms.
const BUDGET: Duration = Duration::from_secs(15);

fn render(source: &str, options: ParserOptions) -> String {
    let allocator = Allocator::new();
    let document =
        Parser::with_options(&allocator, source, options).parse().expect("source should parse");
    HtmlRenderer::new().render(&document).trim().to_string()
}

/// Parses `source` on a worker thread so a regression fails the suite in
/// bounded time instead of hanging it. Returns how long the parse took.
fn parse_within_budget(source: String) -> Duration {
    let (sender, receiver) = mpsc::channel();
    thread::spawn(move || {
        let started = Instant::now();
        let allocator = Allocator::new();
        let parsed =
            Parser::with_options(&allocator, &source, ParserOptions::gfm()).parse().is_ok();
        let _ = sender.send((parsed, started.elapsed()));
    });
    let (parsed, elapsed) = receiver
        .recv_timeout(BUDGET)
        .expect("nested brackets should parse in bounded time, not exponential time");
    assert!(parsed, "nested brackets should parse to a document");
    elapsed
}

fn nested_inline_links(depth: usize) -> String {
    "[".repeat(depth) + "a" + &"](/u)".repeat(depth)
}

fn nested_reference_links(depth: usize) -> String {
    "[".repeat(depth) + "a" + &"][r]".repeat(depth) + "\n\n[r]: /r\n"
}

#[test]
fn deeply_nested_inline_links_finish_in_bounded_time() {
    parse_within_budget(nested_inline_links(64));
}

#[test]
fn deeply_nested_reference_links_finish_in_bounded_time() {
    parse_within_budget(nested_reference_links(64));
}

#[test]
fn deeply_nested_unclosed_brackets_finish_in_bounded_time() {
    // The closing run is one short, so no bracket ever becomes a link and
    // every level takes the literal-text fallback.
    parse_within_budget("[".repeat(64) + "a" + &"](/u)".repeat(63));
}

#[test]
fn deeply_nested_images_in_links_finish_in_bounded_time() {
    parse_within_budget("[![".repeat(48) + "a" + &"](/i)](/u)".repeat(48));
}

#[test]
fn nested_bracket_cost_grows_far_slower_than_it_doubles() {
    // Each added level used to double the work. Sixteen more levels would
    // therefore cost 65536x; anything under 100x proves the doubling is
    // gone without pinning an absolute time on a shared runner.
    let shallow = parse_within_budget(nested_inline_links(32)).max(Duration::from_micros(1));
    let deep = parse_within_budget(nested_inline_links(48));
    assert!(
        deep < shallow * 100,
        "depth 48 took {deep:?} against {shallow:?} at depth 32; the doubling is back"
    );
}

#[test]
fn only_the_innermost_inline_link_survives() {
    assert_eq!(
        render("[a [b [c](/c)](/b)](/a)", ParserOptions::gfm()),
        "<p>[a [b <a href=\"/c\">c</a>](/b)](/a)</p>"
    );
}

#[test]
fn only_the_innermost_reference_link_survives() {
    assert_eq!(
        render("[a [b [c][ref]][ref]][ref]\n\n[ref]: /r", ParserOptions::gfm()),
        "<p>[a [b <a href=\"/r\">c</a>]<a href=\"/r\">ref</a>]<a href=\"/r\">ref</a></p>"
    );
}

#[test]
fn bracket_text_without_a_link_still_becomes_link_children() {
    // The reused probe nodes have to be the same children a fresh parse
    // would have produced, brackets and emphasis included.
    assert_eq!(
        render("[a [b] *c*](/u)", ParserOptions::gfm()),
        "<p><a href=\"/u\">a [b] <em>c</em></a></p>"
    );
}

#[test]
fn an_image_inside_link_text_is_still_allowed() {
    assert_eq!(
        render("[![alt](img.png)](/u)", ParserOptions::gfm()),
        "<p><a href=\"/u\"><img src=\"img.png\" alt=\"alt\"></a></p>"
    );
}

#[test]
fn identical_bracket_text_is_judged_per_occurrence() {
    // The memoized verdict is keyed by the slice, so repeating the same
    // characters in a different position must not inherit an answer.
    assert_eq!(
        render("[same](/1) and [same](/2) and [[same](/3)](/4)", ParserOptions::gfm()),
        concat!(
            "<p><a href=\"/1\">same</a> and <a href=\"/2\">same</a> ",
            "and [<a href=\"/3\">same</a>](/4)</p>"
        )
    );
}

#[test]
fn nesting_inside_a_block_quote_and_a_list_item_behaves_the_same() {
    // Sub-parsers get their own cache; the verdict must not change.
    assert_eq!(
        render("> [a [b](/b)](/a)", ParserOptions::gfm()),
        "<blockquote>\n<p>[a <a href=\"/b\">b</a>](/a)</p>\n</blockquote>"
    );
    assert_eq!(
        render("- [a [b](/b)](/a)", ParserOptions::gfm()),
        "<ul>\n<li>[a <a href=\"/b\">b</a>](/a)</li>\n</ul>"
    );
}
