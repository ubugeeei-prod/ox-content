use super::*;

#[test]
fn generate_docs_markdown_renders_type_alias_return_without_description() {
    let docs = vec![JsDocsMarkdownModule {
            description: None,
            file: "default".to_string(),
            source_path: None,
            examples: None,
            tags: None,
            entries: vec![JsDocsMarkdownEntry {
                name: "OnPluginExtension".to_string(),
                kind: "type".to_string(),
                description: "Plugin extension hook.".to_string(),
                params: Some(vec![
                    JsDocParam {
                        name: "ctx".to_string(),
                        r#type: "Readonly<CommandContext<G>>".to_string(),
                        description: "The command context.".to_string(),
                        optional: Some(false),
                        r#default: None,
                    },
                    JsDocParam {
                        name: "cmd".to_string(),
                        r#type: "Readonly<Command<G>>".to_string(),
                        description: "The command.".to_string(),
                        optional: Some(false),
                        r#default: None,
                    },
                ]),
                returns: Some(JsDocReturn {
                    r#type: "Awaitable<void>".to_string(),
                    description: String::new(),
                    members: None,
                }),
                examples: None,
                tags: None,
                private: false,
                file: "/repo/src/plugin.ts".to_string(),
                line: 1,
                end_line: 5,
                signature: Some(
                    "export type OnPluginExtension<G> = (ctx: Readonly<CommandContext<G>>, cmd: Readonly<Command<G>>) => Awaitable<void>"
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
    let page = markdown.get("default/type-aliases/OnPluginExtension.md").unwrap();

    assert!(page.contains("## Parameters"));
    assert!(page.contains("The command context."));
    assert!(page.contains("The command."));
    assert!(page.contains("## Returns\n\n`Awaitable<void>`"));
    assert!(!page.contains("`unknown`"));
}

#[test]
fn generate_docs_markdown_renders_index_signature_members() {
    let docs = vec![JsDocsMarkdownModule {
        description: None,
        file: "default".to_string(),
        source_path: None,
        examples: None,
        tags: None,
        entries: vec![
            JsDocsMarkdownEntry {
                name: "ArgSchema".to_string(),
                kind: "interface".to_string(),
                description: "Value type.".to_string(),
                params: None,
                returns: None,
                examples: None,
                tags: None,
                private: false,
                file: "/repo/src/args.ts".to_string(),
                line: 1,
                end_line: 1,
                signature: Some("export interface ArgSchema".to_string()),
                extends: None,
                implements: None,
                has_body: None,
                members: None,
                type_parameters: None,
            },
            JsDocsMarkdownEntry {
                name: "Args".to_string(),
                kind: "interface".to_string(),
                description: "Arguments.".to_string(),
                params: None,
                returns: None,
                examples: None,
                tags: None,
                private: false,
                file: "/repo/src/args.ts".to_string(),
                line: 1,
                end_line: 5,
                signature: Some("export interface Args".to_string()),
                extends: None,
                implements: None,
                has_body: None,
                members: Some(vec![JsDocMember {
                    name: "[option: string]".to_string(),
                    kind: "indexSignature".to_string(),
                    description: "Argument schema by option name.".to_string(),
                    signature: Some("readonly [option: string]: ArgSchema".to_string()),
                    r#type: Some("ArgSchema".to_string()),
                    r#default: None,
                    params: Some(vec![JsDocParam {
                        name: "option".to_string(),
                        r#type: "string".to_string(),
                        description: String::new(),
                        optional: None,
                        r#default: None,
                    }]),
                    type_parameters: None,
                    returns: None,
                    members: None,
                    optional: Some(false),
                    readonly: Some(true),
                    r#static: Some(false),
                    private: Some(false),
                    tags: None,
                    implementation_of: None,
                    line: 4,
                    end_line: 4,
                }]),
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
    let page = markdown.get("default/interfaces/Args.md").unwrap();

    assert!(page.contains("## Indexable\n\n"));
    assert!(page.contains("```ts\nreadonly [option: string]: ArgSchema\n```"));
    assert!(page.contains("Argument schema by option name."));
}

#[test]
fn generate_docs_markdown_renders_module_description_in_typedoc_index() {
    let docs = vec![JsDocsMarkdownModule {
        description: Some("The entry for gunshi context.".to_string()),
        file: "context".to_string(),
        source_path: None,
        examples: Some(vec!["```ts\ncreateCommandContext()\n```".to_string()]),
        tags: None,
        entries: vec![JsDocsMarkdownEntry {
            name: "createCommandContext".to_string(),
            kind: "function".to_string(),
            description: "Creates a command context.".to_string(),
            params: None,
            returns: None,
            examples: None,
            tags: None,
            private: false,
            file: "/repo/src/context.ts".to_string(),
            line: 1,
            end_line: 10,
            signature: Some("export function createCommandContext(): void".to_string()),
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
            github_url: None,
            link_style: Some("markdown".to_string()),
            base_path: None,
            path_strategy: Some("typedoc".to_string()),
            render_style: Some("markdown".to_string()),
            ..Default::default()
        }),
    );

    let index = markdown.get("index.md").unwrap();
    assert!(index.contains("The entry for gunshi context."));
    assert!(!index.contains("Creates a command context."));
    let module_index = markdown.get("context/index.md").unwrap();
    assert!(module_index.contains("The entry for gunshi context."));
    assert!(module_index.contains("## Example\n\n```ts\ncreateCommandContext()\n```"));
}
