use super::*;

#[test]
fn generate_docs_markdown_renders_return_members() {
    let docs = vec![JsDocsMarkdownModule {
        file: "default".to_string(),
        entries: vec![
            JsDocsMarkdownEntry {
                name: "resolveArgs".to_string(),
                kind: "function".to_string(),
                description: "Resolve.".to_string(),
                returns: Some(JsDocReturn {
                    r#type: "object".to_string(),
                    description: "Resolved args.".to_string(),
                    members: Some(vec![JsDocMember {
                        name: "values".to_string(),
                        kind: "property".to_string(),
                        r#type: Some("ArgValues<A>".to_string()),
                        ..Default::default()
                    }]),
                }),
                file: "/repo/src/resolver.ts".to_string(),
                signature: Some("export function resolveArgs(): object".to_string()),
                ..Default::default()
            },
            JsDocsMarkdownEntry {
                name: "ArgValues".to_string(),
                kind: "type".to_string(),
                file: "/repo/src/types.ts".to_string(),
                signature: Some("export type ArgValues = unknown".to_string()),
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
    assert_string_map_snapshot("generate_docs_markdown_renders_return_members", &markdown);
}

#[test]
fn generate_docs_markdown_renders_type_alias_function_metadata() {
    let docs = vec![JsDocsMarkdownModule {
        file: "default".to_string(),
        entries: vec![
            JsDocsMarkdownEntry {
                name: "CommandRunner".to_string(),
                kind: "type".to_string(),
                description: "Run a command.".to_string(),
                params: Some(vec![JsDocParam {
                    name: "ctx".to_string(),
                    r#type: "Readonly<CommandContext<G>>".to_string(),
                    ..Default::default()
                }]),
                returns: Some(JsDocReturn {
                    r#type: "Awaitable<string | void>".to_string(),
                    description: "CLI output.".to_string(),
                    ..Default::default()
                }),
                file: "/repo/src/types.ts".to_string(),
                signature: Some(
                    "export type CommandRunner<G> = (ctx: Readonly<CommandContext<G>>) => Awaitable<string | void>"
                        .to_string(),
                ),
                ..Default::default()
            },
            JsDocsMarkdownEntry {
                name: "CommandContext".to_string(),
                kind: "type".to_string(),
                params: Some(vec![]),
                file: "/repo/src/context.ts".to_string(),
                signature: Some("export type CommandContext = unknown".to_string()),
                ..Default::default()
            },
        ],
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
        "generate_docs_markdown_renders_type_alias_function_metadata",
        &markdown,
    );
}

#[test]
fn generate_docs_markdown_does_not_escape_return_union_pipe_inside_inline_code() {
    let docs = vec![JsDocsMarkdownModule {
        file: "default".to_string(),
        entries: vec![JsDocsMarkdownEntry {
            name: "cli".to_string(),
            kind: "function".to_string(),
            description: "Run the command.".to_string(),
            params: Some(vec![JsDocParam {
                name: "entry".to_string(),
                r#type: "Command<G> | CommandRunner<G>".to_string(),
                description: "Command entry.".to_string(),
                ..Default::default()
            }]),
            returns: Some(JsDocReturn {
                r#type: "Promise<string | undefined>".to_string(),
                description: "A rendered usage or undefined.".to_string(),
                ..Default::default()
            }),
            file: "/repo/src/cli.ts".to_string(),
            end_line: 5,
            signature: Some(
                "export function cli(entry: Command<G> | CommandRunner<G>): Promise<string | undefined>"
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
        "generate_docs_markdown_does_not_escape_return_union_pipe_inside_inline_code",
        &markdown,
    );
}
