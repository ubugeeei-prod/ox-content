use super::*;

#[test]
fn render_style_defaults_to_html() {
    let out = generate_markdown(&pure_test_docs(), &MarkdownDocsOptions::default());
    assert_markdown_map_snapshot("render_style_defaults_to_html", &out);
}

#[test]
fn file_group_index_links_default_to_markdown_extension() {
    let markdown = generate_markdown(&link_test_docs(), &MarkdownDocsOptions::default());
    assert_markdown_map_snapshot(
        "file_group_index_links_default_to_markdown_extension__markdown",
        &markdown,
    );
    assert_markdown_map_snapshot(
        "file_group_index_links_default_to_markdown_extension__markdown",
        &markdown,
    );
}

#[test]
fn file_group_index_links_support_clean_urls() {
    let markdown = generate_markdown(
        &link_test_docs(),
        &MarkdownDocsOptions {
            link_style: MarkdownLinkStyle::Clean,
            ..MarkdownDocsOptions::default()
        },
    );
    assert_markdown_map_snapshot("file_group_index_links_support_clean_urls__markdown", &markdown);
}

#[test]
fn file_group_index_links_support_clean_urls_with_base_path() {
    let markdown = generate_markdown(
        &link_test_docs(),
        &MarkdownDocsOptions {
            link_style: MarkdownLinkStyle::Clean,
            base_path: Some("/api-ox".to_string()),
            ..MarkdownDocsOptions::default()
        },
    );
    assert_markdown_map_snapshot(
        "file_group_index_links_support_clean_urls_with_base_path__markdown",
        &markdown,
    );
    assert_markdown_map_snapshot(
        "file_group_index_links_support_clean_urls_with_base_path__markdown",
        &markdown,
    );
}

#[test]
fn category_links_use_configured_link_policy() {
    let markdown = generate_markdown(
        &link_test_docs(),
        &MarkdownDocsOptions {
            group_by: "category".to_string(),
            link_style: MarkdownLinkStyle::Clean,
            base_path: Some("/api-ox".to_string()),
            ..MarkdownDocsOptions::default()
        },
    );
    assert_markdown_map_snapshot("category_links_use_configured_link_policy__markdown", &markdown);
}

#[test]
fn symbol_cross_file_links_use_configured_link_policy() {
    let markdown = generate_markdown(
        &link_test_docs(),
        &MarkdownDocsOptions {
            link_style: MarkdownLinkStyle::Clean,
            base_path: Some("/api-ox".to_string()),
            ..MarkdownDocsOptions::default()
        },
    );
    assert_markdown_map_snapshot(
        "symbol_cross_file_links_use_configured_link_policy__markdown",
        &markdown,
    );
    assert_markdown_map_snapshot(
        "symbol_cross_file_links_use_configured_link_policy__markdown",
        &markdown,
    );
}

#[test]
fn jsdoc_inline_links_render_across_doc_fields() {
    let docs = vec![
        ApiDocModule {
            file: "/repo/src/agent.ts".to_string(),
            entries: vec![test_entry(
                "AgentProfile",
                "interface",
                "/repo/src/agent.ts",
                "Agent profile.",
            )],
            ..ApiDocModule::default()
        },
        ApiDocModule {
            file: "/repo/src/command.ts".to_string(),
            entries: vec![ApiDocEntry {
                name: "Command".to_string(),
                kind: "interface".to_string(),
                description: "Runtime command.".to_string(),
                file: "/repo/src/command.ts".to_string(),
                end_line: 10,
                signature: Some("export interface Command".to_string()),
                members: vec![ApiDocMember {
                    name: "args".to_string(),
                    kind: "property".to_string(),
                    description: "All {@linkcode Command.args} names use kebab-case.".to_string(),
                    type_annotation: Some("Record<string, unknown>".to_string()),
                    line: 5,
                    end_line: 5,
                    ..ApiDocMember::default()
                }],
                ..ApiDocEntry::default()
            }],
            ..ApiDocModule::default()
        },
        ApiDocModule {
            file: "/repo/src/build.ts".to_string(),
            entries: vec![ApiDocEntry {
                name: "buildCommand".to_string(),
                kind: "function".to_string(),
                description: "Builds {@linkcode Command | command} metadata.".to_string(),
                params: vec![ApiParamDoc {
                    name: "entry".to_string(),
                    type_annotation: "Command".to_string(),
                    description: "A {@linkcode Command | entry command}".to_string(),
                    ..ApiParamDoc::default()
                }],
                returns: Some(ApiReturnDoc {
                    type_annotation: "AgentProfile".to_string(),
                    description: "An {@link AgentProfile} result.".to_string(),
                    ..ApiReturnDoc::default()
                }),
                tags: vec![
                    ApiDocTag {
                        tag: "see".to_string(),
                        value: "delegated to {@link https://github.com/unjs/std-env | std-env}"
                            .to_string(),
                    },
                    ApiDocTag {
                        tag: "remarks".to_string(),
                        value: "Falls back to {@link MissingSymbol | missing}.".to_string(),
                    },
                ],
                file: "/repo/src/build.ts".to_string(),
                end_line: 20,
                signature: Some(
                    "export function buildCommand(entry: Command): AgentProfile".to_string(),
                ),
                ..ApiDocEntry::default()
            }],
            ..ApiDocModule::default()
        },
    ];

    let markdown = generate_markdown(&docs, &MarkdownDocsOptions::default());
    assert_markdown_map_snapshot(
        "jsdoc_inline_links_render_across_doc_fields__markdown",
        &markdown,
    );
    assert_markdown_map_snapshot(
        "jsdoc_inline_links_render_across_doc_fields__markdown",
        &markdown,
    );
}
