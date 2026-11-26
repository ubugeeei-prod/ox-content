//! The GFM autolink post-pass searched each text node again after every
//! link it found, and validated every candidate from scratch each time. Four
//! shapes of ordinary-looking text turned that into O(n²):
//!
//! - `www.a.b_` repeated: one run of domain bytes, a `www.` every eight
//!   bytes, each walking the whole run to be rejected for its last segment.
//!   125 KB took 5.4 s.
//! - A link followed by a run of `)` or `;`: every byte trimmed recounted
//!   the parentheses, or searched back for an `&`, over the whole link.
//! - `www.a.b@c.d(` repeated: the `www.` URL was measured to the end of the
//!   node before losing the tie to the email that starts on the same byte.
//! - `www.a.b *x* ` repeated: each link was `insert`ed into the paragraph,
//!   shifting every node behind it.
//!
//! These tests pin the cost back to linear, pin the output around the code
//! that changed, and hold the rewritten search to the old one — kept in
//! `support/gfm_autolink_reference.rs` — over generated text.

use ox_content_allocator::Allocator;
use ox_content_ast::Node;
use ox_content_parser::{Parser, ParserOptions};
use ox_content_renderer::HtmlRenderer;

#[path = "support/gfm_autolink_cases.rs"]
mod cases;
#[path = "support/pretty.rs"]
mod pretty;
#[path = "support/gfm_autolink_reference.rs"]
mod reference;
#[path = "support/scaling.rs"]
mod scaling;

use scaling::Part::{self, Fill, Once};

fn assert_linear(shapes: &[&[Part]]) {
    scaling::assert_linear(&ParserOptions::gfm(), shapes);
}

#[test]
fn candidates_sharing_a_domain_run_cost_linear_time() {
    assert_linear(&[
        &[Fill("www.a.b_")],
        &[Fill("www._")],
        &[Fill("www.a_b.c_")],
        &[Once("x"), Fill("www.a.b_"), Once("www.c.d")],
    ]);
}

#[test]
fn trailing_punctuation_is_trimmed_in_linear_time() {
    assert_linear(&[
        &[Once("www.a.b"), Fill(")")],
        &[Once("www.a.b("), Fill(")")],
        &[Once("(www.a.b"), Fill(")")],
        &[Once("http://a.b/"), Fill(")")],
        &[Once("www.a.b"), Fill(";")],
        &[Once("www.a.b&"), Fill(";")],
        &[Once("www.a.b/&"), Fill("a"), Fill(";")],
        &[Once("www.a.b/"), Fill("&a"), Fill(";")],
        &[Once("www.a.b/"), Fill("&a;)")],
        &[Once("www.a.b/"), Fill("(")],
        &[Once("www.a.b"), Fill(".")],
    ]);
}

#[test]
fn a_url_that_loses_to_an_email_is_not_measured() {
    assert_linear(&[&[Fill("www.a.b@c.d(")], &[Fill("www.a.b@c.d\"")], &[Fill("www.a.b@c.d'")]]);
}

#[test]
fn links_in_front_of_other_nodes_cost_linear_time() {
    assert_linear(&[
        &[Fill("www.a.b *x* ")],
        &[Fill("a@b.co `c` ")],
        &[Fill("a@b.co ~~c~~ ")],
        &[Fill("http://a.b [x](y) ")],
        &[Once("*"), Fill("www.a.b `c` "), Once("*")],
    ]);
}

#[test]
fn links_and_rejected_needles_stay_linear() {
    // Linear before as well: the old search bounded these with a growing
    // window. They are here so the scan that replaced it keeps the bound.
    assert_linear(&[
        &[Fill("www.a.b ")],
        &[Fill("a@b.co ")],
        &[Fill("http://a.b ")],
        &[Fill("a@ ")],
        &[Fill("@b.c ")],
        &[Fill("://x ")],
        &[Fill("http:// ")],
        &[Fill("www. ")],
        &[Fill("www.a ")],
        &[Fill("xwww.a.b ")],
        &[Fill("a_b@c.d_ ")],
        &[Fill("a@")],
        &[Fill("a."), Once("@b.c")],
        &[Fill("@a.b")],
        &[Once("a@b"), Fill(".c"), Once("_")],
        // A rejected email far behind many links: neither its local part
        // nor its domain may be walked again for each link in front of it.
        &[Fill("www.a.b "), Once("a@b"), Fill(".c"), Once("_")],
        &[Fill("www.a.b "), Fill("x"), Once("@")],
        &[Fill("http://a.b "), Once("www.a"), Fill(".b"), Once("_")],
    ]);
}

fn parse<'a>(allocator: &'a Allocator, source: &'a str) -> ox_content_ast::Document<'a> {
    Parser::with_options(allocator, source, ParserOptions::gfm())
        .parse()
        .expect("source should parse")
}

/// Renders with the external-link attributes dropped, so an expected value
/// reads as the link it describes.
fn render(source: &str) -> String {
    let allocator = Allocator::new();
    let html = HtmlRenderer::new().render(&parse(&allocator, source));
    html.trim().replace(r#" target="_blank" rel="noopener noreferrer""#, "")
}

fn tree(source: &str) -> String {
    let allocator = Allocator::new();
    let mut out = String::new();
    pretty::format_document(&parse(&allocator, source), source, &mut out);
    out
}

#[test]
fn autolinks_render_as_they_did() {
    for (source, expected) in cases::CASES {
        assert_eq!(render(source), *expected, "for {source:?}");
    }
}

#[test]
fn split_text_nodes_keep_their_shape() {
    // The renderer links bare `scheme://` text on its own, so HTML cannot
    // show which of these the parser accepted, nor where it cut the text:
    // pieces and spans have to come out exactly as the per-link splitting
    // left them.
    for (source, expected) in cases::TREES {
        assert_eq!(tree(source), *expected, "for {source:?}");
    }
}

#[test]
fn the_scan_agrees_with_the_search_it_replaced() {
    // No `[`, `]` or `>`: every link in the result is an autolink.
    const ALPHABET: [&str; 32] = [
        "www.", "a", "b", ".", "_", "-", "@", "(", ")", ";", "&", "http://", "https://", "ftp://",
        " ", "<", ",", "+", ":", "/", "*", "~", "'", "\"", "?", "!", "c.d", "1", "。", "é", "`",
        "x@y.z",
    ];
    let mut random = scaling::Random(0x9E37_79B9_7F4A_7C15);
    // Mostly short strings, where the edge cases are dense, and some long
    // enough to cross the old search's 512-byte window several times.
    for round in 0..4200 {
        let tokens = if round < 4000 { 1 + random.below(24) } else { 200 + random.below(400) };
        // The `@` keeps the pass switched on for every input.
        let mut source = String::from("@ ");
        for _ in 0..tokens {
            source.push_str(ALPHABET[random.below(ALPHABET.len())]);
        }

        let allocator = Allocator::new();
        for block in &parse(&allocator, &source).children {
            if let Node::Paragraph(paragraph) = block {
                check_inlines(&paragraph.children, &source);
            }
        }
    }
}

fn check_inlines(children: &[Node<'_>], source: &str) {
    let mut index = 0;
    while index < children.len() {
        let run = children[index..]
            .iter()
            .take_while(|node| matches!(node, Node::Text(_) | Node::Link(_)))
            .count();
        if run > 0 {
            check_run(&children[index..index + run], source);
            index += run;
            continue;
        }
        match &children[index] {
            Node::Emphasis(node) => check_inlines(&node.children, source),
            Node::Strong(node) => check_inlines(&node.children, source),
            Node::Delete(node) => check_inlines(&node.children, source),
            _ => {}
        }
        index += 1;
    }
}

/// `run` is what the pass made of one text node: the node's value is the
/// pieces joined, and its span starts where the first piece does.
fn check_run(run: &[Node<'_>], source: &str) {
    let mut value = String::new();
    let mut actual = Vec::new();
    let (mut start, mut end) = (None, 0);
    for node in run {
        let (line, span) = match node {
            Node::Text(text) => {
                value.push_str(text.value);
                (format!("text {:?}", text.value), text.span)
            }
            Node::Link(link) => {
                let [Node::Text(text)] = &link.children[..] else {
                    panic!("an autolink holds one text node, for {source:?}");
                };
                value.push_str(text.value);
                assert_eq!(text.span, link.span, "for {source:?}");
                (format!("link {:?} {:?}", link.url, text.value), link.span)
            }
            _ => unreachable!("runs hold text and links only"),
        };
        actual.push(format!("{line} {}..{}", span.start, span.end));
        start.get_or_insert(span.start as usize);
        end = span.end as usize;
    }
    let start = start.expect("a run is not empty");

    // The old pass: search what is left of the node, split at the link,
    // search what follows it as a string of its own.
    let mut expected = Vec::new();
    let mut done = 0;
    while let Some(candidate) = reference::find_candidate(&value[done..]) {
        let (from, to) = (done + candidate.start, done + candidate.end);
        let link = &value[from..to];
        if from > done {
            let (text, from) = (&value[done..from], start + from);
            expected.push(format!("text {text:?} {}..{from}", start + done));
        }
        let url = format!("{}{link}", candidate.href_prefix);
        expected.push(format!("link {url:?} {link:?} {}..{}", start + from, start + to));
        done = to;
    }
    if done < value.len() {
        expected.push(format!("text {:?} {}..{end}", &value[done..], start + done));
    }
    assert_eq!(actual, expected, "for {source:?}");
}
