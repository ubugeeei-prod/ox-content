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
        type_annotation: None,
        default_value: None,
        params: vec![ApiParamDoc {
            name: "locale".to_string(),
            type_annotation: "string".to_string(),
            description: "Locale name.".to_string(),
            optional: false,
            default_value: None,
        }],
        type_parameters: vec![],
        returns: Some(ApiReturnDoc {
            type_annotation: "Record<string, string> | undefined".to_string(),
            description: "The locale resource.".to_string(),
            members: Vec::new(),
        }),
        members: vec![],
        optional: false,
        readonly: false,
        r#static: false,
        private: false,
        tags: vec![],
        implementation_of: vec![],
        line: 4,
        end_line: 4,
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
        type_annotation: None,
        default_value: None,
        params: vec![ApiParamDoc {
            name: "locale".to_string(),
            type_annotation: "string".to_string(),
            description: "Locale name.".to_string(),
            optional: false,
            default_value: None,
        }],
        type_parameters: vec![],
        returns: Some(ApiReturnDoc {
            type_annotation: "Record<string, string> | undefined".to_string(),
            description: "The locale resource.".to_string(),
            members: Vec::new(),
        }),
        members: vec![],
        optional: false,
        readonly: false,
        r#static: false,
        private: false,
        tags: vec![],
        implementation_of: vec![],
        line: 14,
        end_line: 16,
    }];
    let docs = vec![ApiDocModule {
        description: String::new(),
        file: "default".to_string(),
        source_path: String::new(),
        examples: vec![],
        tags: vec![],
        entries: vec![adapter, implementation],
    }];

    let markdown = generate_markdown(&docs, &html_typedoc_options());
    let page = markdown.get("default/classes/DefaultTranslation.md").unwrap();

    assert!(page.contains("<h4>Implements</h4>"));
    assert!(page.contains("TranslationAdapter"));
    assert!(page.contains("ox-api-entry__member-group--details"));
    assert!(page.contains("<h5>getResource()</h5>"));
    assert!(page.contains("ox-api-entry__member-detail-section--params"));
    assert!(page.contains("<h6>Returns</h6>"));
    assert!(page.contains("ox-api-entry__member-detail-section--implementation-of"));
    assert!(page.contains("TranslationAdapter.getResource"));
}
