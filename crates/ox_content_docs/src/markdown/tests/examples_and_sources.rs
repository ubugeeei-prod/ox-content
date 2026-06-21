use super::*;

#[test]
fn markdown_example_with_prose_and_fence_is_not_double_wrapped() {
    let mut entry = test_entry("ArgSchema", "interface", "/repo/src/a.ts", "Schema.");
    entry.examples =
        vec!["Basic string argument:\n```ts\nconst schema = { type: 'string' }\n```".to_string()];
    let out = generate_markdown(&lifecycle_module(entry), &markdown_typedoc_options());
    assert_markdown_map_snapshot(
        "markdown_example_with_prose_and_fence_is_not_double_wrapped",
        &out,
    );
    assert_markdown_map_snapshot(
        "markdown_example_with_prose_and_fence_is_not_double_wrapped",
        &out,
    );

    // Prose stays a real line immediately followed by the single code fence; the
    // whole example is not wrapped in another ```ts (which would put the fence
    // before the prose).
}

#[test]
fn markdown_example_single_fence_is_unchanged() {
    let mut entry = test_entry("ArgSchema", "interface", "/repo/src/a.ts", "Schema.");
    entry.examples = vec!["```ts\nconst x = 1\n```".to_string()];
    let out = generate_markdown(&lifecycle_module(entry), &markdown_typedoc_options());
    assert_markdown_map_snapshot("markdown_example_single_fence_is_unchanged", &out);
}

#[test]
fn markdown_example_bare_code_is_wrapped_in_ts_fence() {
    let mut entry = test_entry("ArgSchema", "interface", "/repo/src/a.ts", "Schema.");
    entry.examples = vec!["const x = 1".to_string()];
    let out = generate_markdown(&lifecycle_module(entry), &markdown_typedoc_options());
    assert_markdown_map_snapshot("markdown_example_bare_code_is_wrapped_in_ts_fence", &out);
}

#[test]
fn markdown_example_with_multiple_fences_passes_through() {
    let mut entry = test_entry("ArgSchema", "interface", "/repo/src/a.ts", "Schema.");
    entry.examples = vec!["```ts\na\n```\n\n```js\nb\n```".to_string()];
    let out = generate_markdown(&lifecycle_module(entry), &markdown_typedoc_options());
    assert_markdown_map_snapshot("markdown_example_with_multiple_fences_passes_through", &out);

    // Both fenced blocks are preserved verbatim (not collapsed or double-wrapped).
}

#[test]
fn html_example_with_prose_and_fence_renders_blocks() {
    let mut entry = test_entry("ArgSchema", "interface", "/repo/src/a.ts", "Schema.");
    entry.examples = vec!["Basic string argument:\n```ts\nconst schema = 1\n```".to_string()];
    let out = generate_markdown(&lifecycle_module(entry), &html_typedoc_options());
    assert_markdown_map_snapshot("html_example_with_prose_and_fence_renders_blocks", &out);

    // Prose becomes a paragraph and the code a code block, rather than the whole
    // mixed example being escaped inside a single <pre><code>.
}

#[test]
fn entries_without_file_omit_source_link() {
    let docs = vec![ApiDocModule {
        file: "mod".to_string(),
        entries: vec![
            test_entry("localSym", "function", "packages/x/src/a.ts", "Local symbol."),
            // Empty file = external-package source: no in-repo source location.
            test_entry("externalSym", "function", "", "External symbol."),
        ],
        ..ApiDocModule::default()
    }];

    for render_style in [MarkdownRenderStyle::Html, MarkdownRenderStyle::Markdown] {
        let markdown = generate_markdown(
            &docs,
            &MarkdownDocsOptions {
                github_url: Some("https://github.com/o/r".to_string()),
                path_strategy: MarkdownPathStrategy::TypeDoc,
                render_style,
                ..MarkdownDocsOptions::default()
            },
        );
        let snapshot_name = match render_style {
            MarkdownRenderStyle::Html => "entries_without_file_omit_source_link__html",
            MarkdownRenderStyle::Markdown => "entries_without_file_omit_source_link__markdown",
        };
        assert_markdown_map_snapshot(snapshot_name, &markdown);

        // The local symbol links to its in-repo source.

        // The external symbol emits no source link and leaks no path.
    }
}
