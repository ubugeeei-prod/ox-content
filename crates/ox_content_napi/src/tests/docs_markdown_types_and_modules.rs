use super::*;

#[test]
fn generate_docs_markdown_renders_type_alias_return_without_description() {
    let docs = vec![JsDocsMarkdownModule {
        file: "default".to_string(),
        entries: vec![JsDocsMarkdownEntry {
            name: "OnPluginExtension".to_string(),
            kind: "type".to_string(),
            description: "Plugin extension hook.".to_string(),
            params: Some(vec![
                JsDocParam {
                    name: "ctx".to_string(),
                    r#type: "Readonly<CommandContext<G>>".to_string(),
                    description: "The command context.".to_string(),
                    ..Default::default()
                },
                JsDocParam {
                    name: "cmd".to_string(),
                    r#type: "Readonly<Command<G>>".to_string(),
                    description: "The command.".to_string(),
                    ..Default::default()
                },
            ]),
            returns: Some(JsDocReturn {
                r#type: "Awaitable<void>".to_string(),
                ..Default::default()
            }),
            file: "/repo/src/plugin.ts".to_string(),
            end_line: 5,
            signature: Some(
                "export type OnPluginExtension<G> = (ctx: Readonly<CommandContext<G>>, cmd: Readonly<Command<G>>) => Awaitable<void>"
                    .to_string(),
            ),
            ..Default::default()
        }],
        ..Default::default()
    }];

    let markdown = generate_docs_markdown(
        docs,
        Some(JsDocsMarkdownOptions {
            group_by: Some("file".to_string()),
            path_strategy: Some("typedoc".to_string()),
            render_style: Some("markdown".to_string()),
            parameters_format: Some("table".to_string()),
            ..Default::default()
        }),
    );
    assert_string_map_snapshot(
        "generate_docs_markdown_renders_type_alias_return_without_description",
        &markdown,
    );
}

#[test]
fn generate_docs_markdown_renders_index_signature_members() {
    let docs = vec![JsDocsMarkdownModule {
        file: "default".to_string(),
        entries: vec![
            JsDocsMarkdownEntry {
                name: "ArgSchema".to_string(),
                kind: "interface".to_string(),
                description: "Value type.".to_string(),
                file: "/repo/src/args.ts".to_string(),
                signature: Some("export interface ArgSchema".to_string()),
                ..Default::default()
            },
            JsDocsMarkdownEntry {
                name: "Args".to_string(),
                kind: "interface".to_string(),
                description: "Arguments.".to_string(),
                file: "/repo/src/args.ts".to_string(),
                end_line: 5,
                signature: Some("export interface Args".to_string()),
                members: Some(vec![JsDocMember {
                    name: "[option: string]".to_string(),
                    kind: "indexSignature".to_string(),
                    description: "Argument schema by option name.".to_string(),
                    signature: Some("readonly [option: string]: ArgSchema".to_string()),
                    r#type: Some("ArgSchema".to_string()),
                    params: Some(vec![JsDocParam {
                        name: "option".to_string(),
                        r#type: "string".to_string(),
                        ..Default::default()
                    }]),
                    readonly: Some(true),
                    line: 4,
                    end_line: 4,
                    ..Default::default()
                }]),
                ..Default::default()
            },
        ],
        ..Default::default()
    }];

    let markdown = generate_docs_markdown(
        docs,
        Some(JsDocsMarkdownOptions {
            path_strategy: Some("typedoc".to_string()),
            render_style: Some("markdown".to_string()),
            ..Default::default()
        }),
    );
    assert_string_map_snapshot("generate_docs_markdown_renders_index_signature_members", &markdown);
}

#[test]
fn generate_docs_markdown_renders_module_description_in_typedoc_index() {
    let docs = vec![JsDocsMarkdownModule {
        description: Some("The entry for gunshi context.".to_string()),
        file: "context".to_string(),
        examples: Some(vec!["```ts\ncreateCommandContext()\n```".to_string()]),
        entries: vec![JsDocsMarkdownEntry {
            name: "createCommandContext".to_string(),
            kind: "function".to_string(),
            description: "Creates a command context.".to_string(),
            file: "/repo/src/context.ts".to_string(),
            end_line: 10,
            signature: Some("export function createCommandContext(): void".to_string()),
            ..Default::default()
        }],
        ..Default::default()
    }];

    let markdown = generate_docs_markdown(
        docs,
        Some(JsDocsMarkdownOptions {
            group_by: Some("file".to_string()),
            link_style: Some("markdown".to_string()),
            path_strategy: Some("typedoc".to_string()),
            render_style: Some("markdown".to_string()),
            ..Default::default()
        }),
    );
    assert_string_map_snapshot(
        "generate_docs_markdown_renders_module_description_in_typedoc_index",
        &markdown,
    );
}
