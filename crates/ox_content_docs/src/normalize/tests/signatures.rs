use oxc_span::SourceType;

use super::super::*;
use crate::extractor::DocExtractor;

fn normalize_source(source: &str) -> Vec<NormalizedDocEntry> {
    let extractor = DocExtractor::new();
    let items = extractor.extract_source(source, "signatures.ts", SourceType::ts()).unwrap();
    normalize_doc_items(items, true)
}

fn entry<'a>(entries: &'a [NormalizedDocEntry], name: &str) -> &'a NormalizedDocEntry {
    entries.iter().find(|entry| entry.name == name).unwrap()
}

#[test]
fn interface_call_and_construct_signatures_emit_members() {
    let source = r"
/** Translate */
export interface Translate {
    /**
     * Translate a key
     * @typeParam Key - The key type
     * @param key - A key
     * @returns A message
     */
    <Key extends string>(key: Key): string;
    /**
     * Translate with a plural
     * @param key - A key
     * @param plural - A plural choice
     */
    (key: string, plural: number): string;
    /** Construct it */
    new (locale: string): Translate;
    /** The locale */
    locale: string;
}
";

    let entries = normalize_source(source);
    let members = &entries[0].members;
    let kinds = members.iter().map(|member| member.kind.as_str()).collect::<Vec<_>>();
    assert_eq!(kinds, ["callSignature", "callSignature", "constructSignature", "property"]);
    assert!(members[..3].iter().all(|member| member.name.is_empty()));

    let generic = &members[0];
    assert_eq!(generic.signature.as_deref(), Some("<Key extends string>(key: Key): string"));
    assert_eq!(generic.type_annotation, None);
    assert_eq!(generic.description, "Translate a key");
    assert_eq!(generic.type_parameters.len(), 1);
    assert_eq!(generic.type_parameters[0].name, "Key");
    assert_eq!(generic.type_parameters[0].constraint.as_deref(), Some("string"));
    assert_eq!(generic.type_parameters[0].description, "The key type");
    assert_eq!(generic.params.len(), 1);
    assert_eq!(generic.params[0].name, "key");
    assert_eq!(generic.params[0].type_annotation, "Key");
    assert_eq!(generic.params[0].description, "A key");
    let returns = generic.returns.as_ref().unwrap();
    assert_eq!(returns.type_annotation, "string");
    assert_eq!(returns.description, "A message");

    let plural = &members[1];
    assert_eq!(plural.signature.as_deref(), Some("(key: string, plural: number): string"));
    assert_eq!(
        plural.params.iter().map(|param| param.name.as_str()).collect::<Vec<_>>(),
        ["key", "plural"]
    );
    assert_eq!(plural.params[1].description, "A plural choice");

    let construct = &members[2];
    assert_eq!(construct.signature.as_deref(), Some("new (locale: string): Translate"));
    assert_eq!(construct.description, "Construct it");
    assert_eq!(construct.params[0].name, "locale");
    assert_eq!(construct.returns.as_ref().unwrap().type_annotation, "Translate");
}

#[test]
fn type_literal_signatures_emit_members_in_aliases_properties_and_returns() {
    let source = r"
/** A type literal with a call signature */
export type CallType = {
    (a: string): string;
};

/** Options */
export interface Options {
    /** A handler */
    handler: {
        /** Call the handler */
        (event: string): void;
        /** The handler name */
        name: string;
    };
}

/**
 * Create a formatter
 * @returns A formatter
 */
export function createFormatter(): {
    /** Format a value */
    (value: number): string;
    /** The locale */
    locale: string;
} {
    return Object.assign((value: number) => String(value), { locale: 'en' });
}

/** Make a translator */
export function makeTranslator(): {
    /** Translate a key */
    (key: string): string;
} {
    return (key: string) => key;
}
";

    let entries = normalize_source(source);

    let call_type = &entry(&entries, "CallType").members[0];
    assert_eq!(call_type.kind, NormalizedMemberKind::CallSignature);
    assert_eq!(call_type.signature.as_deref(), Some("(a: string): string"));

    let handler = &entry(&entries, "Options").members[0].members;
    assert_eq!(handler[0].kind, NormalizedMemberKind::CallSignature);
    assert_eq!(handler[0].description, "Call the handler");
    assert_eq!(handler[1].name, "name");

    let returns = entry(&entries, "createFormatter").returns.as_ref().unwrap();
    assert_eq!(returns.members[0].kind, NormalizedMemberKind::CallSignature);
    assert_eq!(returns.members[0].signature.as_deref(), Some("(value: number): string"));
    assert_eq!(returns.members[0].description, "Format a value");
    assert_eq!(returns.members[1].name, "locale");
    assert_eq!(returns.type_annotation, "object");

    // A literal with only signatures keeps its own text instead of `object`.
    let translator = entry(&entries, "makeTranslator").returns.as_ref().unwrap();
    assert_eq!(translator.type_annotation, "{ (key: string): string; }");
    assert_eq!(translator.members[0].kind, NormalizedMemberKind::CallSignature);
}

#[test]
fn internal_signatures_are_skipped() {
    let source = r"
/** Parser */
export interface Parser {
    /** Parse a string */
    (source: string): string;
    /** @internal */
    (source: string, debug: boolean): string;
}
";

    let entries = normalize_source(source);
    let members = &entries[0].members;

    assert_eq!(members.len(), 1);
    assert_eq!(members[0].signature.as_deref(), Some("(source: string): string"));
}

#[test]
fn declaration_docs_move_to_an_undocumented_sole_call_signature() {
    let source = r"
/**
 * A translator
 * @param key - The key
 * @returns The message
 */
export type Translator = {
    (key: string): string;
};

/**
 * A documented signature keeps its own docs
 * @param key - Declared key
 */
export interface Documented {
    /**
     * Translate it
     * @param key - Own key
     */
    (key: string): string;
}

/**
 * Two signatures keep the declaration docs
 * @param key - The key
 */
export interface Overloaded {
    (key: string): string;
    (key: string, plural: number): string;
}

/**
 * A param name that the signature does not have
 * @param value - The value
 */
export interface Mismatched {
    (key: string): string;
}
";

    let entries = normalize_source(source);

    let translator = entry(&entries, "Translator");
    assert!(translator.params.is_empty());
    assert!(translator.returns.is_none());
    let signature = &translator.members[0];
    assert_eq!(signature.params[0].type_annotation, "string");
    assert_eq!(signature.params[0].description, "The key");
    let returns = signature.returns.as_ref().unwrap();
    assert_eq!(returns.type_annotation, "string");
    assert_eq!(returns.description, "The message");

    let documented = entry(&entries, "Documented");
    assert_eq!(documented.params[0].description, "Declared key");
    assert_eq!(documented.members[0].params[0].description, "Own key");

    let overloaded = entry(&entries, "Overloaded");
    assert_eq!(overloaded.params[0].description, "The key");
    assert!(overloaded.members.iter().all(|member| member.params[0].description.is_empty()));

    let mismatched = entry(&entries, "Mismatched");
    assert_eq!(mismatched.params[0].name, "value");
    assert!(mismatched.members[0].params[0].description.is_empty());
}

#[test]
fn declaration_docs_keep_signature_types_and_skip_documented_or_extracted_cases() {
    let source = r"
/**
 * A typed tag does not change the signature's types
 * @param {number} key - The key
 * @returns {number} The message
 */
export type Typed = {
    (key: string): unknown;
};

/**
 * A signature with only a throws tag has docs of its own
 * @param key - The key
 */
export interface Throwing {
    /** @throws {Error} When the key is empty */
    (key: string): string;
}

/**
 * An intersection takes its params from the function type
 * @param value - The value
 * @returns The result
 */
export type Convert = ((value: number) => boolean) & {
    (value: string): string;
};
";

    let entries = normalize_source(source);

    let typed = &entry(&entries, "Typed").members[0];
    assert_eq!(typed.params[0].type_annotation, "string");
    assert_eq!(typed.params[0].description, "The key");
    let returns = typed.returns.as_ref().unwrap();
    assert_eq!(returns.type_annotation, "unknown");
    assert_eq!(returns.description, "The message");

    let throwing = entry(&entries, "Throwing");
    assert_eq!(throwing.params[0].description, "The key");
    assert!(throwing.members[0].params[0].description.is_empty());

    let convert = entry(&entries, "Convert");
    assert_eq!(convert.params[0].type_annotation, "number");
    let signature = &convert.members[0];
    assert_eq!(signature.params[0].type_annotation, "string");
    assert!(signature.params[0].description.is_empty());
    let returns = signature.returns.as_ref().unwrap();
    assert_eq!(returns.type_annotation, "string");
    assert!(returns.description.is_empty());
}
