use super::*;

#[test]
fn markdown_property_display_format_controls_property_groups() {
    let mut entry = test_entry("Command", "interface", "src/types.ts", "Command options.");
    entry.members = vec![ApiDocMember {
        name: "name".to_string(),
        kind: "property".to_string(),
        description: "Command name.".to_string(),
        type_annotation: Some("string".to_string()),
        readonly: true,
        line: 2,
        end_line: 2,
        ..ApiDocMember::default()
    }];
    let docs = vec![ApiDocModule {
        file: "mod".to_string(),
        entries: vec![entry],
        ..ApiDocModule::default()
    }];

    let list_markdown = generate_markdown(
        &docs,
        &MarkdownDocsOptions {
            path_strategy: MarkdownPathStrategy::TypeDoc,
            render_style: MarkdownRenderStyle::Markdown,
            ..MarkdownDocsOptions::default()
        },
    );
    let list_page = list_markdown.get("mod/interfaces/Command.md").unwrap();
    assert!(list_page.contains("- `name` _(readonly)_ `property` `string` - Command name."));
    assert!(!list_page.contains("| Name | Kind | Type | Description |"));

    let table_markdown = generate_markdown(
        &docs,
        &MarkdownDocsOptions {
            path_strategy: MarkdownPathStrategy::TypeDoc,
            render_style: MarkdownRenderStyle::Markdown,
            interface_properties_format: MarkdownDisplayFormat::Table,
            ..MarkdownDocsOptions::default()
        },
    );
    let table_page = table_markdown.get("mod/interfaces/Command.md").unwrap();
    // The `Properties` heading states the kind, so the table omits the Kind column.
    assert!(table_page.contains("| Name | Type | Description |"));
    assert!(!table_page.contains("| Name | Kind | Type | Description |"));
    assert!(table_page.contains("| `name` _(readonly)_ | `string` | Command name. |"));
}

#[test]
fn markdown_renders_member_default_values() {
    let mut entry = test_entry("Command", "interface", "src/types.ts", "Command options.");
    entry.members = vec![ApiDocMember {
        name: "timeout".to_string(),
        kind: "property".to_string(),
        description: "Request timeout.".to_string(),
        type_annotation: Some("number".to_string()),
        default_value: Some("5000".to_string()),
        optional: true,
        line: 2,
        end_line: 2,
        ..ApiDocMember::default()
    }];
    let docs = vec![ApiDocModule {
        file: "mod".to_string(),
        entries: vec![entry],
        ..ApiDocModule::default()
    }];

    let html = generate_markdown(
        &docs,
        &MarkdownDocsOptions {
            path_strategy: MarkdownPathStrategy::TypeDoc,
            interface_properties_format: MarkdownDisplayFormat::Table,
            ..MarkdownDocsOptions::default()
        },
    );
    let html_page = html.get("mod/interfaces/Command.md").unwrap();
    assert!(html_page.contains("ox-api-entry__member-default"));
    assert!(
        html_page.contains("<span>Default</span> <code class=\"language-typescript\">5000</code>")
    );

    let markdown = generate_markdown(
        &docs,
        &MarkdownDocsOptions {
            path_strategy: MarkdownPathStrategy::TypeDoc,
            render_style: MarkdownRenderStyle::Markdown,
            interface_properties_format: MarkdownDisplayFormat::Table,
            ..MarkdownDocsOptions::default()
        },
    );
    let markdown_page = markdown.get("mod/interfaces/Command.md").unwrap();
    assert!(markdown_page
        .contains("| `timeout` _(optional)_ | `number` | Request timeout. **Default:** `5000` |"));
}

#[test]
fn markdown_member_parameters_follow_parameters_format() {
    let mut entry = test_entry("Command", "interface", "src/types.ts", "Command options.");
    entry.members = vec![ApiDocMember {
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
        line: 2,
        end_line: 2,
        ..ApiDocMember::default()
    }];
    let docs = vec![ApiDocModule {
        file: "mod".to_string(),
        entries: vec![entry],
        ..ApiDocModule::default()
    }];

    let list_markdown = generate_markdown(
        &docs,
        &MarkdownDocsOptions {
            path_strategy: MarkdownPathStrategy::TypeDoc,
            render_style: MarkdownRenderStyle::Markdown,
            ..MarkdownDocsOptions::default()
        },
    );
    let list_page = list_markdown.get("mod/interfaces/Command.md").unwrap();
    assert!(list_page.contains("### run()"));
    assert!(list_page.contains("#### Parameters"));
    assert!(list_page.contains("- `ctx` (`Context`) - Runtime context."));

    let table_markdown = generate_markdown(
        &docs,
        &MarkdownDocsOptions {
            path_strategy: MarkdownPathStrategy::TypeDoc,
            render_style: MarkdownRenderStyle::Markdown,
            parameters_format: MarkdownDisplayFormat::Table,
            ..MarkdownDocsOptions::default()
        },
    );
    let table_page = table_markdown.get("mod/interfaces/Command.md").unwrap();
    assert!(table_page.contains("### run()"));
    assert!(table_page.contains("#### Parameters"));
    assert!(table_page.contains("| Name | Type | Description |"));
    assert!(table_page.contains("| `ctx` | `Context` | Runtime context. |"));
}
