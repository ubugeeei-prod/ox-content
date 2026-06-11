use super::*;

#[test]
fn generate_docs_markdown_renders_return_members() {
    let docs = vec![JsDocsMarkdownModule {
        description: None,
        file: "default".to_string(),
        source_path: None,
        examples: None,
        tags: None,
        entries: vec![
            JsDocsMarkdownEntry {
                name: "resolveArgs".to_string(),
                kind: "function".to_string(),
                description: "Resolve.".to_string(),
                params: None,
                returns: Some(JsDocReturn {
                    r#type: "object".to_string(),
                    description: "Resolved args.".to_string(),
                    members: Some(vec![JsDocMember {
                        name: "values".to_string(),
                        kind: "property".to_string(),
                        description: String::new(),
                        signature: None,
                        r#type: Some("ArgValues<A>".to_string()),
                        r#default: None,
                        params: None,
                        type_parameters: None,
                        returns: None,
                        throws: None,
                        members: None,
                        optional: Some(false),
                        readonly: Some(false),
                        r#static: Some(false),
                        private: Some(false),
                        tags: None,
                        implementation_of: None,
                        line: 1,
                        end_line: 1,
                    }]),
                }),
                throws: None,
                examples: None,
                tags: None,
                private: false,
                file: "/repo/src/resolver.ts".to_string(),
                line: 1,
                end_line: 1,
                signature: Some("export function resolveArgs(): object".to_string()),
                extends: None,
                implements: None,
                has_body: None,
                members: None,
                type_parameters: None,
            },
            JsDocsMarkdownEntry {
                name: "ArgValues".to_string(),
                kind: "type".to_string(),
                description: String::new(),
                params: None,
                returns: None,
                throws: None,
                examples: None,
                tags: None,
                private: false,
                file: "/repo/src/types.ts".to_string(),
                line: 1,
                end_line: 1,
                signature: Some("export type ArgValues = unknown".to_string()),
                extends: None,
                implements: None,
                has_body: None,
                members: None,
                type_parameters: None,
            },
        ],
    }];

    let markdown = generate_docs_markdown(
        docs,
        Some(JsDocsMarkdownOptions {
            path_strategy: Some("typedoc".to_string()),
            render_style: Some("markdown".to_string()),
            ..Default::default()
        }),
    );
    let page = markdown.get("default/functions/resolveArgs.md").unwrap();

    assert!(page.contains("## Returns\n\n`object` — Resolved args."));
    assert!(page.contains("### values\n\n```ts\nvalues: ArgValues<A>;\n```"));
}

#[test]
fn generate_docs_markdown_renders_type_alias_function_metadata() {
    let docs = vec![JsDocsMarkdownModule {
            description: None,
            file: "default".to_string(),
            source_path: None,
            examples: None,
            tags: None,
            entries: vec![
                JsDocsMarkdownEntry {
                    name: "CommandRunner".to_string(),
                    kind: "type".to_string(),
                    description: "Run a command.".to_string(),
                    params: Some(vec![JsDocParam {
                        name: "ctx".to_string(),
                        r#type: "Readonly<CommandContext<G>>".to_string(),
                        description: String::new(),
                        optional: Some(false),
                        r#default: None,
                    }]),
                    returns: Some(JsDocReturn {
                        r#type: "Awaitable<string | void>".to_string(),
                        description: "CLI output.".to_string(),
                        members: None,
                    }),
                    throws: None,
                    examples: None,
                    tags: None,
                    private: false,
                    file: "/repo/src/types.ts".to_string(),
                    line: 1,
                    end_line: 1,
                    signature: Some(
                        "export type CommandRunner<G> = (ctx: Readonly<CommandContext<G>>) => Awaitable<string | void>"
                            .to_string(),
                    ),
                    extends: None,
                    implements: None,
                    has_body: None,
                    members: None,
                    type_parameters: None,
                },
                JsDocsMarkdownEntry {
                    name: "CommandContext".to_string(),
                    kind: "type".to_string(),
                    description: String::new(),
                    params: Some(vec![]),
                    returns: None,
                    throws: None,
                    examples: None,
                    tags: None,
                    private: false,
                    file: "/repo/src/context.ts".to_string(),
                    line: 1,
                    end_line: 1,
                    signature: Some("export type CommandContext = unknown".to_string()),
                    extends: None,
                    implements: None,
                    has_body: None,
                    members: None,
                    type_parameters: None,
                },
            ],
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
    let page = markdown.get("default/type-aliases/CommandRunner.md").unwrap();

    assert!(page.contains("## Parameters"));
    assert!(page.contains("Readonly"));
    assert!(page.contains("CommandContext"));
    assert!(page.contains("## Returns"));
    assert!(!page.contains("| `ctx` | `unknown` |"));
    assert!(page.contains("`Awaitable<string | void>`"));
    assert!(page.contains("CLI output."));
    assert!(!page.contains("`unknown`"));
}

#[test]
fn generate_docs_markdown_does_not_escape_return_union_pipe_inside_inline_code() {
    let docs = vec![JsDocsMarkdownModule {
            description: None,
            file: "default".to_string(),
            source_path: None,
            examples: None,
            tags: None,
            entries: vec![JsDocsMarkdownEntry {
                name: "cli".to_string(),
                kind: "function".to_string(),
                description: "Run the command.".to_string(),
                params: Some(vec![JsDocParam {
                    name: "entry".to_string(),
                    r#type: "Command<G> | CommandRunner<G>".to_string(),
                    description: "Command entry.".to_string(),
                    optional: Some(false),
                    r#default: None,
                }]),
                returns: Some(JsDocReturn {
                    r#type: "Promise<string | undefined>".to_string(),
                    description: "A rendered usage or undefined.".to_string(),
                    members: None,
                }),
                throws: None,
                examples: None,
                tags: None,
                private: false,
                file: "/repo/src/cli.ts".to_string(),
                line: 1,
                end_line: 5,
                signature: Some(
                    "export function cli(entry: Command<G> | CommandRunner<G>): Promise<string | undefined>"
                        .to_string(),
                ),
                extends: None,
                implements: None,
                has_body: None,
                members: None,
                type_parameters: None,
            }],
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
    let page = markdown.get("default/functions/cli.md").unwrap();

    assert!(page.contains("| `entry` | `Command<G> \\| CommandRunner<G>` | Command entry. |"));
    assert!(page
        .contains("## Returns\n\n`Promise<string | undefined>` — A rendered usage or undefined."));
    assert!(!page.contains("`Promise<string \\| undefined>`"));
}
