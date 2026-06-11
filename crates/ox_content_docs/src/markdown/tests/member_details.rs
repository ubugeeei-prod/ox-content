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
    let page = pure.get("mod/interfaces/PluginContext.md").unwrap();
    let type_parameters = page.find("#### Type Parameters").unwrap();
    let parameters = page.find("#### Parameters").unwrap();
    assert!(type_parameters < parameters);
    assert!(page.contains("`L` *extends*"));
    assert!(page.contains("Extension context."));
    assert!(page.contains("| `Fallback` | - |"));
    assert!(!page.contains("| `Fallback` |  |"));

    let html = generate_markdown(
        &docs,
        &MarkdownDocsOptions {
            path_strategy: MarkdownPathStrategy::TypeDoc,
            render_style: MarkdownRenderStyle::Html,
            parameters_format: MarkdownDisplayFormat::Table,
            ..MarkdownDocsOptions::default()
        },
    );
    let page = html.get("mod/interfaces/PluginContext.md").unwrap();
    assert!(page.contains("<h6>Type Parameters</h6>"));
    assert!(page.contains("Extension context."));
    assert!(page.contains("<td><code>Fallback</code></td><td>-</td>"));
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
    let page = markdown.get("default/classes/DefaultTranslation.md").unwrap();

    assert!(page.contains("## Implements"));
    assert!(page.contains("TranslationAdapter"));
    assert!(page.contains("## Constructors"));
    assert!(page.contains("### Constructor"));
    assert!(page.contains(
        "new DefaultTranslation(options: TranslationAdapterFactoryOptions): DefaultTranslation;"
    ));
    assert!(page.contains("#### Returns"));
    assert!(page.contains("`DefaultTranslation`"));
    assert!(page.contains("## Methods"));
    assert!(page.contains("### getResource()"));
    assert!(page.contains("getResource(locale: string): Record<string, string> | undefined;"));
    assert!(page.contains("#### Parameters"));
    assert!(page.contains("| `locale` | `string` | Locale name. |"));
    assert!(page.contains("#### Returns"));
    assert!(page.contains("`Record<string, string> | undefined` — The locale resource."));
    assert!(!page.contains("`Record<string, string> \\| undefined`"));
    assert!(page.contains("#### Implementation of"));
    assert!(page.contains("TranslationAdapter.getResource"));
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
    let page = markdown.get("default/interfaces/ArgSchema.md").unwrap();

    assert!(page.contains("| `parse` _(optional)_ | `(value: string) => string \\| undefined` | Parses a raw value. |"));
    assert!(!page.contains("Parses a raw value. Returns: Parsed value."));
    assert!(page.contains("### parse Parameters"));
    assert!(page.contains("| `value` | `string` | Raw string value from command line. |"));
    assert!(page.contains("### parse Returns"));
    assert!(page.contains("`string | undefined` — Parsed value."));
    assert!(!page.contains("`string \\| undefined` — Parsed value."));
    assert!(!page.contains("`unknown`"));
}
