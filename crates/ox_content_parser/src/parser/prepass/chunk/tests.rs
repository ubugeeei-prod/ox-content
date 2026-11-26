//! The window has to hand out, for every line it is asked about, exactly the
//! text a fresh join from that line would build. These tests keep the join it
//! replaced as the reference and compare the two over generated documents.
#![allow(clippy::disallowed_macros, clippy::disallowed_methods, clippy::disallowed_types)]

use ox_content_allocator::Allocator;

use super::ChunkWindow;
use crate::parser::Parser;
use crate::parser::line_scan::{line_end, line_terminator_end, next_line_start};
use crate::parser::reference::strip_quote_markers;

/// The per-candidate join the window replaced: the stripped lines from `pos`
/// to the next blank line, and the source offset each of them starts at.
fn join_from(source: &str, mut pos: usize) -> (String, Vec<usize>) {
    let bytes = source.as_bytes();
    let mut joined = String::new();
    let mut line_starts = Vec::new();
    while pos < bytes.len() {
        let end = line_end(bytes, pos);
        let stripped = strip_quote_markers(&source[pos..end]);
        if stripped.trim().is_empty() {
            break;
        }
        line_starts.push(pos);
        joined.push_str(stripped);
        joined.push('\n');
        pos = line_terminator_end(bytes, end);
    }
    line_starts.push(pos.min(bytes.len()));
    (joined, line_starts)
}

fn line_starts(source: &str) -> Vec<usize> {
    let bytes = source.as_bytes();
    let mut starts = Vec::new();
    let mut pos = 0;
    while pos < bytes.len() {
        starts.push(pos);
        pos = next_line_start(bytes, pos);
    }
    starts
}

/// Visits the chosen line starts in order through one window, the way the
/// pre-pass does, and checks each answer against a fresh join.
fn assert_matches_join(source: &str, visit: impl Fn(usize) -> bool) {
    let allocator = Allocator::new();
    let parser = Parser::new(&allocator, source);
    let bytes = source.as_bytes();
    let mut window = ChunkWindow::default();

    for (index, &pos) in line_starts(source).iter().enumerate() {
        if !visit(index) {
            continue;
        }
        let (expected, expected_starts) = join_from(source, pos);
        let text = window.text_at(&parser, pos);
        assert_eq!(text, expected, "chunk at {pos} of {source:?}");

        // Every prefix the definition parser can report as consumed ends
        // just past a newline; the line after it must be the one the join's
        // line table named.
        assert_eq!(window.line_after(bytes, pos, ""), None);
        let mut lines = 0;
        for (offset, _) in text.match_indices('\n') {
            lines += 1;
            assert_eq!(
                window.line_after(bytes, pos, &text[..=offset]),
                Some(expected_starts[lines]),
                "line after {lines} line(s) of the chunk at {pos} of {source:?}"
            );
        }
    }
}

const LINES: &[&str] = &[
    "[x]: /u",
    "> [z]: /v",
    "   > > [n]: /n",
    ">",
    "> ",
    "a",
    "",
    " \t",
    "\u{a0}",
    "> \u{a0}",
    "***",
    "  'title'",
];
const TERMINATORS: &[&str] = &["\n", "\r\n", "\r"];

fn document(mut index: usize, lines: usize, final_terminator: bool) -> String {
    let mut source = String::new();
    for position in 0..lines {
        source.push_str(LINES[index % LINES.len()]);
        index /= LINES.len();
        let terminator = TERMINATORS[index % TERMINATORS.len()];
        index /= TERMINATORS.len();
        if position + 1 < lines || final_terminator {
            source.push_str(terminator);
        }
    }
    source
}

#[test]
fn every_line_gets_the_chunk_a_fresh_join_would_build() {
    let choices = LINES.len() * TERMINATORS.len();
    for lines in 1..=3 {
        for index in 0..choices.pow(lines) {
            for final_terminator in [true, false] {
                let source = document(index, lines as usize, final_terminator);
                assert_matches_join(&source, |_| true);
            }
        }
    }
}

#[test]
fn skipped_lines_do_not_desynchronize_the_window() {
    // The pre-pass asks about candidate lines only, so the window has to
    // step over whatever lies between two of them, in any combination.
    let mut state = 0x2545_f491_4f6c_dd1d_u64;
    let mut next = move || {
        state ^= state << 13;
        state ^= state >> 7;
        state ^= state << 17;
        state as usize
    };
    for _ in 0..4_000 {
        let lines = 4 + next() % 9;
        let mut source = String::new();
        for _ in 0..lines {
            // Blank lines are rare, so most documents are one long window.
            let line = if next() % 8 == 0 { "" } else { LINES[next() % LINES.len()] };
            source.push_str(line);
            source.push_str(TERMINATORS[if next() % 3 == 0 { next() % 3 } else { 0 }]);
        }
        let mask = next();
        assert_matches_join(&source, |index| mask >> (index % 32) & 1 == 1);
        assert_matches_join(&source, |_| true);
    }
}

#[test]
fn a_window_without_markers_borrows_the_source() {
    let source = "[a]: /a\n***\n[b]: /b\n# h\n[c]: /c\n";
    let allocator = Allocator::new();
    let parser = Parser::new(&allocator, source);
    let mut window = ChunkWindow::default();
    let before = allocator.allocated_bytes();
    for pos in line_starts(source) {
        let text = window.text_at(&parser, pos);
        assert!(source.as_bytes().as_ptr_range().contains(&text.as_ptr()), "copied at {pos}");
    }
    assert_eq!(allocator.allocated_bytes(), before);
}

#[test]
fn a_window_that_needs_stripping_is_copied_once() {
    // Asking about every line of one window must cost one copy of it, not
    // one copy per line.
    let source = "> [x]: /u\r\n> ***\r\n".repeat(2_000);
    let allocator = Allocator::with_capacity(4 * source.len());
    let parser = Parser::new(&allocator, &source);
    let used = |allocator: &Allocator| allocator.allocated_bytes() - allocator.chunk_capacity();
    let before = used(&allocator);
    let mut window = ChunkWindow::default();
    for pos in line_starts(&source) {
        assert!(window.text_at(&parser, pos).ends_with("***\n"));
    }
    assert!(
        used(&allocator) - before <= source.len(),
        "{} bytes of arena for a {} byte window",
        used(&allocator) - before,
        source.len()
    );
}
