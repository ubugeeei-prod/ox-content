//! Bare-URL autolinking trims trailing punctuation, and a closing bracket
//! only when the URL has no unmatched partner for it. The partners were
//! recounted over the whole URL for every closing bracket trimmed, so a URL
//! followed by a run of `)` cost O(n²): 256 KiB took 14 s.
//!
//! These tests pin the cost back to linear and hold the trim to the old
//! recount, kept below, over generated tails.

use std::time::{Duration, Instant};

use ox_content_allocator::Allocator;
use ox_content_ast::{Document, Node, Paragraph, Span, Text};
use ox_content_parser::Parser;
use ox_content_renderer::{HtmlRenderer, HtmlRendererOptions};

fn render(source: &str) -> String {
    let allocator = Allocator::new();
    let document = Parser::new(&allocator, source).parse().expect("source should parse");
    let mut renderer = HtmlRenderer::with_options(HtmlRendererOptions {
        autolink_urls: true,
        autolink_target_blank: false,
        ..Default::default()
    });
    renderer.render(&document)
}

fn best_render(source: &str) -> Duration {
    (0..3)
        .map(|_| {
            let started = Instant::now();
            std::hint::black_box(render(source).len());
            started.elapsed()
        })
        .min()
        .unwrap_or(Duration::MAX)
}

#[test]
fn trailing_brackets_are_trimmed_in_linear_time() {
    for (head, unit) in [
        ("x https://a.b/", ")"),
        ("x https://a.b/(", ")"),
        ("x https://a.b/", "]"),
        ("x https://a.b/", "}"),
        ("x https://a.b/", ".)"),
        ("x https://a.b/", "(a))"),
    ] {
        let build = |bytes: usize| {
            let mut source = String::from(head);
            while source.len() < bytes {
                source.push_str(unit);
            }
            source
        };
        let (small, large) = (build(32 * 1024), build(128 * 1024));
        let (small_time, large_time) = (best_render(&small), best_render(&large));
        // 4x the input: linear is about x4, quadratic about x16.
        assert!(
            large_time < small_time * 8 + Duration::from_millis(50),
            "{head:?} then {unit:?}: 128 KiB took {large_time:?} against {small_time:?} for 32 KiB"
        );
    }
}

/// The trim as it was: closing brackets recounted for each one trimmed.
fn reference_trim(url: &[u8]) -> usize {
    let mut end = url.len();
    while end > 0 {
        let b = url[end - 1];
        match b {
            b'.' | b',' | b';' | b':' | b'!' | b'?' => end -= 1,
            b')' | b']' | b'}' => {
                let open = match b {
                    b')' => b'(',
                    b']' => b'[',
                    _ => b'{',
                };
                let (mut opens, mut closes) = (0usize, 0usize);
                for &x in &url[..end - 1] {
                    if x == open {
                        opens += 1;
                    } else if x == b {
                        closes += 1;
                    }
                }
                if closes >= opens {
                    end -= 1;
                } else {
                    break;
                }
            }
            _ => break,
        }
    }
    end
}

/// Renders `text` as the only text node of a paragraph, so that no inline
/// syntax in it splits it up first.
fn render_text(text: &str) -> String {
    let allocator = Allocator::new();
    let span = Span::new(0, text.len() as u32);
    let mut children = allocator.new_vec();
    children.push(Node::Text(Text { value: text, span }));
    let mut blocks = allocator.new_vec();
    blocks.push(Node::Paragraph(allocator.boxed(Paragraph { children, span })));
    let mut renderer = HtmlRenderer::with_options(HtmlRendererOptions {
        autolink_urls: true,
        autolink_target_blank: false,
        ..Default::default()
    });
    renderer.render(&Document { children: blocks, span })
}

#[test]
fn trimming_agrees_with_the_recount_it_replaced() {
    const TAIL: [u8; 13] = *b"()[]{}.,;:!?a";
    let mut state: u64 = 0x9E37_79B9_7F4A_7C15;
    for _ in 0..5000 {
        let mut url = b"https://a.b/".to_vec();
        state ^= state << 13;
        state ^= state >> 7;
        state ^= state << 17;
        let len = (state % 12) as usize;
        for index in 0..len {
            url.push(TAIL[((state >> (index * 4)) % TAIL.len() as u64) as usize]);
        }
        let url = String::from_utf8(url).expect("ASCII");
        let kept = &url[..reference_trim(url.as_bytes())];
        let html = render_text(&format!("x {url} y"));
        let expected = format!(">{kept}</a>{} y", &url[kept.len()..]);
        assert!(html.contains(&expected), "{url:?} should link {kept:?}: {html}");
    }
}
