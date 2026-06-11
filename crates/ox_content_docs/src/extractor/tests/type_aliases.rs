use super::super::*;

#[test]
fn type_alias_function_extracts_params_and_returns() {
    let source = r"
/**
 * Run a command.
 */
export type CommandRunner<G> = (ctx: Readonly<CommandContext<G>>) => Awaitable<string | void>;
";

    let extractor = DocExtractor::new();
    let items = extractor.extract_source(source, "runner.ts", SourceType::ts()).unwrap();
    let alias = items.iter().find(|item| item.name == "CommandRunner").unwrap();

    assert_eq!(alias.params.len(), 1);
    assert_eq!(alias.params[0].name, "ctx");
    assert_eq!(alias.params[0].type_annotation.as_deref(), Some("Readonly<CommandContext<G>>"));
    assert_eq!(alias.return_type.as_deref(), Some("Awaitable<string | void>"));
}

#[test]
fn type_alias_function_with_multiple_parameters_extracts_all_params_and_return() {
    let source = r"
/**
 * Extend a command.
 */
export type PluginExtension<T, G> = (ctx: CommandContextCore<G>, cmd: Command<G>) => Awaitable<T>;
";

    let extractor = DocExtractor::new();
    let items = extractor.extract_source(source, "plugin.ts", SourceType::ts()).unwrap();
    let alias = items.iter().find(|item| item.name == "PluginExtension").unwrap();

    assert_eq!(alias.params.len(), 2);
    assert_eq!(alias.params[0].name, "ctx");
    assert_eq!(alias.params[0].type_annotation.as_deref(), Some("CommandContextCore<G>"));
    assert_eq!(alias.params[1].name, "cmd");
    assert_eq!(alias.params[1].type_annotation.as_deref(), Some("Command<G>"));
    assert_eq!(alias.return_type.as_deref(), Some("Awaitable<T>"));
}

#[test]
fn type_alias_function_with_jsdoc_params_but_no_returns_tag_extracts_return() {
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
    let items = extractor.extract_source(source, "plugin.ts", SourceType::ts()).unwrap();
    let alias = items.iter().find(|item| item.name == "OnPluginExtension").unwrap();

    assert_eq!(alias.params.len(), 2);
    assert_eq!(alias.params[0].name, "ctx");
    assert_eq!(alias.params[0].type_annotation.as_deref(), Some("Readonly<CommandContext<G>>"));
    assert_eq!(alias.params[0].description.as_deref(), Some("The command context."));
    assert_eq!(alias.params[1].name, "cmd");
    assert_eq!(alias.params[1].type_annotation.as_deref(), Some("Readonly<Command<G>>"));
    assert_eq!(alias.params[1].description.as_deref(), Some("The command."));
    assert_eq!(alias.return_type.as_deref(), Some("Awaitable<void>"));
}

#[test]
fn type_alias_function_with_function_param_and_return_extracts_nested_function_types() {
    let source = r"
/**
 * Decorate a runner.
 */
export type CommandDecorator<G> = (
    baseRunner: (ctx: Readonly<CommandContext<G>>) => Awaitable<string | void>
) => (ctx: Readonly<CommandContext<G>>) => Awaitable<string | void>;
";

    let extractor = DocExtractor::new();
    let items = extractor.extract_source(source, "decorator.ts", SourceType::ts()).unwrap();
    let alias = items.iter().find(|item| item.name == "CommandDecorator").unwrap();

    assert_eq!(alias.params.len(), 1);
    assert_eq!(alias.params[0].name, "baseRunner");
    assert_eq!(
        alias.params[0].type_annotation.as_deref(),
        Some("(ctx: Readonly<CommandContext<G>>) => Awaitable<string | void>"),
    );
    assert_eq!(
        alias.return_type.as_deref(),
        Some("(ctx: Readonly<CommandContext<G>>) => Awaitable<string | void>"),
    );
}

#[test]
fn type_alias_function_without_jsdoc_tags_still_extracts_metadata() {
    let source = r"
/**
 * Plugin function.
 */
export type PluginFunction<G> = (ctx: Readonly<PluginContext<G>>) => Awaitable<void>;
";

    let extractor = DocExtractor::new();
    let items = extractor.extract_source(source, "plugin.ts", SourceType::ts()).unwrap();
    let alias = items.iter().find(|item| item.name == "PluginFunction").unwrap();

    assert_eq!(alias.params.len(), 1);
    assert_eq!(alias.params[0].name, "ctx");
    assert_eq!(alias.params[0].type_annotation.as_deref(), Some("Readonly<PluginContext<G>>"));
    assert_eq!(alias.return_type.as_deref(), Some("Awaitable<void>"));
}
