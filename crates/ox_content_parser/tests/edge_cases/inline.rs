use ox_content_allocator::Allocator;
use ox_content_ast::Node;
use ox_content_parser::ParserOptions;

use super::{first_text, parse_with_options};

#[test]
fn inline_link_handles_nested_parentheses() {
    let allocator = Allocator::new();
    let doc = parse_with_options(
        &allocator,
        "[docs](https://example.com/a(b)c)",
        ParserOptions::default(),
    );

    match &doc.children[0] {
        Node::Paragraph(paragraph) => match &paragraph.children[0] {
            Node::Link(link) => assert_eq!(link.url, "https://example.com/a(b)c"),
            other => panic!("expected link, got {other:?}"),
        },
        other => panic!("expected paragraph, got {other:?}"),
    }
}

#[test]
fn inline_raw_html_is_preserved_as_html_node() {
    let allocator = Allocator::new();
    let doc = parse_with_options(
        &allocator,
        "before <input type=\"checkbox\"> after",
        ParserOptions::default(),
    );

    match &doc.children[0] {
        Node::Paragraph(paragraph) => {
            assert!(
                matches!(&paragraph.children[1], Node::Html(html) if html.value == "<input type=\"checkbox\">")
            );
        }
        other => panic!("expected paragraph, got {other:?}"),
    }
}

#[test]
fn list_item_allows_inline_raw_html() {
    let allocator = Allocator::new();
    let doc = parse_with_options(
        &allocator,
        "- <input type=\"checkbox\"> task",
        ParserOptions::default(),
    );

    match &doc.children[0] {
        Node::List(list) => match &list.children[0].children[0] {
            Node::Paragraph(paragraph) => {
                assert!(
                    matches!(&paragraph.children[0], Node::Html(html) if html.value == "<input type=\"checkbox\">")
                );
            }
            other => panic!("expected paragraph, got {other:?}"),
        },
        other => panic!("expected list, got {other:?}"),
    }
}

#[test]
fn inline_code_keeps_raw_html_literal() {
    let allocator = Allocator::new();
    let doc = parse_with_options(&allocator, "`<input>`", ParserOptions::default());

    match &doc.children[0] {
        Node::Paragraph(paragraph) => {
            assert!(
                matches!(&paragraph.children[0], Node::InlineCode(code) if code.value == "<input>")
            );
        }
        other => panic!("expected paragraph, got {other:?}"),
    }
}

#[test]
fn image_url_handles_nested_parentheses() {
    let allocator = Allocator::new();
    let doc =
        parse_with_options(&allocator, "![diagram](./img(test).png)", ParserOptions::default());

    match &doc.children[0] {
        Node::Paragraph(paragraph) => match &paragraph.children[0] {
            Node::Image(image) => assert_eq!(image.url, "./img(test).png"),
            other => panic!("expected image, got {other:?}"),
        },
        other => panic!("expected paragraph, got {other:?}"),
    }
}

#[test]
fn escaped_marker_remains_literal_text() {
    let allocator = Allocator::new();
    let doc = parse_with_options(&allocator, "\\*literal\\*", ParserOptions::default());

    match &doc.children[0] {
        Node::Paragraph(paragraph) => {
            let text = paragraph
                .children
                .iter()
                .filter_map(first_text)
                .collect::<std::vec::Vec<_>>()
                .join("");
            assert_eq!(text, "*literal*");
        }
        other => panic!("expected paragraph, got {other:?}"),
    }
}

#[test]
fn backslash_before_non_punctuation_stays_literal() {
    let allocator = Allocator::new();
    // CommonMark example 13: only ASCII punctuation is escapable; before
    // anything else (including multibyte characters, which used to panic
    // on a byte-index slice) the backslash is literal text.
    let doc = parse_with_options(&allocator, "\\\t\\A\\a\\ \\3\\φ\\«", ParserOptions::default());

    match &doc.children[0] {
        Node::Paragraph(paragraph) => {
            let text = paragraph
                .children
                .iter()
                .filter_map(first_text)
                .collect::<std::vec::Vec<_>>()
                .join("");
            assert_eq!(text, "\\\t\\A\\a\\ \\3\\φ\\«");
        }
        other => panic!("expected paragraph, got {other:?}"),
    }
}

#[test]
fn gfm_autolink_does_not_fire_inside_link_text() {
    let allocator = Allocator::new();
    let doc = parse_with_options(
        &allocator,
        "[visit www.example.com](https://real.example)",
        ParserOptions::gfm(),
    );

    match &doc.children[0] {
        Node::Paragraph(paragraph) => match &paragraph.children[0] {
            Node::Link(link) => {
                assert_eq!(link.url, "https://real.example");
                // GFM excludes link text from the autolink extension; a
                // nested Link here would render as invalid nested <a> tags.
                assert!(
                    link.children.iter().all(|child| matches!(child, Node::Text(_))),
                    "link text must stay plain text, got {:?}",
                    link.children
                );
            }
            other => panic!("expected link, got {other:?}"),
        },
        other => panic!("expected paragraph, got {other:?}"),
    }
}

#[test]
fn gfm_autolink_still_fires_inside_strikethrough() {
    let allocator = Allocator::new();
    let doc = parse_with_options(&allocator, "~~see www.example.com~~", ParserOptions::gfm());

    match &doc.children[0] {
        Node::Paragraph(paragraph) => match &paragraph.children[0] {
            Node::Delete(delete) => {
                let link_count =
                    delete.children.iter().filter(|child| matches!(child, Node::Link(_))).count();
                // The root-level pass recurses into emphasis-like
                // containers, so the bare URL still links exactly once.
                assert_eq!(link_count, 1, "expected one autolink, got {:?}", delete.children);
            }
            other => panic!("expected strikethrough, got {other:?}"),
        },
        other => panic!("expected paragraph, got {other:?}"),
    }
}

#[test]
fn unmatched_strikethrough_remains_text() {
    let allocator = Allocator::new();
    let doc = parse_with_options(&allocator, "~~open", ParserOptions::gfm());

    match &doc.children[0] {
        Node::Paragraph(paragraph) => {
            // The unmatched marker stays literal text. It remains a separate
            // node from the following prose: the GFM autolink post-pass is
            // the only thing that coalesces adjacent text nodes, and it only
            // runs when the block contains a candidate byte (`:`, `@`, `&`,
            // or `www.`) — plain prose keeps the parser's raw segmentation.
            assert!(matches!(&paragraph.children[0], Node::Text(_)));
            assert_eq!(first_text(&paragraph.children[0]), Some("~~"));
            assert_eq!(first_text(&paragraph.children[1]), Some("open"));
        }
        other => panic!("expected paragraph, got {other:?}"),
    }
}

#[test]
fn hard_break_creates_break_node() {
    let allocator = Allocator::new();
    let doc = parse_with_options(&allocator, "line 1\\\nline 2", ParserOptions::default());

    match &doc.children[0] {
        Node::Paragraph(paragraph) => {
            assert!(paragraph.children.iter().any(|node| matches!(node, Node::Break(_))));
        }
        other => panic!("expected paragraph, got {other:?}"),
    }
}

/// Whether the paragraph's first inline is a `Strong` node.
fn parses_as_strong(source: &str, cjk_emphasis: bool) -> bool {
    let allocator = Allocator::new();
    let options = ParserOptions { cjk_emphasis, ..ParserOptions::default() };
    let doc = parse_with_options(&allocator, source, options);
    match &doc.children[0] {
        Node::Paragraph(paragraph) => {
            paragraph.children.iter().any(|node| matches!(node, Node::Strong(_)))
        }
        other => panic!("expected paragraph, got {other:?}"),
    }
}

#[test]
fn cjk_emphasis_off_leaves_punctuation_adjacent_runs_literal() {
    // CommonMark's flanking rules reject these, and the default profile follows
    // the spec: the conformance suite depends on it.
    for source in [
        "A**強調。**B",
        "A**。強調**B",
        "A**強調、**B",
        "A**強調！**B",
        "A**強調）**B",
        "中文**加粗，**测试",
    ] {
        assert!(!parses_as_strong(source, false), "{source} must stay literal by default");
    }
}

#[test]
fn cjk_emphasis_on_pairs_punctuation_adjacent_runs() {
    for source in [
        "A**強調。**B",
        "A**。強調**B",
        "A**強調、**B",
        "A**強調！**B",
        "A**強調）**B",
        "中文**加粗，**测试",
    ] {
        assert!(parses_as_strong(source, true), "{source} must render as strong when enabled");
    }
}

#[test]
fn cjk_emphasis_leaves_ascii_punctuation_alone() {
    // Only fullwidth and CJK-specific punctuation is reclassified. Halfwidth
    // ASCII is written the same way in every script, so enabling the option
    // must not change how a Latin document parses.
    for source in ["a**bold.**c", "a**.bold**c", "a**bold!**c"] {
        assert_eq!(
            parses_as_strong(source, false),
            parses_as_strong(source, true),
            "{source} must parse the same either way"
        );
    }
}

#[test]
fn cjk_emphasis_keeps_working_for_cases_commonmark_already_accepts() {
    // Emphasis between CJK *characters* needs no help from the option; it must
    // keep working with the option in either state.
    for source in ["これは**重要**です。", "「**強調**」というもの", "（**注**）です"]
    {
        assert!(parses_as_strong(source, false), "{source} works per CommonMark");
        assert!(parses_as_strong(source, true), "{source} must keep working when enabled");
    }
}

#[test]
fn cjk_emphasis_does_not_pair_across_whitespace() {
    // The option relaxes punctuation, not the whitespace rule: a run with
    // whitespace on its inner side still cannot open or close.
    assert!(!parses_as_strong("A** 強調。 **B", true));
    // Ideographic space is whitespace, and the flanking rules test that before
    // punctuation, so it is unaffected by the reclassification.
    assert!(!parses_as_strong("A**\u{3000}強調\u{3000}**B", true));
}
