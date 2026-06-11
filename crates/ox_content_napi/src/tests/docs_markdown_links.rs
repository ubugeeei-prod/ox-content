use super::*;

#[test]
fn generate_docs_markdown_accepts_clean_link_options() {
    let docs = vec![JsDocsMarkdownModule {
        description: None,
        file: "/repo/src/context.ts".to_string(),
        source_path: None,
        examples: None,
        tags: None,
        entries: vec![JsDocsMarkdownEntry {
            name: "CommandContext".to_string(),
            kind: "interface".to_string(),
            description: "Runtime context.".to_string(),
            params: None,
            returns: None,
            throws: None,
            examples: None,
            tags: None,
            private: false,
            file: "/repo/src/context.ts".to_string(),
            line: 1,
            end_line: 1,
            signature: Some("export interface CommandContext".to_string()),
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
            link_style: Some("clean".to_string()),
            base_path: Some("/api-ox".to_string()),
            path_strategy: None,
            render_style: None,
            ..Default::default()
        }),
    );
    let index = markdown.get("index.md").unwrap();

    assert!(index.contains("href=\"/api-ox/context\""));
    assert!(index.contains("href=\"/api-ox/context#commandcontext\""));
}

#[test]
fn generate_docs_markdown_render_style_markdown_omits_html() {
    let docs = vec![JsDocsMarkdownModule {
        description: None,
        file: "/repo/src/context.ts".to_string(),
        source_path: None,
        examples: None,
        tags: None,
        entries: vec![JsDocsMarkdownEntry {
            name: "CommandContext".to_string(),
            kind: "interface".to_string(),
            description: "Runtime context.".to_string(),
            params: None,
            returns: None,
            throws: None,
            examples: None,
            tags: None,
            private: false,
            file: "/repo/src/context.ts".to_string(),
            line: 1,
            end_line: 1,
            signature: Some("export interface CommandContext".to_string()),
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
            link_style: None,
            base_path: None,
            path_strategy: None,
            render_style: Some("markdown".to_string()),
            ..Default::default()
        }),
    );
    let page = markdown.get("context.md").unwrap();

    assert!(!page.contains("<details"));
    assert!(!page.contains("class=\"ox-api"));
    assert!(page.contains("### CommandContext"));
    assert!(page.contains("```ts"));
}

#[test]
fn generate_docs_markdown_accepts_display_format_options() {
    let docs = vec![JsDocsMarkdownModule {
        description: None,
        file: "default".to_string(),
        source_path: None,
        examples: None,
        tags: None,
        entries: vec![JsDocsMarkdownEntry {
            name: "make".to_string(),
            kind: "function".to_string(),
            description: "Make a thing.".to_string(),
            params: Some(vec![JsDocParam {
                name: "value".to_string(),
                r#type: "string".to_string(),
                description: "Input value.".to_string(),
                optional: Some(false),
                r#default: None,
            }]),
            returns: None,
            throws: None,
            examples: None,
            tags: None,
            private: false,
            file: "/repo/src/make.ts".to_string(),
            line: 1,
            end_line: 1,
            signature: Some("export function make(value: string): void".to_string()),
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
            render_style: Some("markdown".to_string()),
            parameters_format: Some("table".to_string()),
            ..Default::default()
        }),
    );
    let page = markdown.get("default.md").unwrap();

    assert!(page.contains("| Name | Type | Description |"));
    assert!(page.contains("| `value` | `string` | Input value. |"));
}

#[test]
fn generate_docs_markdown_type_declaration_format_table_renders_html() {
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
                        description: "Resolved values.".to_string(),
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
            group_by: Some("file".to_string()),
            path_strategy: Some("typedoc".to_string()),
            render_style: Some("html".to_string()),
            type_declaration_format: Some("table".to_string()),
            ..Default::default()
        }),
    );
    let page = markdown.get("default/functions/resolveArgs.md").unwrap();

    assert!(page.contains("ox-api-entry__type-declaration-table"));
    assert!(page.contains("<td><code>values</code></td>"));
    assert!(page.contains("Resolved values."));
    assert!(!page.contains("ox-api-entry__return-members"));
}
