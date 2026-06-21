use super::*;

#[test]
fn typedoc_markdown_renders_object_literal_parameter_members() {
    let options = MarkdownDocsOptions {
        parameters_format: MarkdownDisplayFormat::Table,
        ..markdown_typedoc_options()
    };
    let out = generate_markdown(&type_link_module(object_literal_parameter_entry()), &options);
    assert_markdown_map_snapshot("typedoc_markdown_renders_object_literal_parameter_members", &out);
}

#[test]
fn typedoc_html_renders_object_literal_parameter_members() {
    let options = MarkdownDocsOptions {
        parameters_format: MarkdownDisplayFormat::Table,
        ..html_typedoc_options()
    };
    let out = generate_markdown(&type_link_module(object_literal_parameter_entry()), &options);
    assert_markdown_map_snapshot("typedoc_html_renders_object_literal_parameter_members", &out);
}

#[test]
fn typedoc_module_index_source_link_uses_source_path() {
    let options = MarkdownDocsOptions {
        github_url: Some("https://github.com/x/y".to_string()),
        ..markdown_typedoc_options()
    };
    let out =
        generate_markdown(&module_with_source_path("/repo/packages/x/src/index.ts"), &options);
    assert_markdown_map_snapshot("typedoc_module_index_source_link_uses_source_path", &out);

    // Links to the real entry-point file, never the dead `blob/main/default`.
}

#[test]
fn typedoc_module_index_omits_source_link_without_source_path() {
    let options = MarkdownDocsOptions {
        github_url: Some("https://github.com/x/y".to_string()),
        ..markdown_typedoc_options()
    };
    let out = generate_markdown(&module_with_source_path(""), &options);
    assert_markdown_map_snapshot(
        "typedoc_module_index_omits_source_link_without_source_path",
        &out,
    );
    assert_markdown_map_snapshot(
        "typedoc_module_index_omits_source_link_without_source_path",
        &out,
    );

    // No source path -> no module source line (and no dead link).
}
