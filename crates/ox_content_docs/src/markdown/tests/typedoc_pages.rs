use super::*;

#[test]
fn typedoc_path_strategy_emits_per_symbol_pages_and_links() {
    let docs = vec![ApiDocModule {
        description: String::new(),
        file: "default".to_string(),
        source_path: String::new(),
        examples: vec![],
        tags: vec![],
        entries: vec![
            ApiDocEntry {
                name: "cli".to_string(),
                kind: "function".to_string(),
                description: "Run with {@link CliOptions} and {@linkcode CliOptions.usageSilent}."
                    .to_string(),
                params: vec![],
                returns: None,
                examples: vec![],
                tags: vec![],
                private: false,
                file: "/repo/src/cli.ts".to_string(),
                line: 1,
                end_line: 10,
                signature: Some("export function cli(options: CliOptions): void".to_string()),
                extends: vec![],
                implements: vec![],
                has_body: false,
                members: vec![],
                type_parameters: vec![],
            },
            ApiDocEntry {
                name: "CliOptions".to_string(),
                kind: "interface".to_string(),
                description: "CLI options.".to_string(),
                params: vec![],
                returns: None,
                examples: vec![],
                tags: vec![],
                private: false,
                file: "/repo/src/types.ts".to_string(),
                line: 1,
                end_line: 20,
                signature: Some("export interface CliOptions".to_string()),
                extends: vec![],
                implements: vec![],
                has_body: false,
                members: vec![ApiDocMember {
                    name: "usageSilent".to_string(),
                    kind: "property".to_string(),
                    description: "Suppress usage output.".to_string(),
                    signature: None,
                    type_annotation: Some("boolean".to_string()),
                    default_value: None,
                    params: vec![],
                    type_parameters: vec![],
                    returns: None,
                    members: vec![],
                    optional: true,
                    readonly: false,
                    r#static: false,
                    private: false,
                    tags: vec![],
                    implementation_of: vec![],
                    line: 5,
                    end_line: 5,
                }],
                type_parameters: vec![],
            },
            test_entry("Plugin", "type", "/repo/src/plugin.ts", "Plugin type."),
            test_entry(
                "CLI_OPTIONS_DEFAULT",
                "variable",
                "/repo/src/constants.ts",
                "Default options.",
            ),
        ],
    }];

    let markdown = generate_markdown(
        &docs,
        &MarkdownDocsOptions {
            path_strategy: MarkdownPathStrategy::TypeDoc,
            ..MarkdownDocsOptions::default()
        },
    );
    let cli_page = markdown.get("default/functions/cli.md").unwrap();
    let options_page = markdown.get("default/interfaces/CliOptions.md").unwrap();
    let module_index = markdown.get("default/index.md").unwrap();

    assert!(markdown.contains_key("index.md"));
    assert!(markdown.contains_key("default/type-aliases/Plugin.md"));
    assert!(markdown.contains_key("default/variables/CLI_OPTIONS_DEFAULT.md"));
    // The module index lists members as a compact table, not bullets with
    // the full signature inlined.
    assert!(module_index.contains("| Function | Description |"));
    assert!(module_index.contains("| [cli](./functions/cli.md) |"));
    assert!(module_index.contains("| [CliOptions](./interfaces/CliOptions.md) |"));
    assert!(!module_index.contains("[`cli`]"));
    assert!(!module_index.contains("export function cli"));
    assert!(cli_page.contains("<a href=\"../interfaces/CliOptions.md\">CliOptions</a>"));
    assert!(cli_page.contains(
            "<a href=\"../interfaces/CliOptions.md#property-usagesilent\"><code>CliOptions.usageSilent</code></a>"
        ));
    assert!(options_page.contains("<tr id=\"property-usagesilent\">"));
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
        members: Vec::new(),
    });

    let mut docs = type_link_module(entry);
    if let Some(module) = docs.get_mut(0) {
        module.entries.push(ApiDocEntry {
            name: "CommandContext".to_string(),
            kind: "interface".to_string(),
            description: "Command context.".to_string(),
            params: Vec::new(),
            returns: None,
            examples: Vec::new(),
            tags: Vec::new(),
            private: false,
            file: "/repo/src/context.ts".to_string(),
            line: 1,
            end_line: 1,
            signature: Some(
                "export interface CommandContext<G = DefaultGunshiParams> {}".to_string(),
            ),
            extends: Vec::new(),
            implements: Vec::new(),
            has_body: false,
            members: Vec::new(),
            type_parameters: Vec::new(),
        });
        module.entries.push(type_stub("CommandContextCore"));
    }
    let out = generate_markdown(&docs, &markdown_typedoc_options());
    let page = out.get("combinators/type-aliases/CommandRunner.md").unwrap();

    assert!(page.contains("## Parameters"));
    assert!(page.contains("[`CommandContext`](../interfaces/CommandContext.md)\\<`G`\\>"));
    assert!(page.contains("## Returns"));
    assert!(!page.contains("| `ctx` | `unknown` |"));
    assert!(page.contains("`Awaitable<string | void>`"));
    assert!(page.contains("CLI output."));
    assert!(!page.contains("`unknown`"));
    assert!(page.contains("[`CommandContext`](../interfaces/CommandContext.md)"));
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
            optional: false,
            default_value: None,
        },
        ApiParamDoc {
            name: "cmd".to_string(),
            type_annotation: "Readonly<Command<G>>".to_string(),
            description: "The command.".to_string(),
            optional: false,
            default_value: None,
        },
    ];
    entry.returns = Some(ApiReturnDoc {
        type_annotation: "Awaitable<void>".to_string(),
        description: String::new(),
        members: Vec::new(),
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
    let page = out.get("combinators/type-aliases/OnPluginExtension.md").unwrap();

    assert!(page.contains("## Parameters"));
    assert!(page.contains("The command context."));
    assert!(page.contains("The command."));
    assert!(page.contains("[`CommandContext`](../interfaces/CommandContext.md)"));
    assert!(page.contains("[`Command`](../interfaces/Command.md)"));
    assert!(page.contains("## Returns\n\n`Awaitable<void>`"));
    assert!(!page.contains("`unknown`"));
}
