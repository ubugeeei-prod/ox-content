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
    assert_markdown_map_snapshot(
        "markdown_property_display_format_controls_property_groups__list_markdown",
        &list_markdown,
    );

    let table_markdown = generate_markdown(
        &docs,
        &MarkdownDocsOptions {
            path_strategy: MarkdownPathStrategy::TypeDoc,
            render_style: MarkdownRenderStyle::Markdown,
            interface_properties_format: MarkdownDisplayFormat::Table,
            ..MarkdownDocsOptions::default()
        },
    );
    assert_markdown_map_snapshot(
        "markdown_property_display_format_controls_property_groups__table_markdown",
        &table_markdown,
    );
    // The `Properties` heading states the kind, so the table omits the Kind column.
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
    assert_markdown_map_snapshot("markdown_renders_member_default_values__html", &html);

    let markdown = generate_markdown(
        &docs,
        &MarkdownDocsOptions {
            path_strategy: MarkdownPathStrategy::TypeDoc,
            render_style: MarkdownRenderStyle::Markdown,
            interface_properties_format: MarkdownDisplayFormat::Table,
            ..MarkdownDocsOptions::default()
        },
    );
    assert_markdown_map_snapshot("markdown_renders_member_default_values__markdown", &markdown);
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
    assert_markdown_map_snapshot(
        "markdown_member_parameters_follow_parameters_format__list_markdown",
        &list_markdown,
    );

    let table_markdown = generate_markdown(
        &docs,
        &MarkdownDocsOptions {
            path_strategy: MarkdownPathStrategy::TypeDoc,
            render_style: MarkdownRenderStyle::Markdown,
            parameters_format: MarkdownDisplayFormat::Table,
            ..MarkdownDocsOptions::default()
        },
    );
    assert_markdown_map_snapshot(
        "markdown_member_parameters_follow_parameters_format__table_markdown",
        &table_markdown,
    );
}
