use super::*;

#[test]
fn typedoc_path_strategy_emits_per_symbol_pages_and_links() {
    let docs = vec![ApiDocModule {
        file: "default".to_string(),
        entries: vec![
            ApiDocEntry {
                name: "cli".to_string(),
                kind: "function".to_string(),
                description: "Run with {@link CliOptions} and {@linkcode CliOptions.usageSilent}."
                    .to_string(),
                file: "/repo/src/cli.ts".to_string(),
                end_line: 10,
                signature: Some("export function cli(options: CliOptions): void".to_string()),
                ..ApiDocEntry::default()
            },
            ApiDocEntry {
                name: "CliOptions".to_string(),
                kind: "interface".to_string(),
                description: "CLI options.".to_string(),
                file: "/repo/src/types.ts".to_string(),
                end_line: 20,
                signature: Some("export interface CliOptions".to_string()),
                members: vec![ApiDocMember {
                    name: "usageSilent".to_string(),
                    kind: "property".to_string(),
                    description: "Suppress usage output.".to_string(),
                    type_annotation: Some("boolean".to_string()),
                    optional: true,
                    line: 5,
                    end_line: 5,
                    ..ApiDocMember::default()
                }],
                ..ApiDocEntry::default()
            },
            test_entry("Plugin", "type", "/repo/src/plugin.ts", "Plugin type."),
            test_entry(
                "CLI_OPTIONS_DEFAULT",
                "variable",
                "/repo/src/constants.ts",
                "Default options.",
            ),
        ],
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
        "typedoc_path_strategy_emits_per_symbol_pages_and_links__markdown",
        &markdown,
    );
    assert_markdown_map_snapshot(
        "typedoc_path_strategy_emits_per_symbol_pages_and_links__markdown",
        &markdown,
    );

    assert!(markdown.contains_key("index.md"));
    assert!(markdown.contains_key("default/type-aliases/Plugin.md"));
    assert!(markdown.contains_key("default/variables/CLI_OPTIONS_DEFAULT.md"));
    // The module index lists members as a compact table, not bullets with
    // the full signature inlined.
}

#[test]
fn typedoc_type_alias_renders_concrete_function_metadata() {
    let mut entry = test_entry("CommandRunner", "type", "/repo/src/types.ts", "Run a command.");
    entry.signature = Some(
            "export type CommandRunner<G> = (ctx: Readonly<CommandContext<G>>) => Awaitable<string | void>"
                .to_string(),
        );
    entry.params = vec![param("ctx", "Readonly<CommandContext<G>>")];
    entry.returns = Some(ApiReturnDoc {
        type_annotation: "Awaitable<string | void>".to_string(),
        description: "CLI output.".to_string(),
        ..ApiReturnDoc::default()
    });

    let mut docs = type_link_module(entry);
    if let Some(module) = docs.get_mut(0) {
        module.entries.push(ApiDocEntry {
            name: "CommandContext".to_string(),
            kind: "interface".to_string(),
            description: "Command context.".to_string(),
            file: "/repo/src/context.ts".to_string(),
            signature: Some(
                "export interface CommandContext<G = DefaultGunshiParams> {}".to_string(),
            ),
            ..ApiDocEntry::default()
        });
        module.entries.push(type_stub("CommandContextCore"));
    }
    let out = generate_markdown(&docs, &markdown_typedoc_options());
    assert_markdown_map_snapshot("typedoc_type_alias_renders_concrete_function_metadata", &out);
}

#[test]
fn typedoc_type_alias_without_returns_tag_renders_return_section() {
    let mut entry =
        test_entry("OnPluginExtension", "type", "/repo/src/plugin.ts", "Plugin extension hook.");
    entry.signature = Some(
            "export type OnPluginExtension<G> = (ctx: Readonly<CommandContext<G>>, cmd: Readonly<Command<G>>) => Awaitable<void>"
                .to_string(),
        );
    entry.params = vec![
        ApiParamDoc {
            name: "ctx".to_string(),
            type_annotation: "Readonly<CommandContext<G>>".to_string(),
            description: "The command context.".to_string(),
            ..ApiParamDoc::default()
        },
        ApiParamDoc {
            name: "cmd".to_string(),
            type_annotation: "Readonly<Command<G>>".to_string(),
            description: "The command.".to_string(),
            ..ApiParamDoc::default()
        },
    ];
    entry.returns = Some(ApiReturnDoc {
        type_annotation: "Awaitable<void>".to_string(),
        ..ApiReturnDoc::default()
    });

    let mut docs = type_link_module(entry);
    if let Some(module) = docs.get_mut(0) {
        let mut command_context =
            test_entry("CommandContext", "interface", "/repo/src/context.ts", "");
        command_context.signature = Some("export interface CommandContext<G> {}".to_string());
        let mut command = test_entry("Command", "interface", "/repo/src/command.ts", "");
        command.signature = Some("export interface Command<G> {}".to_string());
        module.entries.push(command_context);
        module.entries.push(command);
    }
    let out = generate_markdown(&docs, &markdown_typedoc_options());
    assert_markdown_map_snapshot(
        "typedoc_type_alias_without_returns_tag_renders_return_section",
        &out,
    );
    assert_markdown_map_snapshot(
        "typedoc_type_alias_without_returns_tag_renders_return_section",
        &out,
    );
}
