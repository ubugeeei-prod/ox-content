//! Link cards, timeline items, and attribute blocks drop script URLs even when
//! the scheme is split by tab/newline characters or led by C0 controls, which
//! browsers discard before reading the scheme.

use ox_content_transform::transformer::MarkdownTransformer;
use ox_content_transform::{AttrsOptions, CardOptions, TimelineOptions, TransformOptions};

fn transform(markdown: &str) -> String {
    let transformer = MarkdownTransformer::from_options(&TransformOptions {
        gfm: Some(true),
        attributes: Some(AttrsOptions { enabled: Some(true) }),
        cards: Some(CardOptions { enabled: Some(true) }),
        timelines: Some(TimelineOptions { enabled: Some(true), ..Default::default() }),
        ..Default::default()
    });
    transformer.transform(markdown).html
}

const SPLIT_SCHEMES: [&str; 3] =
    ["java\tscript:alert(1)", "\u{1}javascript:alert(1)", "\u{b}java\nscript:alert(1)"];

#[test]
fn link_cards_drop_split_script_urls() {
    for url in SPLIT_SCHEMES {
        let html = transform(&format!("::: link-card[Guide]{{{url}}}\nbody\n:::\n"));
        assert!(!html.contains("alert(1)\""), "{url:?}: {html}");
    }
    let html = transform("::: link-card[Guide]{/guide}\nbody\n:::\n");
    assert!(html.contains("href=\"/guide\""), "{html}");
}

#[test]
fn timeline_items_drop_split_script_urls() {
    for url in SPLIT_SCHEMES {
        let html =
            transform(&format!("::: timeline\n- 2026-08-26 Release {{href=\"{url}\"}}\n:::\n"));
        assert!(!html.contains("<a href"), "{url:?}: {html}");
    }
    let html = transform("::: timeline\n- 2026-08-26 Release {href=\"/notes\"}\n:::\n");
    assert!(html.contains("<a href=\"/notes\">"), "{html}");
}

#[test]
fn attribute_blocks_drop_split_script_urls() {
    for url in SPLIT_SCHEMES {
        let html = transform(&format!("[link](/g){{href=\"{url}\"}}\n"));
        assert!(!html.contains("alert(1)\""), "{url:?}: {html}");
    }
}
