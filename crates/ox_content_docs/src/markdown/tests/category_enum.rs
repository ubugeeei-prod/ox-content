use super::*;

#[test]
fn category_group_ignores_typedoc_path_strategy() {
    let docs = link_test_docs();

    let category_flat = generate_markdown(
        &docs,
        &MarkdownDocsOptions { group_by: "category".to_string(), ..MarkdownDocsOptions::default() },
    );
    let category_typedoc = generate_markdown(
        &docs,
        &MarkdownDocsOptions {
            group_by: "category".to_string(),
            path_strategy: MarkdownPathStrategy::TypeDoc,
            ..MarkdownDocsOptions::default()
        },
    );

    let mut flat_keys = category_flat.keys().cloned().collect::<Vec<_>>();
    let mut typedoc_keys = category_typedoc.keys().cloned().collect::<Vec<_>>();
    flat_keys.sort();
    typedoc_keys.sort();
    assert_eq!(flat_keys, typedoc_keys);

    assert!(category_typedoc.contains_key("functions.md"));
    assert!(category_typedoc.contains_key("interfaces.md"));
    assert!(!category_typedoc.keys().any(|key| key.contains('/')));
}

#[test]
fn typedoc_path_strategy_emits_enumerations_directory() {
    let docs = vec![ApiDocModule {
        description: String::new(),
        file: "default".to_string(),
        source_path: String::new(),
        examples: vec![],
        tags: vec![],
        entries: vec![
            ApiDocEntry {
                name: "Mode".to_string(),
                kind: "enum".to_string(),
                description: "Execution mode.".to_string(),
                params: vec![],
                returns: None,
                examples: vec![],
                tags: vec![],
                private: false,
                file: "/repo/src/mode.ts".to_string(),
                line: 1,
                end_line: 5,
                signature: Some("export enum Mode".to_string()),
                extends: vec![],
                implements: vec![],
                has_body: false,
                members: vec![ApiDocMember {
                    name: "Strict".to_string(),
                    kind: "enumMember".to_string(),
                    description: "Strict mode.".to_string(),
                    signature: None,
                    type_annotation: Some("\"strict\"".to_string()),
                    default_value: None,
                    params: vec![],
                    type_parameters: vec![],
                    returns: None,
                    members: vec![],
                    optional: false,
                    readonly: false,
                    r#static: false,
                    private: false,
                    tags: vec![],
                    implementation_of: vec![],
                    line: 2,
                    end_line: 2,
                }],
                type_parameters: vec![],
            },
            ApiDocEntry {
                name: "run".to_string(),
                kind: "function".to_string(),
                description: "Runs in {@link Mode} or {@linkcode Mode.Strict}.".to_string(),
                params: vec![],
                returns: None,
                examples: vec![],
                tags: vec![],
                private: false,
                file: "/repo/src/run.ts".to_string(),
                line: 1,
                end_line: 5,
                signature: Some("export function run(mode: Mode): void".to_string()),
                extends: vec![],
                implements: vec![],
                has_body: false,
                members: vec![],
                type_parameters: vec![],
            },
        ],
    }];

    let markdown = generate_markdown(
        &docs,
        &MarkdownDocsOptions {
            path_strategy: MarkdownPathStrategy::TypeDoc,
            ..MarkdownDocsOptions::default()
        },
    );
    let mode_page = markdown.get("default/enumerations/Mode.md").unwrap();
    let run_page = markdown.get("default/functions/run.md").unwrap();
    let module_index = markdown.get("default/index.md").unwrap();

    assert!(module_index.contains("## Enumerations"));
    assert!(module_index.contains("| Enumeration | Description |"));
    assert!(module_index.contains("| [Mode](./enumerations/Mode.md) |"));
    assert!(mode_page.contains("<tr id=\"enumeration-member-strict\">"));
    assert!(run_page.contains("<a href=\"../enumerations/Mode.md\">Mode</a>"));
    assert!(run_page.contains(
            "<a href=\"../enumerations/Mode.md#enumeration-member-strict\"><code>Mode.Strict</code></a>"
        ));
}

#[test]
fn renders_interface_members_table() {
    let docs = vec![ApiDocModule {
        description: String::new(),
        file: "/repo/src/command.ts".to_string(),
        source_path: String::new(),
        examples: vec![],
        tags: vec![],
        entries: vec![ApiDocEntry {
            name: "Command".to_string(),
            kind: "interface".to_string(),
            description: "Runtime command.".to_string(),
            params: vec![],
            returns: None,
            examples: vec![],
            tags: vec![],
            private: false,
            file: "/repo/src/command.ts".to_string(),
            line: 1,
            end_line: 10,
            signature: Some("export interface Command".to_string()),
            extends: vec![],
            implements: vec![],
            has_body: false,
            members: vec![
                ApiDocMember {
                    name: "name".to_string(),
                    kind: "property".to_string(),
                    description: "Command name.".to_string(),
                    signature: None,
                    type_annotation: Some("string".to_string()),
                    default_value: None,
                    params: vec![],
                    type_parameters: vec![],
                    returns: None,
                    members: vec![],
                    optional: false,
                    readonly: true,
                    r#static: false,
                    private: false,
                    tags: vec![],
                    implementation_of: vec![],
                    line: 5,
                    end_line: 5,
                },
                ApiDocMember {
                    name: "run".to_string(),
                    kind: "method".to_string(),
                    description: "Runs the command.".to_string(),
                    signature: Some("run(ctx: Context): Promise<void>".to_string()),
                    type_annotation: None,
                    default_value: None,
                    params: vec![ApiParamDoc {
                        name: "ctx".to_string(),
                        type_annotation: "Context".to_string(),
                        description: "Runtime context.".to_string(),
                        optional: false,
                        default_value: None,
                    }],
                    type_parameters: vec![],
                    returns: Some(ApiReturnDoc {
                        type_annotation: "Promise".to_string(),
                        description: "Run result.".to_string(),
                        members: Vec::new(),
                    }),
                    members: vec![],
                    optional: false,
                    readonly: false,
                    r#static: false,
                    private: false,
                    tags: vec![],
                    implementation_of: vec![],
                    line: 7,
                    end_line: 7,
                },
            ],
            type_parameters: vec![],
        }],
    }];

    let markdown = generate_markdown(&docs, &MarkdownDocsOptions::default());
    let page = markdown.get("command.md").unwrap();

    assert!(page.contains("<h4>Members</h4>"));
    assert!(page.contains("<h5>Properties</h5>"));
    assert!(page.contains("<code>name</code>"));
    assert!(page.contains("readonly"));
    assert!(page.contains("Command name."));
    assert!(page.contains("<h5>Methods</h5>"));
    assert!(page.contains("run(ctx: Context): Promise&lt;void&gt;"));
    assert!(page.contains("Runtime context."));
}
