use super::handle_key;
use crate::tui::{
    render::{Document, render},
    screen::State,
};
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use std::path::PathBuf;

const INDEX: &str = "# Index\n\nSee [the guide](./guide.md), [spaced](./sp%20ace.md?tab=1#part), [missing](./missing.md), [the site](https://example.com), [mail](mailto:a@example.com) and [cdn](//cdn.example.com/x.md).\n\n## Needle one\n\nFiller.\n\n## Other\n\nneedle two\n";
const HEIGHT: usize = 4;

/// A viewer over real files so opening, reloading and link following touch the disk.
struct Viewer {
    _directory: tempfile::TempDir,
    files: Vec<PathBuf>,
    state: State,
    source: String,
    doc: Document,
}

impl Viewer {
    fn new() -> Self {
        let directory = tempfile::tempdir().unwrap();
        let root = crate::files::absolute(&directory.path().canonicalize().unwrap()).unwrap();
        let mut files = Vec::new();
        for (name, content) in
            [("index.md", INDEX), ("guide.md", "# Guide\n\nBody.\n"), ("sp ace.md", "# Spaced\n")]
        {
            std::fs::write(root.join(name), content).unwrap();
            files.push(root.join(name));
        }
        let state = State { sidebar: true, theme: "nord".to_string(), ..State::default() };
        Self {
            _directory: directory,
            files,
            state,
            source: INDEX.to_string(),
            doc: render(INDEX, 60),
        }
    }

    /// Feed keys, re-rendering after each one like the event loop does. Returns whether to quit.
    fn press(&mut self, keys: &[KeyCode]) -> bool {
        keys.iter().any(|code| self.key(KeyEvent::new(*code, KeyModifiers::NONE)))
    }

    fn key(&mut self, key: KeyEvent) -> bool {
        let quit =
            handle_key(key, &mut self.state, &self.doc, &self.files, &mut self.source, HEIGHT)
                .unwrap();
        self.doc = render(&self.source, 60);
        quit
    }

    fn typed(&mut self, text: &str) -> bool {
        let keys: Vec<_> = text.chars().map(KeyCode::Char).collect();
        self.press(&keys)
    }

    fn focus(&mut self, pane: usize) {
        self.press(&vec![KeyCode::Tab; pane]);
        assert_eq!(self.state.focus, pane);
    }
}

#[test]
fn q_and_control_c_quit_from_the_reader() {
    assert!(Viewer::new().typed("q"));
    assert!(Viewer::new().key(KeyEvent::new(KeyCode::Char('c'), KeyModifiers::CONTROL)));
    assert!(!Viewer::new().typed("c"), "a plain c is not a quit key");
    for pane in 1..4 {
        let mut viewer = Viewer::new();
        viewer.focus(pane);
        assert!(viewer.typed("q"), "pane {pane}");
    }
}

#[test]
fn control_c_always_quits_but_q_is_text_while_searching() {
    let mut viewer = Viewer::new();
    assert!(!viewer.typed("/quiet q"));
    assert_eq!(viewer.state.query, "quiet q");
    assert!(viewer.key(KeyEvent::new(KeyCode::Char('c'), KeyModifiers::CONTROL)));
    let mut help = Viewer::new();
    help.typed("?");
    assert!(help.key(KeyEvent::new(KeyCode::Char('c'), KeyModifiers::CONTROL)));
}

#[test]
fn help_swallows_keys_until_it_is_dismissed() {
    for close in [KeyCode::Esc, KeyCode::Enter, KeyCode::Char('?')] {
        let mut viewer = Viewer::new();
        viewer.typed("?");
        assert!(viewer.state.help);
        viewer.press(&[KeyCode::Down, KeyCode::Tab, KeyCode::Char('t'), KeyCode::Char('/')]);
        assert!(viewer.state.help && !viewer.state.searching);
        assert_eq!(
            (viewer.state.offset, viewer.state.focus, viewer.state.theme.as_str()),
            (0, 0, "nord")
        );
        viewer.press(&[close]);
        assert!(!viewer.state.help, "{close:?}");
    }
}

#[test]
fn reader_scrolling_is_clamped_to_the_document() {
    let mut viewer = Viewer::new();
    let last = viewer.doc.lines.len() - 1;
    for (keys, offset) in [
        (&[KeyCode::Down][..], 1),
        (&[KeyCode::Char('j')], 2),
        (&[KeyCode::Up], 1),
        (&[KeyCode::Char('k'), KeyCode::Char('k'), KeyCode::Up], 0),
        (&[KeyCode::PageDown], HEIGHT),
        (&[KeyCode::Char(' ')], 2 * HEIGHT),
        (&[KeyCode::PageUp], HEIGHT),
        (&[KeyCode::End], last),
        (&[KeyCode::Down, KeyCode::PageDown, KeyCode::Char('G')], last),
        (&[KeyCode::Home], 0),
        (&[KeyCode::Char('G')], last),
        (&[KeyCode::Char('g')], 0),
        (&[KeyCode::PageUp], 0),
    ] {
        viewer.press(keys);
        assert_eq!(viewer.state.offset, offset, "{keys:?}");
    }
    // Unbound keys leave the reader where it is.
    viewer.press(&[
        KeyCode::Char('x'),
        KeyCode::F(5),
        KeyCode::Left,
        KeyCode::Right,
        KeyCode::Delete,
    ]);
    assert_eq!(viewer.state.offset, 0);
}

#[test]
fn tab_cycles_panes_and_escape_returns_to_the_reader() {
    let mut viewer = Viewer::new();
    viewer.typed("b");
    assert!(!viewer.state.sidebar);
    let mut seen = Vec::new();
    for _ in 0..5 {
        viewer.press(&[KeyCode::Tab]);
        seen.push(viewer.state.focus);
        assert!(viewer.state.sidebar, "switching panes reveals the sidebar");
    }
    assert_eq!(seen, [1, 2, 3, 0, 1]);
    seen.clear();
    for _ in 0..5 {
        viewer.press(&[KeyCode::BackTab]);
        seen.push(viewer.state.focus);
    }
    assert_eq!(seen, [0, 3, 2, 1, 0]);
    viewer.focus(2);
    viewer.press(&[KeyCode::Esc]);
    assert_eq!(viewer.state.focus, 0);
}

#[test]
fn each_pane_moves_its_own_selection_within_bounds() {
    let mut viewer = Viewer::new();
    viewer.focus(1);
    viewer.press(&[KeyCode::Down, KeyCode::Down, KeyCode::Down, KeyCode::Down]);
    assert_eq!(viewer.state.file_index, 2);
    viewer.press(&[KeyCode::Home]);
    assert_eq!(viewer.state.file_index, 0);
    viewer.press(&[KeyCode::Tab, KeyCode::End]);
    assert_eq!(viewer.state.focus, 2);
    assert_eq!(viewer.state.outline_index, viewer.doc.outline.len() - 1);
    viewer.press(&[KeyCode::Up]);
    assert_eq!(viewer.state.outline_index, viewer.doc.outline.len() - 2);
    viewer.press(&[KeyCode::Tab, KeyCode::PageDown]);
    assert_eq!(viewer.state.link_index, HEIGHT);
    viewer.press(&[KeyCode::PageDown, KeyCode::PageDown]);
    assert_eq!(viewer.state.link_index, viewer.doc.links.len() - 1);
    // The reader and the other panes kept their own positions.
    assert_eq!((viewer.state.offset, viewer.state.file_index), (0, 0));
}

#[test]
fn empty_lists_keep_their_selection_at_zero() {
    let mut viewer = Viewer::new();
    viewer.source = "Plain text without headings or links.\n".to_string();
    viewer.doc = render(&viewer.source, 60);
    for pane in [2, 3] {
        viewer.state.focus = pane;
        viewer.press(&[KeyCode::Down, KeyCode::End, KeyCode::PageDown, KeyCode::Enter]);
        assert_eq!((viewer.state.outline_index, viewer.state.link_index), (0, 0));
    }
}

#[test]
fn enter_opens_the_selected_file_and_resets_the_view() {
    let mut viewer = Viewer::new();
    viewer.press(&[KeyCode::PageDown]);
    viewer.focus(1);
    viewer.press(&[KeyCode::Down, KeyCode::Enter]);
    assert_eq!(viewer.source, "# Guide\n\nBody.\n");
    assert_eq!((viewer.state.current, viewer.state.focus, viewer.state.offset), (1, 0, 0));
    assert_eq!(viewer.doc.outline[0].title, "Guide");
}

#[test]
fn enter_jumps_to_the_selected_heading() {
    let mut viewer = Viewer::new();
    viewer.focus(2);
    viewer.press(&[KeyCode::Down, KeyCode::Down, KeyCode::Enter]);
    let heading = &viewer.doc.outline[2];
    assert_eq!(heading.title, "Other");
    assert_eq!((viewer.state.offset, viewer.state.focus), (heading.line, 0));
    assert_eq!(viewer.doc.lines[viewer.state.offset], "## Other");
}

#[test]
fn local_links_open_listed_documents_after_decoding() {
    for (link, opened, heading) in [(0, 1, "Guide"), (1, 2, "Spaced")] {
        let mut viewer = Viewer::new();
        viewer.focus(3);
        viewer.press(&vec![KeyCode::Down; link]);
        viewer.press(&[KeyCode::Enter]);
        assert_eq!((viewer.state.current, viewer.state.focus), (opened, 0), "link {link}");
        assert_eq!(viewer.doc.outline[0].title, heading);
        assert_eq!(viewer.state.message, "");
    }
}

#[test]
fn unreachable_links_explain_themselves_and_keep_the_document() {
    for (link, message) in [
        (2, "Local link is outside the selected Markdown files"),
        (3, "External links are displayed as text"),
        (4, "External links are displayed as text"),
        (5, "External links are displayed as text"),
    ] {
        let mut viewer = Viewer::new();
        viewer.focus(3);
        viewer.press(&vec![KeyCode::Down; link]);
        viewer.press(&[KeyCode::Enter]);
        assert_eq!(viewer.state.message, message, "{}", viewer.doc.links[link].href);
        assert_eq!((viewer.state.current, viewer.state.focus), (0, 3));
        assert_eq!(viewer.source, INDEX);
        // The next key press clears the notice.
        viewer.press(&[KeyCode::Up]);
        assert_eq!(viewer.state.message, "");
    }
}

#[test]
fn search_edits_a_query_and_moves_between_matches() {
    let mut viewer = Viewer::new();
    viewer.typed("/NEEDLX");
    assert!(viewer.state.searching);
    viewer.press(&[KeyCode::Backspace]);
    viewer.typed("e");
    assert_eq!(viewer.state.query, "NEEDLe");
    viewer.press(&[KeyCode::Enter]);
    assert!(!viewer.state.searching);
    let first = viewer.state.offset;
    assert_eq!(viewer.doc.lines[first], "## Needle one");
    viewer.typed("n");
    let second = viewer.state.offset;
    assert_eq!(viewer.doc.lines[second], "needle two");
    viewer.typed("n");
    assert_eq!(viewer.state.offset, first, "search wraps around");
    viewer.typed("N");
    assert_eq!(viewer.state.offset, second, "and backwards");
}

#[test]
fn search_can_be_cancelled_and_reports_missing_text() {
    let mut viewer = Viewer::new();
    viewer.typed("/needle");
    viewer.press(&[KeyCode::Esc]);
    assert!(!viewer.state.searching);
    assert_eq!((viewer.state.query.as_str(), viewer.state.offset), ("", 0));
    viewer.typed("n");
    assert_eq!(viewer.state.message, "No matching text", "an empty query matches nothing");
    viewer.typed("/absent");
    viewer.press(&[KeyCode::Enter]);
    assert_eq!((viewer.state.message.as_str(), viewer.state.offset), ("No matching text", 0));
    // Starting a new search discards the previous query; navigation keys are plain text.
    viewer.typed("/jk");
    assert_eq!((viewer.state.query.as_str(), viewer.state.offset), ("jk", 0));
    viewer.press(&[KeyCode::Tab, KeyCode::Down, KeyCode::Char('\u{7}')]);
    assert_eq!((viewer.state.query.as_str(), viewer.state.focus), ("jk", 0));
}

#[test]
fn theme_key_cycles_through_every_theme() {
    let mut viewer = Viewer::new();
    let mut seen = Vec::new();
    for _ in 0..4 {
        viewer.typed("t");
        seen.push(viewer.state.theme.clone());
    }
    assert_eq!(seen, ["light", "mono", "nord", "light"]);
    viewer.state.theme = "unknown".to_string();
    viewer.typed("t");
    assert_eq!(viewer.state.theme, "light");
}

#[test]
fn reload_rereads_the_open_file_without_losing_the_position() {
    let mut viewer = Viewer::new();
    viewer.press(&[KeyCode::Down, KeyCode::Down]);
    std::fs::write(&viewer.files[0], "# Rewritten\n\none\n\ntwo\n\nthree\n").unwrap();
    viewer.typed("r");
    assert_eq!(viewer.doc.outline[0].title, "Rewritten");
    assert_eq!((viewer.state.offset, viewer.state.current), (2, 0));
}

#[test]
fn unreadable_files_leave_the_current_document_open() {
    let mut viewer = Viewer::new();
    std::fs::remove_file(&viewer.files[1]).unwrap();
    viewer.focus(1);
    viewer.press(&[KeyCode::Down, KeyCode::Enter]);
    assert!(!viewer.state.message.is_empty());
    assert_eq!((viewer.state.current, viewer.source.as_str()), (0, INDEX));
    std::fs::write(&viewer.files[2], [0xff, 0xfe]).unwrap();
    viewer.state.focus = 1;
    viewer.press(&[KeyCode::Down, KeyCode::Enter]);
    assert!(viewer.state.message.contains("utf-8"), "{}", viewer.state.message);
    assert_eq!(viewer.state.current, 0);
}
