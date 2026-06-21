use super::*;

#[test]
fn markdown_member_type_parameters_render_before_parameters() {
    let mut entry = test_entry("PluginContext", "interface", "src/context.ts", "Plugin context.");
    entry.members = vec![ApiDocMember {
            name: "decorateCommand".to_string(),
            kind: "method".to_string(),
            description: "Decorate the command.".to_string(),
            signature: Some(
                "decorateCommand<L extends Record<string, unknown> = DefaultExtensions>(decorator: (value: L) => void): void"
                    .to_string(),
            ),
            params: vec![ApiParamDoc {
                name: "decorator".to_string(),
                type_annotation: "(value: L) => void".to_string(),
                description: "Decorator function.".to_string(),
                ..ApiParamDoc::default()
            }],
            type_parameters: vec![
                ApiTypeParamDoc {
                    name: "L".to_string(),
                    constraint: Some("Record<string, unknown>".to_string()),
                    default: Some("DefaultExtensions".to_string()),
                    description: "Extension context.".to_string(),
                },
                ApiTypeParamDoc {
                    name: "Fallback".to_string(),
                    ..ApiTypeParamDoc::default()
                },
            ],
            line: 2,
            end_line: 5,
            ..ApiDocMember::default()
        }];
    let docs = vec![ApiDocModule {
        file: "mod".to_string(),
        entries: vec![entry],
        ..ApiDocModule::default()
    }];

    let pure = generate_markdown(
        &docs,
        &MarkdownDocsOptions {
            path_strategy: MarkdownPathStrategy::TypeDoc,
            render_style: MarkdownRenderStyle::Markdown,
            parameters_format: MarkdownDisplayFormat::Table,
            ..MarkdownDocsOptions::default()
        },
    );
    assert_markdown_map_snapshot(
        "markdown_member_type_parameters_render_before_parameters__pure",
        &pure,
    );
    let page = pure.get("mod/interfaces/PluginContext.md").unwrap();
    let type_parameters = page.find("#### Type Parameters").unwrap();
    let parameters = page.find("#### Parameters").unwrap();
    assert!(type_parameters < parameters);

    let html = generate_markdown(
        &docs,
        &MarkdownDocsOptions {
            path_strategy: MarkdownPathStrategy::TypeDoc,
            render_style: MarkdownRenderStyle::Html,
            parameters_format: MarkdownDisplayFormat::Table,
            ..MarkdownDocsOptions::default()
        },
    );
    assert_markdown_map_snapshot(
        "markdown_member_type_parameters_render_before_parameters__html",
        &html,
    );
}

#[test]
fn typedoc_markdown_renders_class_callable_member_details() {
    let mut adapter =
        test_entry("TranslationAdapter", "interface", "/repo/src/i18n.ts", "Runtime adapter.");
    adapter.members = vec![ApiDocMember {
        name: "getResource".to_string(),
        kind: "method".to_string(),
        description: "Gets a locale resource.".to_string(),
        signature: Some(
            "getResource(locale: string): Record<string, string> | undefined".to_string(),
        ),
        params: vec![ApiParamDoc {
            name: "locale".to_string(),
            type_annotation: "string".to_string(),
            description: "Locale name.".to_string(),
            ..ApiParamDoc::default()
        }],
        returns: Some(ApiReturnDoc {
            type_annotation: "Record<string, string> | undefined".to_string(),
            description: "The locale resource.".to_string(),
            ..ApiReturnDoc::default()
        }),
        line: 4,
        end_line: 4,
        ..ApiDocMember::default()
    }];

    let mut implementation =
        test_entry("DefaultTranslation", "class", "/repo/src/i18n.ts", "Default runtime adapter.");
    implementation.signature =
        Some("class DefaultTranslation implements TranslationAdapter".to_string());
    implementation.implements = vec!["TranslationAdapter".to_string()];
    implementation.members = vec![
        ApiDocMember {
            name: "constructor".to_string(),
            kind: "constructor".to_string(),
            description: "Creates the adapter.".to_string(),
            signature: Some("constructor(options: TranslationAdapterFactoryOptions)".to_string()),
            params: vec![ApiParamDoc {
                name: "options".to_string(),
                type_annotation: "TranslationAdapterFactoryOptions".to_string(),
                description: "Adapter options.".to_string(),
                ..ApiParamDoc::default()
            }],
            line: 10,
            end_line: 10,
            ..ApiDocMember::default()
        },
        ApiDocMember {
            name: "getResource".to_string(),
            kind: "method".to_string(),
            description: "Gets a locale resource.".to_string(),
            signature: Some(
                "getResource(locale: string): Record<string, string> | undefined".to_string(),
            ),
            params: vec![ApiParamDoc {
                name: "locale".to_string(),
                type_annotation: "string".to_string(),
                description: "Locale name.".to_string(),
                ..ApiParamDoc::default()
            }],
            returns: Some(ApiReturnDoc {
                type_annotation: "Record<string, string> | undefined".to_string(),
                description: "The locale resource.".to_string(),
                ..ApiReturnDoc::default()
            }),
            line: 14,
            end_line: 16,
            ..ApiDocMember::default()
        },
    ];
    let docs = vec![ApiDocModule {
        file: "default".to_string(),
        entries: vec![adapter, implementation],
        ..ApiDocModule::default()
    }];

    let markdown = generate_markdown(
        &docs,
        &MarkdownDocsOptions {
            parameters_format: MarkdownDisplayFormat::Table,
            ..markdown_typedoc_options()
        },
    );
    assert_markdown_map_snapshot(
        "typedoc_markdown_renders_class_callable_member_details__markdown",
        &markdown,
    );
    assert_markdown_map_snapshot(
        "typedoc_markdown_renders_class_callable_member_details__markdown",
        &markdown,
    );
}

#[test]
fn typedoc_markdown_renders_function_valued_property_details() {
    let mut schema =
        test_entry("ArgSchema", "interface", "/repo/src/schema.ts", "Argument schema.");
    schema.members = vec![function_valued_parse_member()];
    let docs = vec![ApiDocModule {
        file: "default".to_string(),
        entries: vec![schema],
        ..ApiDocModule::default()
    }];

    let markdown = generate_markdown(
        &docs,
        &MarkdownDocsOptions {
            interface_properties_format: MarkdownDisplayFormat::Table,
            parameters_format: MarkdownDisplayFormat::Table,
            ..markdown_typedoc_options()
        },
    );
    assert_markdown_map_snapshot(
        "typedoc_markdown_renders_function_valued_property_details__markdown",
        &markdown,
    );
    assert_markdown_map_snapshot(
        "typedoc_markdown_renders_function_valued_property_details__markdown",
        &markdown,
    );
}
