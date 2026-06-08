use super::*;

#[test]
fn type_parameters_render_as_a_section() {
    let mut entry = test_entry("make", "function", "src/make.ts", "Make a thing.");
    entry.type_parameters = vec![
        ApiTypeParamDoc {
            name: "G".to_string(),
            constraint: Some("Base".to_string()),
            default: Some("Default".to_string()),
            description: String::new(),
        },
        ApiTypeParamDoc {
            name: "T".to_string(),
            constraint: None,
            default: None,
            description: "The value type.".to_string(),
        },
    ];
    let docs = vec![ApiDocModule {
        description: String::new(),
        file: "mod".to_string(),
        source_path: String::new(),
        examples: vec![],
        tags: vec![],
        entries: vec![entry],
    }];

    let markdown = generate_markdown(
        &docs,
        &MarkdownDocsOptions {
            path_strategy: MarkdownPathStrategy::TypeDoc,
            render_style: MarkdownRenderStyle::Markdown,
            ..MarkdownDocsOptions::default()
        },
    );
    let page = markdown.get("mod/functions/make.md").unwrap();

    assert!(page.contains("## Type Parameters"));
    assert!(!page.contains("**Type Parameters**"));
    assert!(page.contains("`G` *extends* `Base` = `Default`"));
    assert!(page.contains("- `T` - The value type."));
}

#[test]
fn markdown_display_format_options_render_tables() {
    let mut entry = test_entry("make", "function", "src/make.ts", "Make a thing.");
    entry.params = vec![ApiParamDoc {
        name: "value".to_string(),
        type_annotation: "string".to_string(),
        description: "Input value.".to_string(),
        optional: false,
        default_value: None,
    }];
    entry.type_parameters = vec![ApiTypeParamDoc {
        name: "T".to_string(),
        constraint: None,
        default: None,
        description: "Value type.".to_string(),
    }];
    let docs = vec![ApiDocModule {
        description: String::new(),
        file: "mod".to_string(),
        source_path: String::new(),
        examples: vec![],
        tags: vec![],
        entries: vec![entry],
    }];

    let markdown = generate_markdown(
        &docs,
        &MarkdownDocsOptions {
            path_strategy: MarkdownPathStrategy::TypeDoc,
            render_style: MarkdownRenderStyle::Markdown,
            index_format: MarkdownDisplayFormat::Table,
            parameters_format: MarkdownDisplayFormat::Table,
            ..MarkdownDocsOptions::default()
        },
    );
    let index = markdown.get("mod/index.md").unwrap();
    let page = markdown.get("mod/functions/make.md").unwrap();

    assert!(index.contains("| Function | Description |"));
    assert!(page.contains("| Name | Type | Description |"));
    assert!(page.contains("| `value` | `string` | Input value. |"));
    assert!(page.contains("| `T` | Value type. |"));
}

#[test]
fn markdown_type_parameter_table_omits_description_column_when_all_empty() {
    let mut entry = test_entry("make", "function", "src/make.ts", "Make a thing.");
    entry.type_parameters = vec![
        ApiTypeParamDoc {
            name: "G".to_string(),
            constraint: Some("Base".to_string()),
            default: Some("Default".to_string()),
            description: String::new(),
        },
        ApiTypeParamDoc {
            name: "V".to_string(),
            constraint: None,
            default: None,
            description: "   ".to_string(),
        },
    ];
    let docs = vec![ApiDocModule {
        description: String::new(),
        file: "mod".to_string(),
        source_path: String::new(),
        examples: vec![],
        tags: vec![],
        entries: vec![entry],
    }];

    let markdown = generate_markdown(
        &docs,
        &MarkdownDocsOptions {
            path_strategy: MarkdownPathStrategy::TypeDoc,
            render_style: MarkdownRenderStyle::Markdown,
            parameters_format: MarkdownDisplayFormat::Table,
            ..MarkdownDocsOptions::default()
        },
    );
    let page = markdown.get("mod/functions/make.md").unwrap();

    assert!(page.contains("## Type Parameters"));
    assert!(page.contains("| Name |\n| --- |"));
    assert!(!page.contains("| Name | Description |"));
    assert!(page.contains("| `G` *extends* `Base` = `Default` |"));
    assert!(page.contains("| `V` |"));
    assert!(!page.contains("|  |"));
}

#[test]
fn markdown_type_parameter_table_renders_dash_for_missing_descriptions() {
    let mut entry = test_entry("make", "function", "src/make.ts", "Make a thing.");
    entry.type_parameters = vec![
        ApiTypeParamDoc {
            name: "T".to_string(),
            constraint: None,
            default: None,
            description: "Value type.".to_string(),
        },
        ApiTypeParamDoc {
            name: "G".to_string(),
            constraint: Some("Base".to_string()),
            default: Some("Default".to_string()),
            description: String::new(),
        },
    ];
    let docs = vec![ApiDocModule {
        description: String::new(),
        file: "mod".to_string(),
        source_path: String::new(),
        examples: vec![],
        tags: vec![],
        entries: vec![entry],
    }];

    let markdown = generate_markdown(
        &docs,
        &MarkdownDocsOptions {
            path_strategy: MarkdownPathStrategy::TypeDoc,
            render_style: MarkdownRenderStyle::Markdown,
            parameters_format: MarkdownDisplayFormat::Table,
            ..MarkdownDocsOptions::default()
        },
    );
    let page = markdown.get("mod/functions/make.md").unwrap();

    assert!(page.contains("| Name | Description |\n| --- | --- |"));
    assert!(page.contains("| `T` | Value type. |"));
    assert!(page.contains("| `G` *extends* `Base` = `Default` | - |"));
    assert!(!page.contains("| `G` *extends* `Base` = `Default` |  |"));
}

#[test]
fn html_type_parameter_tables_follow_empty_description_policy() {
    let mut all_empty = test_entry("make", "function", "src/make.ts", "Make a thing.");
    all_empty.type_parameters = vec![ApiTypeParamDoc {
        name: "G".to_string(),
        constraint: Some("Base".to_string()),
        default: Some("Default".to_string()),
        description: String::new(),
    }];

    let mut mixed = test_entry("build", "function", "src/build.ts", "Build a thing.");
    mixed.type_parameters = vec![
        ApiTypeParamDoc {
            name: "T".to_string(),
            constraint: None,
            default: None,
            description: "Value type.".to_string(),
        },
        ApiTypeParamDoc {
            name: "G".to_string(),
            constraint: Some("Base".to_string()),
            default: Some("Default".to_string()),
            description: String::new(),
        },
    ];

    let docs = vec![ApiDocModule {
        description: String::new(),
        file: "mod".to_string(),
        source_path: String::new(),
        examples: vec![],
        tags: vec![],
        entries: vec![all_empty, mixed],
    }];
    let html = generate_markdown(
        &docs,
        &MarkdownDocsOptions {
            path_strategy: MarkdownPathStrategy::TypeDoc,
            render_style: MarkdownRenderStyle::Html,
            parameters_format: MarkdownDisplayFormat::Table,
            ..MarkdownDocsOptions::default()
        },
    );
    let all_empty_page = html.get("mod/functions/make.md").unwrap();
    let mixed_page = html.get("mod/functions/build.md").unwrap();

    assert!(all_empty_page.contains("<thead><tr><th>Name</th></tr></thead>"));
    assert!(!all_empty_page.contains("<th>Description</th>"));
    assert!(!all_empty_page.contains("<td></td>"));

    assert!(mixed_page.contains("<thead><tr><th>Name</th><th>Description</th></tr></thead>"));
    assert!(mixed_page.contains("<td>Value type.</td>"));
    assert!(mixed_page.contains("<td>-</td>"));
}
