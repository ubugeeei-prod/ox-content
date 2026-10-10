use crate::tui::{render::render, style::width};

#[track_caller]
fn lines(source: &str, columns: usize) -> Vec<String> {
    render(source, columns).lines
}

#[test]
fn headings_keep_their_level_and_only_h1_is_underlined() {
    let doc = render("# One\n\n## Two\n\n### Three\n\n#### Four\n\n##### Five\n\n###### Six\n", 20);
    assert_eq!(
        doc.lines,
        [
            "# One",
            "────────────────────",
            "",
            "## Two",
            "",
            "### Three",
            "",
            "#### Four",
            "",
            "##### Five",
            "",
            "###### Six",
            ""
        ]
    );
    let outline: Vec<_> = doc.outline.iter().map(|h| (h.title.as_str(), h.line)).collect();
    assert_eq!(
        outline,
        [("One", 0), ("Two", 3), ("Three", 5), ("Four", 7), ("Five", 9), ("Six", 11)]
    );
}

#[test]
fn outline_lines_point_at_the_rendered_heading() {
    let source = "intro\n\n# First *styled* `code`\n\ntext\n\nSetext\n------\n\n> ## Quoted\n\n- ### Listed\n";
    let doc = render(source, 60);
    let titles: Vec<_> = doc.outline.iter().map(|heading| heading.title.as_str()).collect();
    assert_eq!(titles, ["First styled code", "Setext", "Quoted", "Listed"]);
    for heading in &doc.outline {
        let line = &doc.lines[heading.line];
        assert!(line.contains('#') && line.ends_with(&heading.title), "{line:?} for {heading:?}");
    }
}

#[test]
fn frontmatter_is_summarised_above_the_document() {
    assert_eq!(
        lines("---\ntitle: Sample\ntags: [a, b]\n---\n\nBody\n", 40),
        ["◇ Frontmatter", "  title: Sample", "  tags: [a, b]", "", "Body", ""]
    );
    // YAML's document end marker closes the block too.
    assert_eq!(
        lines("---\ntitle: Sample\n...\nBody\n", 40)[..3],
        ["◇ Frontmatter", "  title: Sample", ""]
    );
    assert_eq!(lines("---\n---\nBody\n", 40), ["◇ Frontmatter", "", "Body", ""]);
}

#[test]
fn long_frontmatter_shows_its_first_eight_lines() {
    let fields =
        (1..=12).map(|index| format!("key{index}: {index}\n")).collect::<Vec<_>>().concat();
    let rendered = lines(&format!("---\n{fields}---\nBody\n"), 40);
    assert_eq!(rendered.len(), 1 + 8 + 1 + 2);
    assert_eq!(rendered[8], "  key8: 8");
    assert_eq!(rendered[10], "Body");
}

#[test]
fn only_a_leading_closed_block_is_frontmatter() {
    for source in
        ["Body\n\n---\ntitle: x\n---\n", "\n---\ntitle: x\n---\n", "---\ntitle: never closed\n"]
    {
        let rendered = lines(source, 40);
        assert!(!rendered.iter().any(|line| line == "◇ Frontmatter"), "{source:?}");
        assert!(rendered.iter().any(|line| line.contains("title:")), "{source:?} lost its text");
    }
}

#[test]
fn paragraphs_join_soft_breaks_and_keep_hard_breaks() {
    assert_eq!(lines("one\ntwo\nthree\n", 40), ["one two three", ""]);
    assert_eq!(lines("one  \ntwo\\\nthree\n", 40), ["one", "two", "three", ""]);
    assert_eq!(lines("first\n\nsecond\n", 40), ["first", "", "second", ""]);
}

#[test]
fn inline_markup_is_reduced_to_readable_text() {
    assert_eq!(
        lines("**bold** *em* ~~gone~~ `code` <kbd>key</kbd> &amp; &copy;\n", 60),
        ["bold em gone code key & ©", ""]
    );
}

#[test]
fn links_and_images_keep_their_destination_visible() {
    let source = "[guide](./guide.md) and [site](https://example.com \"Title\") ![diagram](a.png) <https://auto.link>\n\n[ref]\n\n[ref]: ./ref.md\n";
    let doc = render(source, 200);
    assert_eq!(
        doc.lines,
        [
            "guide (./guide.md) and site (https://example.com) [image: diagram] https://auto.link (https://auto.link)",
            "",
            "ref (./ref.md)",
            ""
        ]
    );
    let links: Vec<_> =
        doc.links.iter().map(|link| (link.title.as_str(), link.href.as_str())).collect();
    assert_eq!(
        links,
        [
            ("guide", "./guide.md"),
            ("site", "https://example.com"),
            ("https://auto.link", "https://auto.link"),
            ("ref", "./ref.md")
        ]
    );
}

#[test]
fn lists_number_from_their_start_and_indent_nested_items() {
    assert_eq!(lines("- one\n- two\n", 40), ["• one", "", "• two", ""]);
    assert_eq!(
        lines("3. three\n4. four\n5. five\n", 40),
        ["3. three", "", "4. four", "", "5. five", ""]
    );
    assert_eq!(
        lines("- top\n  - middle\n    - bottom\n- next\n", 40),
        ["• top", "  • middle", "    • bottom", "", "• next", ""]
    );
    assert_eq!(
        lines("1. first\n   - bullet\n2. second\n", 40),
        ["1. first", "  • bullet", "", "2. second", ""]
    );
}

#[test]
fn task_lists_show_their_state() {
    assert_eq!(
        lines("- [x] done\n- [ ] todo\n- plain\n", 40),
        ["☑ done", "", "☐ todo", "", "• plain", ""]
    );
}

#[test]
fn block_quotes_are_prefixed_per_nesting_level() {
    assert_eq!(
        lines("> one\n> two\n>\n> > deeper\n\nafter\n", 40),
        ["│ one two", "", "│ │ deeper", "", "after", ""]
    );
    assert_eq!(lines("> - item\n> - other\n", 40), ["│ • item", "", "│ • other", ""]);
}

#[test]
fn code_blocks_keep_their_text_verbatim_inside_a_frame() {
    assert_eq!(
        lines("```ts\nconst a = 1;\n\n  indented(); // *not* emphasis\n```\n", 40),
        [
            "╭─ ts",
            "│ const a = 1;",
            "│ ",
            "│   indented(); // *not* emphasis",
            "╰────────────────",
            ""
        ]
    );
    assert_eq!(lines("    indented\n", 40), ["╭─ code", "│ indented", "╰────────────────", ""]);
    assert_eq!(lines("```\nplain\n```\n", 40), ["╭─", "│ plain", "╰────────────────", ""]);
    assert_eq!(
        lines("```rust title=\"main.rs\"\nfn main() {}\n```\n", 40)[0],
        "╭─ rust title=\"main.rs\""
    );
    assert_eq!(
        lines("> ```sh\n> ls\n> ```\n", 40),
        ["│ ╭─ sh", "│ │ ls", "│ ╰────────────────", ""]
    );
}

#[test]
fn tables_align_columns_by_display_width() {
    assert_eq!(
        lines("| Name | Value |\n| --- | :-: |\n| 日本語 | 42 |\n| a | b |\n", 40),
        [
            "│ Name   │ Value │",
            "├────────┼───────┤",
            "│ 日本語 │ 42    │",
            "│ a      │ b     │",
            ""
        ]
    );
    let wrapped =
        lines("| Key | Description |\n| --- | --- |\n| a | A cell that has to wrap |\n", 24);
    let widths: Vec<_> =
        wrapped.iter().filter(|line| !line.is_empty()).map(|line| width(line)).collect();
    assert!(widths.len() > 3 && widths.iter().all(|cells| *cells == widths[0]), "{wrapped:?}");
    assert!(widths[0] <= 24);
}

#[test]
fn thematic_breaks_span_the_page() {
    assert_eq!(lines("above\n\n---\n\nbelow\n", 12), ["above", "", "────────────", "below", ""]);
}

#[test]
fn raw_html_blocks_and_comments_are_not_shown() {
    let rendered = lines(
        "<div class=\"x\">\n<script>alert(1)</script>\n</div>\n\n<!-- note -->\n\nvisible\n",
        40,
    );
    assert_eq!(rendered, ["visible", ""]);
}

#[test]
fn every_line_fits_any_supported_width() {
    let source = "---\ntitle: A fairly long frontmatter title value\n---\n\n# 日本語の見出し 👩‍💻 with a long tail\n\nA paragraph with a https://example.com/very/long/url/that/cannot/break and more text after it.\n\n> Quote with `inline code` and a second sentence that wraps.\n\n1. First item that is long enough to wrap around the page\n   - Nested item 日本語の文章をここに書きます\n\n```rust\nfn main() { println!(\"a long line of code that goes past the margin\"); }\n```\n\n| Column one | Column two | Column three |\n| --- | --- | --- |\n| 日本語 | A longer cell | 42 |\n\n---\n";
    for columns in 20..=120 {
        let doc = render(source, columns);
        for line in &doc.lines {
            assert!(width(line) <= columns, "{columns}: {line:?} is {} wide", width(line));
        }
        assert_eq!(doc.outline.len(), 1);
        assert!(doc.outline[0].line < doc.lines.len());
    }
}

#[test]
fn hostile_and_degenerate_input_renders_without_panicking() {
    let deep_quote = format!("{}deep\n", "> ".repeat(60));
    let deep_list = (0..40)
        .map(|depth| format!("{}- level {depth}\n", "  ".repeat(depth)))
        .collect::<Vec<_>>()
        .concat();
    let wide_table =
        format!("|{}\n|{}\n|{}\n", " h |".repeat(80), " - |".repeat(80), " c |".repeat(80));
    let long_word = "x".repeat(5000);
    for source in [
        "",
        "\n\n\n",
        "---",
        "---\n",
        "---\n---",
        "```",
        "```\n",
        "| a |\n| - |",
        "[unclosed](",
        "![](",
        "***",
        "# ",
        "\u{feff}# BOM\n",
        "\0\0\0",
        "\t\t\t- tabbed\n",
        &deep_quote,
        &deep_list,
        &wide_table,
        &long_word,
    ] {
        for columns in [1, 2, 3, 7, 20, 80, 240] {
            let doc = render(source, columns);
            assert!(
                doc.outline.iter().all(|heading| heading.line <= doc.lines.len()),
                "{source:?}"
            );
            assert!(doc.lines.iter().all(|line| !line.contains(['\x1b', '\r'])), "{source:?}");
        }
    }
}
