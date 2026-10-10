use crate::tui::{
    render::{Document, render},
    screen::{State, find_match, frame, layout},
    style::width,
};
use std::path::PathBuf;

const SOURCE: &str = "# Title\n\nIntro with [a guide](./guide.md) and [the site](https://example.com).\n\n## Usage\n\nText.\n\n## 日本語の節\n\nMore text.\n";

fn files() -> Vec<PathBuf> {
    ["/docs/readme.md", "/docs/guide.md", "/docs/日本語.md"]
        .into_iter()
        .map(PathBuf::from)
        .collect()
}

fn state(focus: usize) -> State {
    State { focus, sidebar: true, theme: "nord".to_string(), ..State::default() }
}

fn rows(state: &State, doc: &Document, columns: u16, rows: u16) -> Vec<String> {
    let size = layout(columns, rows, state.sidebar);
    frame(state, doc, &files(), &size, false).split("\r\n").map(str::to_string).collect()
}

#[test]
fn layout_gives_wide_terminals_a_sidebar_and_narrow_ones_the_full_page() {
    for (columns, lines, sidebar, expected) in [
        // (sidebar, content, height)
        (80, 24, true, (26, 54, 21)),
        (79, 24, true, (0, 79, 21)),
        (90, 24, true, (30, 60, 21)),
        (240, 60, true, (30, 210, 57)),
        (120, 10, false, (0, 120, 7)),
        (80, 3, true, (26, 54, 0)),
        (80, 1, true, (26, 54, 0)),
        (1, 1, true, (0, 1, 0)),
        (0, 0, true, (0, 1, 0)),
    ] {
        let size = layout(columns, lines, sidebar);
        assert_eq!((size.sidebar, size.content, size.height), expected, "{columns}x{lines}");
        assert_eq!(size.sidebar + size.content, size.columns, "{columns}x{lines}");
        assert_eq!(size.height + 3, size.rows, "{columns}x{lines}");
    }
}

#[test]
fn frames_fill_the_terminal_exactly_in_every_mode() {
    let files = files();
    for columns in [1, 2, 19, 20, 79, 80, 81, 132, 240] {
        for lines in [1, 3, 4, 10, 24, 60] {
            for focus in 0..4 {
                for (help, searching, message, sidebar) in [
                    (false, false, "", true),
                    (true, false, "", true),
                    (false, true, "", true),
                    (false, false, "External links are displayed as text", true),
                    (false, false, "", false),
                    (true, false, "", false),
                ] {
                    let state = State {
                        help,
                        searching,
                        sidebar,
                        message: message.to_string(),
                        query: "日本語 needle".to_string(),
                        ..state(focus)
                    };
                    let size = layout(columns, lines, sidebar);
                    let doc = render(SOURCE, size.content);
                    let output = frame(&state, &doc, &files, &size, false);
                    let rows: Vec<_> = output.split("\r\n").collect();
                    let label =
                        format!("{columns}x{lines} focus {focus} help {help} sidebar {sidebar}");
                    assert_eq!(rows.len(), size.rows, "{label}");
                    assert!(rows.iter().all(|row| width(row) == size.columns), "{label}: {rows:?}");
                    assert!(!output.contains('\x1b'), "{label}");
                }
            }
        }
    }
}

#[test]
fn header_names_the_open_file_and_the_focused_pane() {
    let doc = render(SOURCE, 54);
    for (focus, pane) in [(0, "READER"), (1, "FILES"), (2, "OUTLINE"), (3, "LINKS")] {
        let rows = rows(&state(focus), &doc, 80, 24);
        assert!(rows[0].starts_with(" ◆ OX CONTENT · readme.md"), "{:?}", rows[0]);
        assert!(rows[1].starts_with(&format!(" {pane} · Tab panes · / search · ? help · q quit")));
        assert_eq!(rows[23].trim_end(), format!(" 1 / 3 · nord · {pane}"));
    }
    let second = State { current: 2, theme: "mono".to_string(), ..state(0) };
    let rows = rows(&second, &doc, 80, 24);
    assert!(rows[0].starts_with(" ◆ OX CONTENT · 日本語.md"));
    assert_eq!(rows[23].trim_end(), " 3 / 3 · mono · READER");
}

#[test]
fn status_line_prefers_search_input_then_messages() {
    let doc = render(SOURCE, 54);
    let searching =
        State { searching: true, query: "needle".into(), message: "old".into(), ..state(0) };
    assert_eq!(rows(&searching, &doc, 80, 24)[23].trim_end(), "/needle");
    let message = State { message: "No matching text".into(), query: "needle".into(), ..state(0) };
    assert_eq!(rows(&message, &doc, 80, 24)[23].trim_end(), "No matching text");
    let hostile = State { message: "bad\x1b[2Jmessage\x07".into(), ..state(0) };
    assert_eq!(rows(&hostile, &doc, 80, 24)[23].trim_end(), "bad[2Jmessage");
}

#[test]
fn sidebar_lists_the_entries_of_the_focused_pane() {
    let doc = render(SOURCE, 54);
    let sidebar = |focus: usize| {
        rows(&state(focus), &doc, 80, 24)[2..6]
            .iter()
            .map(|row| row.split('│').next().unwrap().trim_end().to_string())
            .collect::<Vec<_>>()
    };
    assert_eq!(sidebar(0), ["readme.md", "guide.md", "日本語.md", ""]);
    assert_eq!(sidebar(1), sidebar(0));
    assert_eq!(sidebar(2), ["Title", "Usage", "日本語の節", ""]);
    assert_eq!(sidebar(3), ["a guide (./guide.md)", "the site (https://example", "", ""]);
}

#[test]
fn reader_shows_the_document_from_the_scroll_offset() {
    let doc = render(SOURCE, 54);
    let content = |state: &State| {
        rows(state, &doc, 80, 24)[2..23]
            .iter()
            .map(|row| row.split_once('│').unwrap().1.trim_end().to_string())
            .collect::<Vec<_>>()
    };
    assert_eq!(content(&state(0))[..3], ["# Title", &"─".repeat(54), ""]);
    let scrolled = State { offset: 5, ..state(0) };
    assert_eq!(content(&scrolled)[0], doc.lines[5]);
    let past_the_end = State { offset: doc.lines.len() + 10, ..state(0) };
    assert!(content(&past_the_end).iter().all(String::is_empty));
}

#[test]
fn narrow_terminals_swap_the_page_for_the_focused_list() {
    let doc = render(SOURCE, 40);
    assert!(rows(&state(0), &doc, 40, 12)[2].starts_with("# Title"));
    assert!(rows(&state(1), &doc, 40, 12)[2].starts_with("readme.md"));
    assert!(rows(&state(2), &doc, 40, 12)[3].starts_with("Usage"));
    assert!(rows(&state(3), &doc, 40, 12)[2].starts_with("a guide (./guide.md)"));
    // Hiding the sidebar on a wide terminal behaves the same way.
    let zen = State { sidebar: false, ..state(2) };
    assert!(rows(&zen, &doc, 120, 12)[2].starts_with("Title"));
}

#[test]
fn help_replaces_the_page_until_it_is_closed() {
    let doc = render(SOURCE, 54);
    let help = State { help: true, ..state(0) };
    let rows = rows(&help, &doc, 80, 24);
    let page: Vec<_> =
        rows[2..9].iter().map(|row| row.split_once('│').unwrap().1.trim_end()).collect();
    assert_eq!(
        page,
        [
            "READER KEYS",
            "Tab / Shift-Tab: switch panes",
            "j/k or arrows: move",
            "Enter: open file, heading or link",
            "/: search, n/N: next/previous",
            "b: zen mode, t: theme, r: reload",
            "?: help, q or Ctrl-C: quit"
        ]
    );
    assert!(!rows.iter().any(|row| row.contains("# Title")));
}

#[test]
fn long_lists_scroll_to_keep_the_selection_visible() {
    let files: Vec<_> =
        (0..200).map(|index| PathBuf::from(format!("/docs/{index:03}.md"))).collect();
    let doc = render(SOURCE, 54);
    let size = layout(80, 24, true);
    for selected in [0, 5, 10, 11, 100, 199] {
        let state = State { file_index: selected, ..state(1) };
        let output = frame(&state, &doc, &files, &size, true);
        let highlighted: Vec<_> =
            output.split("\r\n").filter(|row| row.contains("\x1b[7m")).collect();
        // The header is always reversed; exactly one list row joins it.
        assert_eq!(highlighted.len(), 2, "{selected}");
        assert!(
            highlighted[1].contains(&format!("{selected:03}.md")),
            "{selected}: {highlighted:?}"
        );
    }
    let reader = frame(&State { file_index: 5, ..state(0) }, &doc, &files, &size, true);
    assert_eq!(reader.matches("\x1b[7m").count(), 1, "the reader pane highlights no list row");
}

#[test]
fn file_names_cannot_inject_terminal_controls() {
    let files = [PathBuf::from("/docs/evil\x1b[2J\x07name.md")];
    let doc = render(SOURCE, 54);
    for focus in 0..2 {
        let output = frame(&state(focus), &doc, &files, &layout(80, 24, true), false);
        assert!(!output.contains(['\x1b', '\x07']), "{output:?}");
        assert!(output.contains("evil[2Jname.md"));
    }
}

#[test]
fn an_empty_file_list_still_draws_a_frame() {
    let output = frame(&state(1), &Document::default(), &[], &layout(80, 24, true), false);
    assert_eq!(output.split("\r\n").count(), 24);
    assert!(output.contains(" 1 / 0 · nord · FILES"));
}

#[test]
fn search_wraps_in_both_directions_and_ignores_case() {
    let lines: Vec<_> = ["alpha Needle", "beta", "gamma NEEDLE", "delta", "Écran needle"]
        .into_iter()
        .map(String::from)
        .collect();
    for (query, offset, backwards, expected) in [
        ("needle", 0, false, Some(2)),
        ("needle", 2, false, Some(4)),
        ("needle", 4, false, Some(0)),
        ("needle", 0, true, Some(4)),
        ("needle", 4, true, Some(2)),
        ("needle", 1, true, Some(0)),
        ("NEEDLE", 3, false, Some(4)),
        ("écran", 0, false, Some(4)),
        ("beta", 1, false, Some(1)),
        ("beta", 1, true, Some(1)),
        ("absent", 0, false, None),
        ("", 0, false, None),
        (" ", 3, false, Some(4)),
    ] {
        assert_eq!(
            find_match(&lines, query, offset, backwards),
            expected,
            "{query:?} from {offset}"
        );
    }
    assert_eq!(find_match(&[], "needle", 0, false), None);
    assert_eq!(find_match(&["only needle".to_string()], "needle", 0, true), Some(0));
}
