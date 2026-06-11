pub(in crate::markdown::tests) fn assert_no_api_html(markdown: &str) {
    assert!(!markdown.contains("<details"), "unexpected <details> in:\n{markdown}");
    assert!(!markdown.contains("class=\"ox-api"), "unexpected ox-api html in:\n{markdown}");
    assert!(!markdown.contains("<table"), "unexpected <table> in:\n{markdown}");
    assert!(!markdown.contains("ox-api-controls"), "unexpected controls in:\n{markdown}");
}

/// Asserts heading levels never increase by more than one (markdownlint
/// MD001), ignoring `#` lines inside fenced code blocks.
pub(in crate::markdown::tests) fn assert_no_heading_level_skips(markdown: &str) {
    let mut previous = 0usize;
    let mut in_fence = false;
    for line in markdown.lines() {
        if line.trim_start().starts_with("```") {
            in_fence = !in_fence;
            continue;
        }
        if in_fence {
            continue;
        }
        let hashes = line.chars().take_while(|&ch| ch == '#').count();
        if hashes == 0 || line.as_bytes().get(hashes) != Some(&b' ') {
            continue;
        }
        if previous != 0 {
            assert!(
                hashes <= previous + 1,
                "heading level skip {previous} -> {hashes} at: {line}\nin:\n{markdown}"
            );
        }
        previous = hashes;
    }
}
