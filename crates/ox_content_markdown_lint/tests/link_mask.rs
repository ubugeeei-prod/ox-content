//! `mask_link_targets` used to advance its cursor only when a reference label
//! closed, so any line holding a `][` without a later `]` spun forever. These
//! tests pin both halves of the contract: the masker always terminates, and it
//! still blanks exactly the link targets it is supposed to blank.

use std::sync::mpsc::{RecvTimeoutError, channel};
use std::thread;
use std::time::Duration;

use ox_content_markdown_lint::{MarkdownLintOptions, lint_markdown, lint_markdown_documents};

/// Runs `work` on a helper thread so a hang fails the test instead of wedging
/// the whole suite. A regression here is an infinite loop, not a slow path, so
/// the budget is deliberately generous.
fn within_timeout<T: Send + 'static>(
    label: &str,
    seconds: u64,
    work: impl FnOnce() -> T + Send + 'static,
) -> T {
    let (sender, receiver) = channel();
    thread::spawn(move || {
        let _ = sender.send(work());
    });

    match receiver.recv_timeout(Duration::from_secs(seconds)) {
        Ok(value) => value,
        Err(RecvTimeoutError::Timeout) => panic!("{label} did not finish within {seconds}s"),
        Err(RecvTimeoutError::Disconnected) => panic!("{label} panicked"),
    }
}

fn mask_of(source: &str) -> String {
    lint_markdown(source, None).masked_document
}

/// `visible` kept, then `blanks` spaces, then `tail` kept.
fn expected(visible: &str, blanks: usize, tail: &str) -> String {
    let mut out = String::from(visible);
    out.extend(std::iter::repeat_n(' ', blanks));
    out.push_str(tail);
    out
}

#[test]
fn unpaired_reference_bracket_terminates() {
    let masked = within_timeout("lone `][`", 30, || mask_of("text ][ text"));

    // Nothing here is a link, so only the bracket characters themselves are
    // blanked (every masked line loses its `[]()` markers). Both words survive.
    assert_eq!(masked, "text    text");
}

#[test]
fn repeated_unpaired_reference_brackets_terminate() {
    let masked =
        within_timeout("repeated `][`", 30, || mask_of("prose ][ more ][ and ][ still going"));

    // The first `][` finds its closing `]` in the *second* `][`, so the span
    // between them is treated as a reference label and blanked; the third `][`
    // never closes. What matters is that the scan terminates and the text past
    // the last unpaired bracket still reaches the rules.
    assert_eq!(masked, "prose            and    still going");
}

#[test]
fn unpaired_reference_bracket_inside_a_large_document_terminates() {
    // Guards against the fix being linear-per-line but quadratic overall: 20k
    // lines each carrying the pathological shape.
    let source = (0..20_000).map(|_| "a ][ b\n").collect::<String>();
    let line_count = source.lines().count();

    let masked = within_timeout("20k pathological lines", 60, move || mask_of(&source));

    assert_eq!(masked.lines().count(), line_count);
    assert!(masked.lines().all(|line| line == "a    b"));
}

#[test]
fn reference_labels_are_still_blanked() {
    // The label must be blanked so the spellchecker never sees it, and the
    // blanking must be length-preserving so diagnostic columns stay accurate.
    assert_eq!(mask_of("[shown][hidden]"), expected(" shown", 9, ""));
    assert_eq!(mask_of("see [shown][hidden] here"), expected("see  shown", 9, " here"));
    assert!(!mask_of("[shown][hidden]").contains("hidden"));
}

#[test]
fn inline_link_targets_are_still_blanked() {
    assert_eq!(mask_of("[shown](/hidden)"), expected(" shown", 10, ""));
    assert_eq!(mask_of("[shown](/a(nested)b) tail"), expected(" shown", 14, " tail"));
    assert!(!mask_of("[shown](/hidden)").contains("hidden"));
}

#[test]
fn text_after_an_unpaired_bracket_is_still_linted() {
    // The old loop never reached anything past the `][`. Text that follows it
    // has to keep flowing through the rules.
    let result = within_timeout("lint after `][`", 30, || {
        lint_markdown("Sentence ][ with with a repeated word.", None)
    });

    let rules: Vec<_> = result.diagnostics.iter().map(|d| d.rule_id.as_str()).collect();
    assert!(
        rules.contains(&"repeated-word"),
        "expected a repeated-word diagnostic after the `][`, got {rules:?}"
    );
}

#[test]
fn every_short_bracket_soup_terminates() {
    // Exhaustive over the characters that drive the masker's state machine.
    // Every string up to length 6 must terminate and must preserve length.
    const ALPHABET: [char; 5] = ['[', ']', '(', ')', 'a'];
    const MAX_LEN: u32 = 6;

    let mut sources = Vec::new();
    for len in 1..=MAX_LEN {
        for index in 0..ALPHABET.len().pow(len) {
            let mut rest = index;
            let mut source = String::with_capacity(len as usize);
            for _ in 0..len {
                source.push(ALPHABET[rest % ALPHABET.len()]);
                rest /= ALPHABET.len();
            }
            sources.push(source);
        }
    }

    let expected = sources.clone();
    let results = within_timeout("exhaustive bracket soup", 180, move || {
        lint_markdown_documents(&sources, Some(MarkdownLintOptions::default()))
    });

    assert_eq!(results.len(), expected.len());
    for (result, source) in results.iter().zip(&expected) {
        assert_eq!(
            result.masked_document.chars().count(),
            source.chars().count(),
            "masking changed the length of {source:?}"
        );
    }
}

#[test]
fn multibyte_text_around_an_unpaired_bracket_terminates() {
    // The masker works in `char`s; a regression that reintroduced byte
    // indexing would panic rather than hang, so cover that too.
    let masked = within_timeout("multibyte `][`", 30, || mask_of("日本語 ][ の文章です"));

    assert_eq!(masked, "日本語    の文章です");
}
