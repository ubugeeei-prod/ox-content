use super::*;

#[test]
fn generate_docs_markdown_accepts_clean_link_options() {
    let docs = vec![JsDocsMarkdownModule {
        file: "/repo/src/context.ts".to_string(),
        entries: vec![JsDocsMarkdownEntry {
            name: "CommandContext".to_string(),
            kind: "interface".to_string(),
            description: "Runtime context.".to_string(),
            file: "/repo/src/context.ts".to_string(),
            signature: Some("export interface CommandContext".to_string()),
            ..Default::default()
        }],
        ..Default::default()
    }];
    let markdown = generate_docs_markdown(
        docs,
        Some(JsDocsMarkdownOptions {
            group_by: Some("file".to_string()),
            link_style: Some("clean".to_string()),
            base_path: Some("/api-ox".to_string()),
            ..Default::default()
        }),
    );
    assert_string_map_snapshot("generate_docs_markdown_accepts_clean_link_options", &markdown);
}

#[test]
fn generate_docs_markdown_render_style_markdown_omits_html() {
    let docs = vec![JsDocsMarkdownModule {
        file: "/repo/src/context.ts".to_string(),
        entries: vec![JsDocsMarkdownEntry {
            name: "CommandContext".to_string(),
            kind: "interface".to_string(),
            description: "Runtime context.".to_string(),
            file: "/repo/src/context.ts".to_string(),
            signature: Some("export interface CommandContext".to_string()),
            ..Default::default()
        }],
        ..Default::default()
    }];
    let markdown = generate_docs_markdown(
        docs,
        Some(JsDocsMarkdownOptions {
            group_by: Some("file".to_string()),
            render_style: Some("markdown".to_string()),
            ..Default::default()
        }),
    );
    assert_string_map_snapshot(
        "generate_docs_markdown_render_style_markdown_omits_html",
        &markdown,
    );
}

#[test]
fn generate_docs_markdown_accepts_display_format_options() {
    let docs = vec![JsDocsMarkdownModule {
        file: "default".to_string(),
        entries: vec![JsDocsMarkdownEntry {
            name: "make".to_string(),
            kind: "function".to_string(),
            description: "Make a thing.".to_string(),
            params: Some(vec![JsDocParam {
                name: "value".to_string(),
                r#type: "string".to_string(),
                description: "Input value.".to_string(),
                ..Default::default()
            }]),
            file: "/repo/src/make.ts".to_string(),
            signature: Some("export function make(value: string): void".to_string()),
            ..Default::default()
        }],
        ..Default::default()
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
    assert_string_map_snapshot("generate_docs_markdown_accepts_display_format_options", &markdown);
}

#[test]
fn generate_docs_markdown_type_declaration_format_table_renders_html() {
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
                        description: "Resolved values.".to_string(),
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
            group_by: Some("file".to_string()),
            path_strategy: Some("typedoc".to_string()),
            render_style: Some("html".to_string()),
            type_declaration_format: Some("table".to_string()),
            ..Default::default()
        }),
    );
    assert_string_map_snapshot(
        "generate_docs_markdown_type_declaration_format_table_renders_html",
        &markdown,
    );
}
