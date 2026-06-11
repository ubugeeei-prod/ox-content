use super::super::*;

#[test]
fn index_signatures_are_extracted_from_members_and_return_literals() {
    let source = r"
/**
 * Value type.
 */
export interface ArgSchema {}

/**
 * Arguments.
 */
export interface Args {
    /** Argument schema by option name. */
    readonly [option: string]: ArgSchema;
}

/**
 * Numeric arguments.
 */
export type NumericArgs = {
    [index: number]: ArgSchema;
};

/**
 * Argument store.
 */
export class Store {
    [key: string]: ArgSchema;
}

/**
 * Makes arguments.
 */
export function makeArgs(): {
    [key: string]: ArgSchema;
} {
    return {} as any;
}
";

    let extractor = DocExtractor::new();
    let items = extractor.extract_source(source, "args.ts", SourceType::ts()).unwrap();
    let args = items.iter().find(|item| item.name == "Args").unwrap();
    let numeric_args = items.iter().find(|item| item.name == "NumericArgs").unwrap();
    let store = items.iter().find(|item| item.name == "Store").unwrap();
    let make_args = items.iter().find(|item| item.name == "makeArgs").unwrap();

    let args_member = &args.children[0];
    assert_eq!(args_member.kind, DocItemKind::IndexSignature);
    assert_eq!(args_member.name, "[option: string]");
    assert_eq!(args_member.signature.as_deref(), Some("readonly [option: string]: ArgSchema"));
    assert_eq!(args_member.return_type.as_deref(), Some("ArgSchema"));
    assert_eq!(args_member.params[0].name, "option");
    assert_eq!(args_member.params[0].type_annotation.as_deref(), Some("string"));
    assert!(args_member.readonly);

    let numeric_member = &numeric_args.children[0];
    assert_eq!(numeric_member.kind, DocItemKind::IndexSignature);
    assert_eq!(numeric_member.name, "[index: number]");
    assert_eq!(numeric_member.signature.as_deref(), Some("[index: number]: ArgSchema"));

    let store_member = &store.children[0];
    assert_eq!(store_member.kind, DocItemKind::IndexSignature);
    assert_eq!(store_member.signature.as_deref(), Some("[key: string]: ArgSchema"));

    let return_member = &make_args.return_members[0];
    assert_eq!(return_member.kind, DocItemKind::IndexSignature);
    assert_eq!(return_member.signature.as_deref(), Some("[key: string]: ArgSchema"));
}

#[test]
fn test_extract_interface() {
    let source = r"
/**
 * User interface.
 */
export interface User {
    /** User's name */
    name: string;
    /** User's age */
    age: number;
}
";

    let extractor = DocExtractor::new();
    let items = extractor.extract_source(source, "test.ts", SourceType::ts()).unwrap();

    assert_eq!(items.len(), 1);
    assert_eq!(items[0].name, "User");
    assert_eq!(items[0].kind, DocItemKind::Interface);
    assert_eq!(items[0].children.len(), 2);
}

#[test]
fn extracts_interface_extends_and_class_implements() {
    let source = r"
/**
 * Base runtime adapter.
 */
export interface BaseAdapter {}

/**
 * Runtime adapter.
 */
export interface TranslationAdapter extends BaseAdapter {
    /**
     * Gets a locale resource.
     * @param locale - Locale name.
     * @returns The locale resource.
     */
    getResource(locale: string): Record<string, string> | undefined;
}

/**
 * Default runtime adapter.
 */
export class DefaultTranslation implements TranslationAdapter {
    /**
     * Gets a locale resource.
     * @param locale - Locale name.
     * @returns The locale resource.
     */
    getResource(locale: string): Record<string, string> | undefined {
        return undefined;
    }
}
";

    let extractor = DocExtractor::new();
    let items = extractor.extract_source(source, "adapter.ts", SourceType::ts()).unwrap();
    let adapter = items.iter().find(|item| item.name == "TranslationAdapter").unwrap();
    let implementation = items.iter().find(|item| item.name == "DefaultTranslation").unwrap();

    assert_eq!(adapter.kind, DocItemKind::Interface);
    assert_eq!(adapter.extends, vec!["BaseAdapter"]);
    assert_eq!(implementation.kind, DocItemKind::Class);
    assert_eq!(implementation.implements, vec!["TranslationAdapter"]);
    assert_eq!(
        implementation.signature.as_deref(),
        Some("export class DefaultTranslation implements TranslationAdapter")
    );
}
