use super::*;

#[test]
fn typedoc_module_index_resolves_links_in_table_cells() {
    let docs = vec![ApiDocModule {
        file: "default".to_string(),
        entries: vec![
            test_entry(
                "Args",
                "interface",
                "/repo/src/args.ts",
                "An object that contains {@link ArgSchema | argument schema}.",
            ),
            test_entry("ArgSchema", "interface", "/repo/src/args.ts", "A schema."),
        ],
        ..ApiDocModule::default()
    }];

    let out = generate_markdown(
        &docs,
        &MarkdownDocsOptions {
            path_strategy: MarkdownPathStrategy::TypeDoc,
            ..MarkdownDocsOptions::default()
        },
    );
    assert_markdown_map_snapshot("typedoc_module_index_resolves_links_in_table_cells", &out);

    // The `{@link}` is resolved to a Markdown link inside the cell, not left raw.
}

#[test]
fn typedoc_module_index_collapses_overloads_to_one_row() {
    let docs = vec![ApiDocModule {
        file: "default".to_string(),
        entries: vec![
            test_entry("cli", "function", "/repo/src/cli.ts", "Run the command."),
            test_entry("cli", "function", "/repo/src/cli.ts", "Run the command."),
        ],
        ..ApiDocModule::default()
    }];

    let out = generate_markdown(
        &docs,
        &MarkdownDocsOptions {
            path_strategy: MarkdownPathStrategy::TypeDoc,
            ..MarkdownDocsOptions::default()
        },
    );
    assert_markdown_map_snapshot("typedoc_module_index_collapses_overloads_to_one_row", &out);
    let index = out.get("default/index.md").unwrap();

    assert_eq!(index.matches("| [cli](./functions/cli.md) |").count(), 1);
}

#[test]
fn typedoc_module_index_escapes_pipes_in_table_cells() {
    let docs = vec![ApiDocModule {
        file: "default".to_string(),
        entries: vec![test_entry("toUnion", "function", "/repo/src/u.ts", "Returns A | B.")],
        ..ApiDocModule::default()
    }];

    let out = generate_markdown(
        &docs,
        &MarkdownDocsOptions {
            path_strategy: MarkdownPathStrategy::TypeDoc,
            ..MarkdownDocsOptions::default()
        },
    );
    assert_markdown_map_snapshot("typedoc_module_index_escapes_pipes_in_table_cells", &out);
}

#[test]
fn typedoc_path_strategy_uses_clean_base_path_and_module_scope() {
    let docs = vec![
        ApiDocModule {
            file: "default".to_string(),
            entries: vec![
                test_entry("Command", "interface", "/repo/src/default.ts", "Default command."),
                test_entry(
                    "runDefault",
                    "function",
                    "/repo/src/default.ts",
                    "Runs {@link Command}.",
                ),
            ],
            ..ApiDocModule::default()
        },
        ApiDocModule {
            file: "plugin".to_string(),
            entries: vec![
                test_entry("Command", "interface", "/repo/src/plugin.ts", "Plugin command."),
                test_entry("runPlugin", "function", "/repo/src/plugin.ts", "Runs {@link Command}."),
            ],
            ..ApiDocModule::default()
        },
    ];

    let markdown = generate_markdown(
        &docs,
        &MarkdownDocsOptions {
            link_style: MarkdownLinkStyle::Clean,
            base_path: Some("/api".to_string()),
            path_strategy: MarkdownPathStrategy::TypeDoc,
            ..MarkdownDocsOptions::default()
        },
    );
    assert_markdown_map_snapshot(
        "typedoc_path_strategy_uses_clean_base_path_and_module_scope__markdown",
        &markdown,
    );
    assert_markdown_map_snapshot(
        "typedoc_path_strategy_uses_clean_base_path_and_module_scope__markdown",
        &markdown,
    );
}
