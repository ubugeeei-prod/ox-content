use super::{
    render::render,
    screen::{self, State},
    style,
};

const SOURCE: &str = "---\ntitle: Sample\n---\n\n# 日本語の見出し 👩‍💻\n\nA **bold** paragraph\nwith *emphasis* and [a guide](./guide.md).\n\n## Code\n\n```ts\n  const greeting = \"hello\";\n```\n\n> A quote\n\n- [x] Finished\n- [ ] Pending\n\n| Name | Value |\n| --- | --- |\n| 日本語 | 42 |\n";

#[test]
fn renders_markdown_and_navigation() {
    let doc = render(SOURCE, 64);
    let text = doc.lines.join("\n");
    for expected in [
        "◇ Frontmatter",
        "A bold paragraph with emphasis",
        "  const greeting = \"hello\";",
        "☑ Finished",
        "☐ Pending",
        "日本語",
    ] {
        assert!(text.contains(expected), "Missing {expected}: {text}");
    }
    assert_eq!(
        doc.outline.iter().map(|heading| heading.title.as_str()).collect::<Vec<_>>(),
        ["日本語の見出し 👩‍💻", "Code"]
    );
    assert_eq!(doc.links[0].href, "./guide.md");
    assert!(doc.lines.iter().all(|line| style::width(line) <= 64));
}

#[test]
fn rejects_terminal_controls_and_keeps_graphemes() {
    let doc = render("# Hello\x1b[2J\rhidden\n\n<script>alert(1)</script>\n", 80);
    let text = doc.lines.join("\n");
    assert!(!text.contains(['\x1b', '\x07', '\u{009b}', '\r']));
    assert!(!text.contains("<script>"));
    assert_eq!(style::clip("日👩‍💻本", 4), "日👩‍💻");
    assert_eq!(style::width(&style::clip("日👩‍💻本", 4)), 4);
}

#[test]
fn bounded_frames_and_wrapping_search() {
    let state = State { focus: 2, sidebar: true, theme: "nord".to_string(), ..State::default() };
    for columns in [40, 100, 160] {
        let size = screen::layout(columns, 24, true);
        let doc = render(SOURCE, size.content);
        let output = screen::frame(&state, &doc, &["/project/readme.md".into()], &size, false);
        assert_eq!(output.split("\r\n").count(), 24);
        assert!(output.split("\r\n").all(|line| style::width(line) <= columns as usize));
        assert!(output.contains("OUTLINE"));
    }
    let lines = vec!["first needle".to_string(), "other".to_string(), "last needle".to_string()];
    assert_eq!(screen::find_match(&lines, "NEEDLE", 0, false), Some(2));
    assert_eq!(screen::find_match(&lines, "needle", 0, true), Some(2));
    assert_eq!(screen::find_match(&lines, "needle", 2, false), Some(0));
    assert_eq!(screen::find_match(&lines, "absent", 0, false), None);
}
