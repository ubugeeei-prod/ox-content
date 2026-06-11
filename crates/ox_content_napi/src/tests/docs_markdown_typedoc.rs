use super::*;

#[test]
fn generate_docs_markdown_property_members_format_table_renders_html() {
    let docs = vec![JsDocsMarkdownModule {
        file: "default".to_string(),
        entries: vec![JsDocsMarkdownEntry {
            name: "Options".to_string(),
            kind: "interface".to_string(),
            description: "Request options.".to_string(),
            file: "/repo/src/options.ts".to_string(),
            end_line: 8,
            signature: Some("export interface Options".to_string()),
            members: Some(vec![JsDocMember {
                name: "http".to_string(),
                kind: "property".to_string(),
                description: "HTTP options.".to_string(),
                r#type: Some("{ timeout?: number }".to_string()),
                members: Some(vec![JsDocMember {
                    name: "timeout".to_string(),
                    kind: "property".to_string(),
                    description: "Request timeout.".to_string(),
                    r#type: Some("number".to_string()),
                    optional: Some(true),
                    line: 4,
                    end_line: 4,
                    ..Default::default()
                }]),
                line: 3,
                end_line: 6,
                ..Default::default()
            }]),
            ..Default::default()
        }],
        ..Default::default()
    }];

    let markdown = generate_docs_markdown(
        docs,
        Some(JsDocsMarkdownOptions {
            group_by: Some("file".to_string()),
            path_strategy: Some("typedoc".to_string()),
            render_style: Some("html".to_string()),
            interface_properties_format: Some("table".to_string()),
            property_members_format: Some("table".to_string()),
            ..Default::default()
        }),
    );
    let page = markdown.get("default/interfaces/Options.md").unwrap();

    assert!(page.contains("ox-api-entry__property-members-table"));
    assert!(
        page.contains("<td><code>timeout</code><span class=\"ox-api-badge\">optional</span></td>")
    );
    assert!(page.contains("Request timeout."));
}

#[test]
fn generate_docs_markdown_resolves_jsdoc_inline_links() {
    let docs = vec![
        JsDocsMarkdownModule {
            file: "/repo/src/command.ts".to_string(),
            entries: vec![JsDocsMarkdownEntry {
                name: "Command".to_string(),
                kind: "interface".to_string(),
                description: "Runtime command.".to_string(),
                file: "/repo/src/command.ts".to_string(),
                end_line: 10,
                signature: Some("export interface Command".to_string()),
                members: Some(vec![JsDocMember {
                    name: "args".to_string(),
                    kind: "property".to_string(),
                    description: "All {@linkcode Command.args} names.".to_string(),
                    r#type: Some("Record<string, unknown>".to_string()),
                    line: 5,
                    end_line: 5,
                    ..Default::default()
                }]),
                ..Default::default()
            }],
            ..Default::default()
        },
        JsDocsMarkdownModule {
            file: "/repo/src/build.ts".to_string(),
            entries: vec![JsDocsMarkdownEntry {
                name: "buildCommand".to_string(),
                kind: "function".to_string(),
                description: "Builds {@linkcode Command | command} metadata.".to_string(),
                params: Some(vec![JsDocParam {
                    name: "entry".to_string(),
                    r#type: "Command".to_string(),
                    description: "A {@linkcode Command | entry command}".to_string(),
                    ..Default::default()
                }]),
                returns: Some(JsDocReturn {
                    r#type: "Command".to_string(),
                    description: "A {@link Command} result.".to_string(),
                    ..Default::default()
                }),
                tags: Some(vec![JsDocsMarkdownTag {
                    tag: "see".to_string(),
                    value: "{@link https://github.com/unjs/std-env | std-env}".to_string(),
                }]),
                file: "/repo/src/build.ts".to_string(),
                end_line: 20,
                signature: Some(
                    "export function buildCommand(entry: Command): Command".to_string(),
                ),
                ..Default::default()
            }],
            ..Default::default()
        },
    ];

    let markdown = generate_docs_markdown(docs, None);
    let build_page = markdown.get("build.md").unwrap();
    let command_page = markdown.get("command.md").unwrap();

    assert!(!build_page.contains("{@link"));
    assert!(!command_page.contains("{@link"));
    assert!(build_page.contains("<a href=\"./command.md#command\"><code>entry command</code></a>"));
    assert!(build_page.contains("<a href=\"./command.md#command\">Command</a>"));
    assert!(build_page.contains("<a href=\"https://github.com/unjs/std-env\">std-env</a>"));
    assert!(command_page.contains("<tr id=\"command-args\">"));
    assert!(command_page.contains("<a href=\"#command-args\"><code>Command.args</code></a>"));
}

#[test]
fn generate_docs_markdown_accepts_typedoc_path_strategy() {
    let docs = vec![JsDocsMarkdownModule {
        file: "default".to_string(),
        entries: vec![
            JsDocsMarkdownEntry {
                name: "Command".to_string(),
                kind: "interface".to_string(),
                description: "Runtime command.".to_string(),
                file: "/repo/src/types.ts".to_string(),
                end_line: 10,
                signature: Some("export interface Command".to_string()),
                ..Default::default()
            },
            JsDocsMarkdownEntry {
                name: "cli".to_string(),
                kind: "function".to_string(),
                description: "Runs {@link Command}.".to_string(),
                file: "/repo/src/cli.ts".to_string(),
                end_line: 10,
                signature: Some("export function cli(): void".to_string()),
                ..Default::default()
            },
        ],
        ..Default::default()
    }];

    let markdown = generate_docs_markdown(
        docs,
        Some(JsDocsMarkdownOptions {
            group_by: Some("file".to_string()),
            link_style: Some("clean".to_string()),
            base_path: Some("/api".to_string()),
            path_strategy: Some("typedoc".to_string()),
            ..Default::default()
        }),
    );
    let cli_page = markdown.get("default/functions/cli.md").unwrap();
    let root_index = markdown.get("index.md").unwrap();
    let module_index = markdown.get("default/index.md").unwrap();

    assert!(markdown.contains_key("default/index.md"));
    assert!(markdown.contains_key("default/interfaces/Command.md"));
    assert!(root_index.contains("[default](/api/default)"));
    assert!(!root_index.contains("[Default]"));
    assert!(module_index.starts_with("# default\n\n"));
    assert!(cli_page.contains("<a href=\"/api/default/interfaces/Command\">Command</a>"));
}
