use super::*;

#[test]
fn extract_file_doc_entries_preserves_type_alias_return_without_returns_tag() {
    let unique =
        std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_nanos();
    let root = std::env::temp_dir()
        .join(format!("ox-content-napi-type-alias-return-{}-{unique}", std::process::id()));
    let _ = fs::remove_dir_all(&root);
    fs::create_dir_all(&root).unwrap();
    let file = root.join("plugin.ts");
    fs::write(
        &file,
        r"
/**
 * Plugin extension hook.
 *
 * @param ctx - The command context.
 * @param cmd - The command.
 */
export type OnPluginExtension<G> = (
    ctx: Readonly<CommandContext<G>>,
    cmd: Readonly<Command<G>>
) => Awaitable<void>;
",
    )
    .unwrap();

    let entries =
        extract_file_doc_entries(file.to_string_lossy().into_owned(), None, None, None).unwrap();
    let entry = entries.iter().find(|entry| entry.name == "OnPluginExtension").unwrap();
    let params = entry.params.as_ref().unwrap();
    let returns = entry.returns.as_ref().unwrap();

    assert_eq!(params.len(), 2);
    assert_eq!(params[0].r#type, "Readonly<CommandContext<G>>");
    assert_eq!(params[0].description, "The command context.");
    assert_eq!(params[1].r#type, "Readonly<Command<G>>");
    assert_eq!(params[1].description, "The command.");
    assert_eq!(returns.r#type, "Awaitable<void>");
    assert_eq!(returns.description, "");

    let _ = fs::remove_dir_all(root);
}
#[test]
fn extract_file_doc_entries_preserves_object_literal_parameter_members() {
    let unique =
        std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_nanos();
    let root = std::env::temp_dir()
        .join(format!("ox-content-napi-object-literal-param-{}-{unique}", std::process::id()));
    let _ = fs::remove_dir_all(&root);
    fs::create_dir_all(&root).unwrap();
    let file = root.join("plugin.ts");
    fs::write(
        &file,
        r"
/**
 * Define a plugin.
 *
 * @param options - Plugin options.
 * @param options.id - Plugin id.
 * @param options.name - Plugin display name.
 */
export function plugin<Id, PluginExt>(options: {
    id: Id;
    name?: string;
    setup?: (
        ctx: Readonly<
            PluginContext
        >
    ) => Awaitable<void>;
    extension: PluginExt;
}): PluginWithExtension<PluginExt>;
",
    )
    .unwrap();

    let entries =
        extract_file_doc_entries(file.to_string_lossy().into_owned(), None, None, None).unwrap();
    let entry = entries.iter().find(|entry| entry.name == "plugin").unwrap();
    let params = entry.params.as_ref().unwrap();

    assert_eq!(params.len(), 5);
    assert_eq!(params[0].name, "options");
    assert_ne!(params[0].r#type, "{ ... }");
    assert!(params[0].r#type.contains("id: Id"));
    assert!(params[0].r#type.contains("name?: string"));
    assert!(params[0].r#type.contains("setup?: (ctx: Readonly<PluginContext>) => Awaitable<void>"));
    assert_eq!(params[0].description, "Plugin options.");
    assert_eq!(params[1].name, "options.id");
    assert_eq!(params[1].r#type, "Id");
    assert_eq!(params[1].description, "Plugin id.");
    assert_eq!(params[2].name, "options.name?");
    assert_eq!(params[2].description, "Plugin display name.");
    assert_eq!(params[2].optional, Some(true));
    assert_eq!(params[3].name, "options.setup?");
    assert_eq!(params[3].r#type, "(ctx: Readonly<PluginContext>) => Awaitable<void>");
    assert_eq!(params[4].name, "options.extension");
    assert_eq!(params[4].r#type, "PluginExt");

    let _ = fs::remove_dir_all(root);
}

#[test]
fn has_body_round_trips_from_extract_output_to_markdown_model() {
    let normalized = NormalizedDocEntry {
        name: "plugin".to_string(),
        kind: NormalizedDocKind::Function,
        description: String::new(),
        params: vec![],
        returns: None,
        throws: vec![],
        examples: vec![],
        tags: BTreeMap::new(),
        private: false,
        file: "plugin.ts".to_string(),
        line: 1,
        end_line: 1,
        signature: Some("export function plugin(): void".to_string()),
        extends: vec![],
        implements: vec![],
        has_body: true,
        members: vec![],
        type_parameters: vec![],
    };

    // `extractDocsFromEntryPoints` output exposes the flag ...
    let js_entry = map_normalized_doc_entry(normalized);
    assert!(js_entry.has_body);

    // ... and `generateDocsMarkdown` input round-trips it back into the model
    // (gunshi's `{ ...entry }` spread carries it across the boundary).
    let markdown_entry = JsDocsMarkdownEntry {
        name: js_entry.name,
        kind: js_entry.kind,
        description: js_entry.description,
        params: None,
        returns: None,
        throws: None,
        examples: None,
        tags: None,
        private: js_entry.private,
        file: js_entry.file,
        line: js_entry.line,
        end_line: js_entry.end_line,
        signature: js_entry.signature,
        extends: None,
        implements: None,
        has_body: Some(js_entry.has_body),
        members: None,
        type_parameters: None,
    };
    assert!(convert_markdown_entry(markdown_entry).has_body);
}

#[test]
fn convert_markdown_entry_defaults_has_body_to_false_when_absent() {
    let entry = JsDocsMarkdownEntry {
        name: "Command".to_string(),
        kind: "interface".to_string(),
        description: String::new(),
        params: None,
        returns: None,
        throws: None,
        examples: None,
        tags: None,
        private: false,
        file: "command.ts".to_string(),
        line: 1,
        end_line: 1,
        signature: Some("export interface Command".to_string()),
        extends: None,
        implements: None,
        has_body: None,
        members: None,
        type_parameters: None,
    };

    assert!(!convert_markdown_entry(entry).has_body);
}

#[test]
fn convert_markdown_entry_preserves_heritage_and_implementation_metadata() {
    let entry = JsDocsMarkdownEntry {
        name: "DefaultTranslation".to_string(),
        kind: "class".to_string(),
        description: "Default adapter.".to_string(),
        params: None,
        returns: None,
        throws: Some(vec![JsDocThrows {
            r#type: Some("AdapterError".to_string()),
            description: "When adapter metadata is invalid.".to_string(),
        }]),
        examples: None,
        tags: None,
        private: false,
        file: "adapter.ts".to_string(),
        line: 1,
        end_line: 10,
        signature: Some("class DefaultTranslation implements TranslationAdapter".to_string()),
        extends: Some(vec!["BaseTranslation".to_string()]),
        implements: Some(vec!["TranslationAdapter".to_string()]),
        has_body: None,
        members: Some(vec![JsDocMember {
            name: "getResource".to_string(),
            kind: "method".to_string(),
            description: "Gets a locale resource.".to_string(),
            signature: Some(
                "getResource(locale: string): Record<string, string> | undefined".to_string(),
            ),
            r#type: None,
            r#default: Some("undefined".to_string()),
            params: None,
            type_parameters: Some(vec![JsTypeParam {
                name: "L".to_string(),
                constraint: Some("Base".to_string()),
                r#default: Some("Default".to_string()),
                description: "Locale type.".to_string(),
            }]),
            returns: None,
            throws: Some(vec![JsDocThrows {
                r#type: Some("ResourceError".to_string()),
                description: "When resource loading fails.".to_string(),
            }]),
            members: None,
            optional: None,
            readonly: None,
            r#static: None,
            private: None,
            tags: None,
            implementation_of: Some(vec!["TranslationAdapter.getResource".to_string()]),
            line: 5,
            end_line: 8,
        }]),
        type_parameters: None,
    };

    let converted = convert_markdown_entry(entry);

    assert_eq!(converted.extends, vec!["BaseTranslation"]);
    assert_eq!(converted.implements, vec!["TranslationAdapter"]);
    assert_eq!(converted.members[0].implementation_of, vec!["TranslationAdapter.getResource"]);
    assert_eq!(converted.members[0].default_value.as_deref(), Some("undefined"));
    assert_eq!(converted.members[0].type_parameters[0].name, "L");
    assert_eq!(converted.members[0].type_parameters[0].constraint.as_deref(), Some("Base"));
    assert_eq!(converted.throws[0].type_annotation.as_deref(), Some("AdapterError"));
    assert_eq!(converted.throws[0].description, "When adapter metadata is invalid.");
    assert_eq!(converted.members[0].throws[0].type_annotation.as_deref(), Some("ResourceError"));
    assert_eq!(converted.members[0].throws[0].description, "When resource loading fails.");
}
