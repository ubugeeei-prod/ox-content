use super::*;

#[test]
fn html_display_format_options_switch_explicit_sections() {
    let mut make = test_entry("make", "function", "src/make.ts", "Make a thing.");
    make.params = vec![ApiParamDoc {
        name: "value".to_string(),
        type_annotation: "string".to_string(),
        description: "Input value.".to_string(),
        ..ApiParamDoc::default()
    }];
    make.type_parameters = vec![ApiTypeParamDoc {
        name: "T".to_string(),
        description: "Value type.".to_string(),
        ..ApiTypeParamDoc::default()
    }];

    let mut command = test_entry("Command", "interface", "src/types.ts", "Command options.");
    command.members = vec![
        ApiDocMember {
            name: "name".to_string(),
            kind: "property".to_string(),
            description: "Command name.".to_string(),
            type_annotation: Some("string".to_string()),
            readonly: true,
            line: 2,
            end_line: 2,
            ..ApiDocMember::default()
        },
        ApiDocMember {
            name: "run".to_string(),
            kind: "method".to_string(),
            description: "Runs the command.".to_string(),
            signature: Some("run(ctx: Context): Promise<void>".to_string()),
            params: vec![ApiParamDoc {
                name: "ctx".to_string(),
                type_annotation: "Context".to_string(),
                description: "Runtime context.".to_string(),
                ..ApiParamDoc::default()
            }],
            line: 3,
            end_line: 3,
            ..ApiDocMember::default()
        },
    ];

    let docs = vec![ApiDocModule {
        file: "mod".to_string(),
        entries: vec![make, command],
        ..ApiDocModule::default()
    }];

    let table_markdown = generate_markdown(
        &docs,
        &MarkdownDocsOptions {
            index_format: MarkdownDisplayFormat::Table,
            parameters_format: MarkdownDisplayFormat::Table,
            interface_properties_format: MarkdownDisplayFormat::List,
            ..MarkdownDocsOptions::default()
        },
    );
    let index = table_markdown.get("index.md").unwrap();
    assert!(index.contains("<table class=\"ox-api-modules-table\">"));
    assert!(index.contains("<th>Module</th><th>Symbols</th><th>Description</th>"));

    let page = table_markdown.get("mod.md").unwrap();
    assert!(page.contains("<table class=\"ox-api-entry__params-table\">"));
    assert!(page.contains("<table class=\"ox-api-entry__member-params-table\">"));
    assert!(!page.contains("<ul class=\"ox-api-entry__params\">"));
    assert!(page.contains("<ul class=\"ox-api-entry__members-list\">"));
    assert!(page.contains("<li id=\"command-name\" class=\"ox-api-entry__member\">"));

    let list_markdown = generate_markdown(
        &docs,
        &MarkdownDocsOptions {
            index_format: MarkdownDisplayFormat::List,
            parameters_format: MarkdownDisplayFormat::List,
            ..MarkdownDocsOptions::default()
        },
    );
    let index = list_markdown.get("index.md").unwrap();
    assert!(index.contains("<ul class=\"ox-api-modules-list\">"));
    assert!(!index.contains("<details class=\"ox-api-module\">"));

    let page = list_markdown.get("mod.md").unwrap();
    assert!(page.contains("<ul class=\"ox-api-entry__type-parameters\">"));
    assert!(page.contains("<ul class=\"ox-api-entry__member-params\">"));
    assert!(!page.contains("<table class=\"ox-api-entry__type-parameters-table\">"));
}

#[test]
fn typedoc_module_index_renders_member_tables() {
    let docs = vec![ApiDocModule {
        file: "default".to_string(),
        entries: vec![
            test_entry("cli", "function", "/repo/src/cli.ts", "Run the command."),
            test_entry("CliOptions", "interface", "/repo/src/types.ts", "CLI options."),
        ],
        ..ApiDocModule::default()
    }];

    let out = generate_markdown(
        &docs,
        &MarkdownDocsOptions {
            path_strategy: MarkdownPathStrategy::TypeDoc,
            ..MarkdownDocsOptions::default()
        },
    );
    let index = out.get("default/index.md").unwrap();

    assert!(index.contains("## Functions"));
    assert!(index.contains("| Function | Description |\n| ------ | ------ |"));
    assert!(index.contains("| [cli](./functions/cli.md) | Run the command. |"));
    assert!(index.contains("## Interfaces"));
    assert!(index.contains("| Interface | Description |"));
    assert!(index.contains("| [CliOptions](./interfaces/CliOptions.md) | CLI options. |"));
    // No bullet list, no inlined kind label or signature.
    assert!(!index.contains("- [`cli`]"));
    assert!(!index.contains("`function`"));
    assert!(!index.contains("export function cli"));
}
