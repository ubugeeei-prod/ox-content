use super::*;

#[test]
fn typedoc_html_renders_class_callable_member_details() {
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
    implementation.members = vec![ApiDocMember {
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
        throws: vec![ApiThrowsDoc {
            type_annotation: Some("ResourceError".to_string()),
            description: "When the locale resource cannot be loaded.".to_string(),
        }],
        line: 14,
        end_line: 16,
        ..ApiDocMember::default()
    }];
    let docs = vec![ApiDocModule {
        file: "default".to_string(),
        entries: vec![adapter, implementation],
        ..ApiDocModule::default()
    }];

    let markdown = generate_markdown(&docs, &html_typedoc_options());
    assert_markdown_map_snapshot(
        "typedoc_html_renders_class_callable_member_details__markdown",
        &markdown,
    );
    assert_markdown_map_snapshot(
        "typedoc_html_renders_class_callable_member_details__markdown",
        &markdown,
    );
}
