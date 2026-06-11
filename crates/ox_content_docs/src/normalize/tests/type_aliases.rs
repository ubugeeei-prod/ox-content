use oxc_span::SourceType;

use super::super::*;
use crate::extractor::DocExtractor;

#[test]
fn function_type_alias_metadata_keeps_extracted_types_and_jsdoc_descriptions() {
    let source = r"
/**
 * Run a command.
 * @param ctx - Command execution context.
 * @returns CLI output result.
 */
export type CommandRunner<G> = (ctx: Readonly<CommandContext<G>>) => Awaitable<string | void>;
";

    let extractor = DocExtractor::new();
    let entries = normalize_doc_items(
        extractor.extract_source(source, "runner.ts", SourceType::ts()).unwrap(),
        false,
    );
    let alias = entries.iter().find(|entry| entry.name == "CommandRunner").unwrap();

    assert_eq!(alias.params.len(), 1);
    assert_eq!(alias.params[0].name, "ctx");
    assert_eq!(alias.params[0].type_annotation, "Readonly<CommandContext<G>>");
    assert_eq!(alias.params[0].description, "Command execution context.");
    let returns = alias.returns.as_ref().unwrap();
    assert_eq!(returns.type_annotation, "Awaitable<string | void>");
    assert_eq!(returns.description, "CLI output result.");
}

#[test]
fn function_type_alias_without_jsdoc_tags_still_has_type_information() {
    let source = r"
/**
 * Plugin function.
 */
export type PluginFunction<G> = (ctx: Readonly<PluginContext<G>>) => Awaitable<void>;
";

    let extractor = DocExtractor::new();
    let entries = normalize_doc_items(
        extractor.extract_source(source, "plugin.ts", SourceType::ts()).unwrap(),
        false,
    );
    let alias = entries.iter().find(|entry| entry.name == "PluginFunction").unwrap();

    assert_eq!(alias.params.len(), 1);
    assert_eq!(alias.params[0].name, "ctx");
    assert_eq!(alias.params[0].type_annotation, "Readonly<PluginContext<G>>");
    assert_eq!(alias.params[0].description, "");
    assert_eq!(alias.returns.as_ref().unwrap().type_annotation, "Awaitable<void>");
}

#[test]
fn intersection_type_alias_merges_callable_reference_metadata() {
    let source = r"
/**
 * Plugin function.
 */
export type PluginFunction<G> = (ctx: Readonly<PluginContext<G>>) => Awaitable<void>;

/**
 * Plugin.
 * @param ctx - Plugin context.
 * @returns Plugin setup result.
 */
export type Plugin<E> = PluginFunction & {
    id: string;
    name?: string;
};
";

    let extractor = DocExtractor::new();
    let entries = normalize_doc_items(
        extractor.extract_source(source, "plugin.ts", SourceType::ts()).unwrap(),
        false,
    );
    let alias = entries.iter().find(|entry| entry.name == "Plugin").unwrap();

    assert_eq!(alias.params.len(), 1);
    assert_eq!(alias.params[0].name, "ctx");
    assert_eq!(alias.params[0].type_annotation, "Readonly<PluginContext<G>>");
    assert_eq!(alias.params[0].description, "Plugin context.");
    let returns = alias.returns.as_ref().unwrap();
    assert_eq!(returns.type_annotation, "Awaitable<void>");
    assert_eq!(returns.description, "Plugin setup result.");
    assert_eq!(
        alias.members.iter().map(|member| member.name.as_str()).collect::<Vec<_>>(),
        ["id", "name"]
    );
    assert_eq!(alias.members[0].type_annotation.as_deref(), Some("string"));
    assert!(alias.members[1].optional);
}

#[test]
fn function_type_alias_without_returns_tag_still_normalizes_return_section() {
    let source = r"
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
";

    let extractor = DocExtractor::new();
    let entries = normalize_doc_items(
        extractor.extract_source(source, "plugin.ts", SourceType::ts()).unwrap(),
        false,
    );
    let alias = entries.iter().find(|entry| entry.name == "OnPluginExtension").unwrap();

    assert_eq!(alias.params.len(), 2);
    assert_eq!(alias.params[0].name, "ctx");
    assert_eq!(alias.params[0].type_annotation, "Readonly<CommandContext<G>>");
    assert_eq!(alias.params[0].description, "The command context.");
    assert_eq!(alias.params[1].name, "cmd");
    assert_eq!(alias.params[1].type_annotation, "Readonly<Command<G>>");
    assert_eq!(alias.params[1].description, "The command.");
    let returns = alias.returns.as_ref().unwrap();
    assert_eq!(returns.type_annotation, "Awaitable<void>");
    assert_eq!(returns.description, "");
}
