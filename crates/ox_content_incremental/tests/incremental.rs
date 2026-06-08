use ox_content_incremental::{
    complete_provisional_markdown, stable_prefix_len, IncrementalHtmlRenderer,
    IncrementalRenderOptions,
};
use ox_content_parser::ParserOptions;

#[test]
fn keeps_tail_pending_until_next_top_level_line() {
    assert_eq!(stable_prefix_len("hello\n\n"), 0);
    assert_eq!(stable_prefix_len("hello\n\nnext"), "hello\n\n".len());
}

#[test]
fn does_not_commit_blank_lines_inside_fenced_code() {
    let source = "```ts\nconst value = 1;\n\nstill code";
    assert_eq!(stable_prefix_len(source), 0);
    let closed = "```ts\nconst value = 1;\n\nstill code\n```\n\nnext";
    assert_eq!(stable_prefix_len(closed), "```ts\nconst value = 1;\n\nstill code\n```\n\n".len());
}

#[test]
fn keeps_possible_loose_list_continuations_pending() {
    assert_eq!(stable_prefix_len("- item\n\n  continuation"), 0);
    assert_eq!(stable_prefix_len("- item\n\nnext"), "- item\n\n".len());
}

#[test]
fn provisional_completion_closes_inline_delimiters() {
    let options = ParserOptions::gfm();
    assert_eq!(complete_provisional_markdown("hello **wor", &options), "hello **wor**");
    assert_eq!(complete_provisional_markdown("hello `wor", &options), "hello `wor`");
    assert_eq!(complete_provisional_markdown("hello ~~wor", &options), "hello ~~wor~~");
}

#[test]
fn incremental_renderer_renders_pending_heading_and_emphasis() {
    let mut renderer = IncrementalHtmlRenderer::new(ParserOptions::gfm());
    let result = renderer
        .append("# Hel", IncrementalRenderOptions::default())
        .expect("pending heading should render");
    assert_eq!(result.delta_html, "");
    assert_eq!(result.pending_html, "<h1 id=\"hel\">Hel</h1>\n");

    let result = renderer
        .append("lo **wor", IncrementalRenderOptions::default())
        .expect("pending emphasis should render");
    assert_eq!(result.pending_html, "<h1 id=\"hello-wor\">Hello <strong>wor</strong></h1>\n");
}

#[test]
fn incremental_renderer_commits_stable_prefix_and_replaces_pending() {
    let mut renderer = IncrementalHtmlRenderer::new(ParserOptions::gfm());
    let result = renderer
        .append("# Hello", IncrementalRenderOptions::default())
        .expect("pending heading should render");
    assert!(!result.did_commit);
    assert_eq!(result.html, "<h1 id=\"hello\">Hello</h1>\n");

    let result = renderer
        .append("\n\nNext", IncrementalRenderOptions::default())
        .expect("heading should commit once the next block starts");
    assert!(result.did_commit);
    assert_eq!(result.delta_html, "<h1 id=\"hello\">Hello</h1>\n");
    assert_eq!(result.pending_html, "<p>Next</p>\n");
    assert_eq!(result.html, "<h1 id=\"hello\">Hello</h1>\n<p>Next</p>\n");
}

#[test]
fn provisional_heading_ids_use_committed_heading_state_without_mutating_it() {
    let mut renderer = IncrementalHtmlRenderer::new(ParserOptions::default());
    renderer
        .append("# Same\n\n# Same", IncrementalRenderOptions::default())
        .expect("first heading should commit");
    assert_eq!(
        renderer
            .append("", IncrementalRenderOptions::default())
            .expect("pending heading should render")
            .pending_html,
        "<h1 id=\"same-1\">Same</h1>\n"
    );
    let result = renderer.finish().expect("pending heading should commit");
    assert_eq!(result.delta_html, "<h1 id=\"same-1\">Same</h1>\n");
}
