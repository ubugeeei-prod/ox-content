use std::collections::BTreeMap;

pub(in crate::markdown::tests) fn assert_markdown_snapshot(name: &str, markdown: &str) {
    let markdown = visible_trailing_whitespace(markdown);
    insta::with_settings!({
        snapshot_path => "../snapshots",
        prepend_module_to_snapshot => false,
        omit_expression => true,
    }, {
        insta::assert_snapshot!(name, markdown);
    });
}

pub(in crate::markdown::tests) fn assert_markdown_map_snapshot(
    name: &str,
    markdown: &BTreeMap<String, String>,
) {
    let mut rendered = String::new();
    for (path, content) in markdown {
        rendered.push_str("===== ");
        rendered.push_str(path);
        rendered.push_str(" =====\n");
        rendered.push_str(content);
        if !content.ends_with('\n') {
            rendered.push('\n');
        }
    }
    assert_markdown_snapshot(name, &rendered);
}

fn visible_trailing_whitespace(value: &str) -> String {
    let mut rendered = String::new();
    for segment in value.split_inclusive('\n') {
        let (line, has_newline) =
            segment.strip_suffix('\n').map_or((segment, false), |line| (line, true));
        let trimmed = line.trim_end_matches([' ', '\t']);
        rendered.push_str(trimmed);
        for ch in line[trimmed.len()..].chars() {
            match ch {
                ' ' => rendered.push_str("<sp>"),
                '\t' => rendered.push_str("<tab>"),
                _ => rendered.push(ch),
            }
        }
        if has_newline {
            rendered.push('\n');
        }
    }
    rendered
}

pub(in crate::markdown::tests) fn assert_no_api_html(markdown: &str) {
    assert!(
        !["<details", "class=\"ox-api", "<table", "ox-api-controls"]
            .iter()
            .any(|needle| markdown.contains(needle)),
        "unexpected API HTML in:\n{markdown}"
    );
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
