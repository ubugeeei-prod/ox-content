use super::*;

#[test]
fn typedoc_html_type_alias_without_returns_tag_renders_return_section() {
    let mut entry =
        test_entry("OnPluginExtension", "type", "/repo/src/plugin.ts", "Plugin extension hook.");
    entry.signature = Some(
            "export type OnPluginExtension<G> = (ctx: Readonly<CommandContext<G>>, cmd: Readonly<Command<G>>) => Awaitable<void>"
                .to_string(),
        );
    entry.params = vec![param("ctx", "Readonly<CommandContext<G>>")];
    entry.returns = Some(ApiReturnDoc {
        type_annotation: "Awaitable<void>".to_string(),
        ..ApiReturnDoc::default()
    });

    let out = generate_markdown(&type_link_module(entry), &html_typedoc_options());
    assert_markdown_map_snapshot(
        "typedoc_html_type_alias_without_returns_tag_renders_return_section",
        &out,
    );
    assert_markdown_map_snapshot(
        "typedoc_html_type_alias_without_returns_tag_renders_return_section",
        &out,
    );
}

#[test]
fn typedoc_index_uses_module_description_not_symbol_description() {
    let docs = vec![
        ApiDocModule {
            description: "The entry for gunshi context.".to_string(),
            file: "context".to_string(),
            examples: vec!["```ts\ncreateCommandContext()\n```".to_string()],
            entries: vec![test_entry(
                "CommandContextParams",
                "interface",
                "/repo/src/context.ts",
                "Parameters of createCommandContext.",
            )],
            ..ApiDocModule::default()
        },
        ApiDocModule {
            file: "plugin".to_string(),
            entries: vec![test_entry(
                "plugin",
                "function",
                "/repo/src/plugin.ts",
                "Define a plugin.",
            )],
            ..ApiDocModule::default()
        },
    ];

    let markdown = generate_markdown(
        &docs,
        &MarkdownDocsOptions {
            path_strategy: MarkdownPathStrategy::TypeDoc,
            render_style: MarkdownRenderStyle::Markdown,
            ..MarkdownDocsOptions::default()
        },
    );
    assert_markdown_map_snapshot(
        "typedoc_index_uses_module_description_not_symbol_description__markdown",
        &markdown,
    );
    assert_markdown_map_snapshot(
        "typedoc_index_uses_module_description_not_symbol_description__markdown",
        &markdown,
    );

    // Root module list shows the module-level `@module` description, never a
    // symbol's description, and renders nothing for a module without one.

    // The module index page renders its own description as a paragraph under
    // the heading (followed by the stats line, which starts with `_`); an
    // empty description emits no paragraph, so the heading is followed
    // directly by the stats line.
    let context_index = markdown.get("context/index.md").unwrap();
    assert!(context_index.starts_with(
            "# context\n\nThe entry for gunshi context.\n\n## Example\n\n```ts\ncreateCommandContext()\n```\n\n_"
        ));

    let plugin_index = markdown.get("plugin/index.md").unwrap();
    assert!(plugin_index.starts_with("# plugin\n\n_"));
}

#[test]
fn typedoc_single_entry_root_flatten_uses_root_as_module_index() {
    let docs = vec![ApiDocModule {
        description: "Runtime API.".to_string(),
        file: "default".to_string(),
        entries: vec![
            test_entry("cli", "function", "/repo/src/cli.ts", "Run the CLI."),
            test_entry("Command", "interface", "/repo/src/types.ts", "Runtime command."),
        ],
        ..ApiDocModule::default()
    }];

    let markdown = generate_markdown(
        &docs,
        &MarkdownDocsOptions {
            path_strategy: MarkdownPathStrategy::TypeDoc,
            render_style: MarkdownRenderStyle::Markdown,
            single_entry_root: MarkdownSingleEntryRoot::Flatten,
            ..MarkdownDocsOptions::default()
        },
    );
    assert_markdown_map_snapshot(
        "typedoc_single_entry_root_flatten_uses_root_as_module_index__markdown",
        &markdown,
    );
    assert_markdown_map_snapshot(
        "typedoc_single_entry_root_flatten_uses_root_as_module_index__markdown",
        &markdown,
    );

    assert!(markdown.contains_key("index.md"));
    assert!(!markdown.contains_key("default/index.md"));
    assert!(markdown.contains_key("default/functions/cli.md"));
    assert!(markdown.contains_key("default/interfaces/Command.md"));

    let index = markdown.get("index.md").unwrap();
    assert!(index.starts_with("# API Documentation\n\n"));
}

#[test]
fn typedoc_module_index_renders_module_examples_in_html_style() {
    let docs = vec![ApiDocModule {
        description: "Parser combinator entry point.".to_string(),
        file: "combinators".to_string(),
        examples: vec!["```ts\nstring()\n```".to_string()],
        ..ApiDocModule::default()
    }];

    let markdown = generate_markdown(
        &docs,
        &MarkdownDocsOptions {
            path_strategy: MarkdownPathStrategy::TypeDoc,
            ..MarkdownDocsOptions::default()
        },
    );
    assert_markdown_map_snapshot(
        "typedoc_module_index_renders_module_examples_in_html_style__markdown",
        &markdown,
    );
    assert_markdown_map_snapshot(
        "typedoc_module_index_renders_module_examples_in_html_style__markdown",
        &markdown,
    );
}
