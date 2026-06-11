use super::super::*;

#[test]
fn test_extract_jsdoc_types_from_javascript() {
    let source = r"
/**
 * Creates a user-facing label.
 *
 * @param {string} value - The label source
 * @param {number} [maxLength=20] - Maximum length before truncation
 * @returns {string} Formatted label
 */
export function label(value, maxLength = 20) {
    return value.slice(0, maxLength);
}
";

    let extractor = DocExtractor::new();
    let items = extractor.extract_source(source, "test.js", SourceType::mjs()).unwrap();

    assert_eq!(items.len(), 1);
    assert_eq!(items[0].name, "label");
    assert_eq!(items[0].doc.as_deref(), Some("Creates a user-facing label."));
    assert_eq!(items[0].return_type.as_deref(), Some("string"));
    assert_eq!(items[0].params.len(), 2);
    assert_eq!(items[0].params[0].type_annotation.as_deref(), Some("string"));
    assert_eq!(items[0].params[0].description.as_deref(), Some("The label source"));
    assert_eq!(items[0].params[1].type_annotation.as_deref(), Some("number"));
    assert!(items[0].params[1].optional);
    assert_eq!(items[0].params[1].default_value.as_deref(), Some("20"));
    assert_eq!(items[0].params[1].description.as_deref(), Some("Maximum length before truncation"));

    let value_tag = items[0]
        .tags
        .iter()
        .find(|tag| tag.tag == "param" && tag.name.as_deref() == Some("value"))
        .unwrap();
    assert_eq!(value_tag.type_annotation.as_deref(), Some("string"));
    assert_eq!(value_tag.description.as_deref(), Some("The label source"));

    let max_length_tag = items[0]
        .tags
        .iter()
        .find(|tag| tag.tag == "param" && tag.name.as_deref() == Some("maxLength"))
        .unwrap();
    assert_eq!(max_length_tag.type_annotation.as_deref(), Some("number"));
    assert_eq!(max_length_tag.optional, Some(true));
    assert_eq!(max_length_tag.default_value.as_deref(), Some("20"));

    let returns_tag = items[0].tags.iter().find(|tag| tag.tag == "returns").unwrap();
    assert_eq!(returns_tag.type_annotation.as_deref(), Some("string"));
    assert_eq!(returns_tag.description.as_deref(), Some("Formatted label"));
}

#[test]
fn test_extract_plain_top_level_variable() {
    let source = r"
/** Default placeholder when a command has no explicit name. */
export const ANONYMOUS_COMMAND_NAME = '__anonymous__';

/** Default retry count. */
export let retries: number = 3;

/** Creates labels. */
export const label = (value: string): string => value;
";

    let extractor = DocExtractor::new();
    let items = extractor.extract_source(source, "constants.ts", SourceType::ts()).unwrap();

    assert_eq!(items.len(), 3);
    assert_eq!(items[0].name, "ANONYMOUS_COMMAND_NAME");
    assert_eq!(items[0].kind, DocItemKind::Variable);
    assert_eq!(
        items[0].signature.as_deref(),
        Some("export const ANONYMOUS_COMMAND_NAME = '__anonymous__'")
    );
    assert_eq!(items[1].name, "retries");
    assert_eq!(items[1].kind, DocItemKind::Variable);
    assert_eq!(items[1].signature.as_deref(), Some("export let retries: number"));
    assert_eq!(items[2].name, "label");
    assert_eq!(items[2].kind, DocItemKind::Function);
}

#[test]
fn test_undocumented_top_level_variable_is_skipped_by_default() {
    let source = "export const ANONYMOUS_COMMAND_NAME = '(anonymous)';";

    let items =
        DocExtractor::new().extract_source(source, "constants.ts", SourceType::ts()).unwrap();

    assert!(items.is_empty());
}
