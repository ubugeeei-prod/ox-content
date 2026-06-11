use oxc_span::SourceType;

use super::super::*;
use crate::extractor::DocExtractor;

#[test]
fn normalizes_jsdoc_types_and_custom_tags() {
    let source = r#"
/**
 * Creates a user-facing label.
 *
 * @param {string} value - The label source
 * @param {number} [maxLength=20] - Maximum length before truncation
 * @returns {string} Formatted label
 * @throws {RangeError} When maxLength is negative.
 * @exception {TypeError} When value is not a string.
 * @example
 * label("hello", 3)
 * @since 1.2.3
 */
export function label(value, maxLength = 20) {
    return value.slice(0, maxLength);
}
"#;

    let extractor = DocExtractor::new();
    let items = extractor.extract_source(source, "labels.js", SourceType::mjs()).unwrap();
    let entries = normalize_doc_items(items, false);

    assert_eq!(entries.len(), 1);
    let entry = &entries[0];
    assert_eq!(entry.name, "label");
    assert_eq!(entry.kind, NormalizedDocKind::Function);
    assert_eq!(entry.description, "Creates a user-facing label.");
    assert!(!entry.private);
    assert_eq!(entry.params.len(), 2);
    assert_eq!(entry.params[0].type_annotation, "string");
    assert_eq!(entry.params[0].description, "The label source");
    assert_eq!(entry.params[1].type_annotation, "number");
    assert!(entry.params[1].optional);
    assert_eq!(entry.params[1].default_value.as_deref(), Some("20"));
    assert_eq!(entry.params[1].description, "Maximum length before truncation");
    assert_eq!(
        entry.returns,
        Some(NormalizedReturnDoc {
            type_annotation: "string".to_string(),
            description: "Formatted label".to_string(),
            members: Vec::new()
        })
    );
    assert_eq!(
        entry.throws,
        vec![
            NormalizedThrowsDoc {
                type_annotation: Some("RangeError".to_string()),
                description: "When maxLength is negative.".to_string(),
            },
            NormalizedThrowsDoc {
                type_annotation: Some("TypeError".to_string()),
                description: "When value is not a string.".to_string(),
            }
        ]
    );
    assert_eq!(entry.examples, vec!["label(\"hello\", 3)"]);
    assert_eq!(entry.tags.get("since").map(String::as_str), Some("1.2.3"));
    assert!(!entry.tags.contains_key("throws"));
    assert!(!entry.tags.contains_key("exception"));
}

#[test]
fn preserves_private_flag_when_private_items_are_included() {
    let source = r"
/**
 * Internal helper.
 * @private
 */
export function internalHelper(): void {}
";

    let extractor = DocExtractor::with_private(true);
    let items = extractor.extract_source(source, "internal.ts", SourceType::ts()).unwrap();
    let entries = normalize_doc_items(items, false);

    assert_eq!(entries.len(), 1);
    assert!(entries[0].private);
}

#[test]
fn preserves_enum_kind_in_normalized_entries() {
    let source = r"
/**
 * Available modes.
 */
export enum Mode {
    Fast,
    Slow,
}
";

    let extractor = DocExtractor::new();
    let items = extractor.extract_source(source, "mode.ts", SourceType::ts()).unwrap();
    let entries = normalize_doc_items(items, false);

    assert_eq!(entries.len(), 1);
    assert_eq!(entries[0].kind, NormalizedDocKind::Enum);
}

#[test]
fn preserves_heritage_fields_in_normalized_entries() {
    let source = r"
/**
 * Base adapter.
 */
export interface BaseAdapter {}

/**
 * Runtime adapter.
 */
export interface TranslationAdapter extends BaseAdapter {}

/**
 * Default runtime adapter.
 */
export class DefaultTranslation implements TranslationAdapter {}
";

    let extractor = DocExtractor::new();
    let items = extractor.extract_source(source, "adapter.ts", SourceType::ts()).unwrap();
    let entries = normalize_doc_items(items, false);
    let adapter = entries.iter().find(|entry| entry.name == "TranslationAdapter").unwrap();
    let implementation = entries.iter().find(|entry| entry.name == "DefaultTranslation").unwrap();

    assert_eq!(adapter.extends, vec!["BaseAdapter"]);
    assert_eq!(implementation.implements, vec!["TranslationAdapter"]);
}
