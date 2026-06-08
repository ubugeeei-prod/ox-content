use super::*;

#[test]
fn typedoc_module_index_resolves_links_in_table_cells() {
    let docs = vec![ApiDocModule {
        description: String::new(),
        file: "default".to_string(),
        source_path: String::new(),
        examples: vec![],
        tags: vec![],
        entries: vec![
            test_entry(
                "Args",
                "interface",
                "/repo/src/args.ts",
                "An object that contains {@link ArgSchema | argument schema}.",
            ),
            test_entry("ArgSchema", "interface", "/repo/src/args.ts", "A schema."),
        ],
    }];

    let out = generate_markdown(
        &docs,
        &MarkdownDocsOptions {
            path_strategy: MarkdownPathStrategy::TypeDoc,
            ..MarkdownDocsOptions::default()
        },
    );
    let index = out.get("default/index.md").unwrap();

    // The `{@link}` is resolved to a Markdown link inside the cell, not left raw.
    assert!(index.contains("[argument schema](./interfaces/ArgSchema.md)"));
    assert!(!index.contains("{@link"));
}

#[test]
fn typedoc_module_index_collapses_overloads_to_one_row() {
    let docs = vec![ApiDocModule {
        description: String::new(),
        file: "default".to_string(),
        source_path: String::new(),
        examples: vec![],
        tags: vec![],
        entries: vec![
            test_entry("cli", "function", "/repo/src/cli.ts", "Run the command."),
            test_entry("cli", "function", "/repo/src/cli.ts", "Run the command."),
        ],
    }];

    let out = generate_markdown(
        &docs,
        &MarkdownDocsOptions {
            path_strategy: MarkdownPathStrategy::TypeDoc,
            ..MarkdownDocsOptions::default()
        },
    );
    let index = out.get("default/index.md").unwrap();

    assert_eq!(index.matches("| [cli](./functions/cli.md) |").count(), 1);
}

#[test]
fn typedoc_module_index_escapes_pipes_in_table_cells() {
    let docs = vec![ApiDocModule {
        description: String::new(),
        file: "default".to_string(),
        source_path: String::new(),
        examples: vec![],
        tags: vec![],
        entries: vec![test_entry("toUnion", "function", "/repo/src/u.ts", "Returns A | B.")],
    }];

    let out = generate_markdown(
        &docs,
        &MarkdownDocsOptions {
            path_strategy: MarkdownPathStrategy::TypeDoc,
            ..MarkdownDocsOptions::default()
        },
    );
    let index = out.get("default/index.md").unwrap();

    assert!(index.contains("Returns A \\| B."));
}

#[test]
fn typedoc_path_strategy_uses_clean_base_path_and_module_scope() {
    let docs = vec![
        ApiDocModule {
            description: String::new(),
            file: "default".to_string(),
            source_path: String::new(),
            examples: vec![],
            tags: vec![],
            entries: vec![
                test_entry("Command", "interface", "/repo/src/default.ts", "Default command."),
                test_entry(
                    "runDefault",
                    "function",
                    "/repo/src/default.ts",
                    "Runs {@link Command}.",
                ),
            ],
        },
        ApiDocModule {
            description: String::new(),
            file: "plugin".to_string(),
            source_path: String::new(),
            examples: vec![],
            tags: vec![],
            entries: vec![
                test_entry("Command", "interface", "/repo/src/plugin.ts", "Plugin command."),
                test_entry("runPlugin", "function", "/repo/src/plugin.ts", "Runs {@link Command}."),
            ],
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
    let default_page = markdown.get("default/functions/runDefault.md").unwrap();
    let plugin_page = markdown.get("plugin/functions/runPlugin.md").unwrap();
    let index = markdown.get("index.md").unwrap();

    assert!(index.contains("[default](/api/default)"));
    assert!(default_page.contains("<a href=\"/api/default/interfaces/Command\">Command</a>"));
    assert!(plugin_page.contains("<a href=\"/api/plugin/interfaces/Command\">Command</a>"));
    assert!(!default_page.contains(".md"));
}
