use super::*;

#[test]
fn render_stats_keeps_stats_summary_by_default() {
    let out = generate_markdown(&stats_docs(), &markdown_typedoc_options());
    assert_markdown_map_snapshot("render_stats_keeps_stats_summary_by_default", &out);
}

#[test]
fn render_stats_false_omits_stats_summary() {
    let options = MarkdownDocsOptions { render_stats: false, ..markdown_typedoc_options() };
    let out = generate_markdown(&stats_docs(), &options);
    assert_markdown_map_snapshot("render_stats_false_omits_stats_summary", &out);

    // The gate also drops the trailing separator, so no stray blank line is
    // left where the stats summary used to be.
}

#[test]
fn render_stats_false_omits_html_stats_block() {
    let options = MarkdownDocsOptions {
        path_strategy: MarkdownPathStrategy::TypeDoc,
        render_style: MarkdownRenderStyle::Html,
        render_stats: false,
        ..MarkdownDocsOptions::default()
    };
    let out = generate_markdown(&stats_docs(), &options);
    assert_markdown_map_snapshot("render_stats_false_omits_html_stats_block", &out);

    // The option also gates the HTML render style's stats block.
}

#[test]
fn render_generated_by_keeps_attribution_by_default() {
    let markdown_out = generate_markdown(&stats_docs(), &markdown_typedoc_options());
    assert_markdown_map_snapshot(
        "render_generated_by_keeps_attribution_by_default__markdown_out",
        &markdown_out,
    );
    let html_out = generate_markdown(&stats_docs(), &html_typedoc_options());
    assert_markdown_map_snapshot(
        "render_generated_by_keeps_attribution_by_default__html_out",
        &html_out,
    );
}

#[test]
fn render_generated_by_false_omits_typedoc_root_attribution() {
    let options = MarkdownDocsOptions {
        render_generated_by: false,
        render_stats: false,
        ..markdown_typedoc_options()
    };
    let out = generate_markdown(&stats_docs(), &options);
    assert_markdown_map_snapshot("render_generated_by_false_omits_typedoc_root_attribution", &out);
    let root = out.get("index.md").unwrap();

    assert!(root.starts_with("# API Documentation\n\n## Modules\n\n"));
}

#[test]
fn render_generated_by_false_keeps_stats_summary_when_enabled() {
    let options = MarkdownDocsOptions {
        render_generated_by: false,
        render_stats: true,
        ..markdown_typedoc_options()
    };
    let out = generate_markdown(&stats_docs(), &options);
    assert_markdown_map_snapshot(
        "render_generated_by_false_keeps_stats_summary_when_enabled",
        &out,
    );
    assert_markdown_map_snapshot(
        "render_generated_by_false_keeps_stats_summary_when_enabled",
        &out,
    );
}

#[test]
fn render_generated_by_false_omits_flat_and_category_root_attribution() {
    let flat_options =
        MarkdownDocsOptions { render_generated_by: false, ..MarkdownDocsOptions::default() };
    let flat_out = generate_markdown(&stats_docs(), &flat_options);
    assert_markdown_map_snapshot(
        "render_generated_by_false_omits_flat_and_category_root_attribution__flat_out",
        &flat_out,
    );
    let flat_root = flat_out.get("index.md").unwrap();

    assert!(
        flat_root.starts_with("# API Documentation\n\n> Use search scopes like `@api transform`")
    );

    let category_options = MarkdownDocsOptions {
        group_by: "category".to_string(),
        render_style: MarkdownRenderStyle::Markdown,
        render_generated_by: false,
        render_stats: false,
        ..MarkdownDocsOptions::default()
    };
    let category_out = generate_markdown(&stats_docs(), &category_options);
    assert_markdown_map_snapshot(
        "render_generated_by_false_omits_flat_and_category_root_attribution__category_out",
        &category_out,
    );
    let category_root = category_out.get("index.md").unwrap();

    assert!(category_root.starts_with("# API Documentation\n\n## [Functions]"));
}
