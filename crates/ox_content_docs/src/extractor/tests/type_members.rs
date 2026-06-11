use super::super::*;

#[test]
fn type_alias_intersection_extracts_object_literal_members() {
    let source = r"
/**
 * Command options.
 */
export type CommandOptions = BaseOptions & {
    /** Command name. */
    name: string;
};
";

    let extractor = DocExtractor::new();
    let items = extractor.extract_source(source, "options.ts", SourceType::ts()).unwrap();

    assert_eq!(items.len(), 1);
    assert_eq!(items[0].name, "CommandOptions");
    assert_eq!(items[0].children.len(), 1);
    let signature = items[0].signature.as_deref().unwrap();
    assert!(!signature.contains("/**"));
    assert!(signature.contains("BaseOptions & { name: string }"));
    assert_eq!(items[0].children[0].name, "name");
    assert_eq!(items[0].children[0].signature.as_deref(), Some("string"));
    assert!(items[0].signature.as_deref().unwrap().contains("BaseOptions &"));
}

#[test]
fn type_alias_intersection_resolves_callable_alias_and_members() {
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
    let items = extractor.extract_source(source, "plugin.ts", SourceType::ts()).unwrap();
    let plugin = items.iter().find(|item| item.name == "Plugin").unwrap();

    assert_eq!(plugin.params.len(), 1);
    assert_eq!(plugin.params[0].name, "ctx");
    assert_eq!(plugin.params[0].type_annotation.as_deref(), Some("Readonly<PluginContext<G>>"));
    assert_eq!(plugin.params[0].description, None);
    assert_eq!(plugin.return_type.as_deref(), Some("Awaitable<void>"));
    assert_eq!(
        plugin.children.iter().map(|child| child.name.as_str()).collect::<Vec<_>>(),
        ["id", "name"]
    );
    assert_eq!(plugin.children[0].signature.as_deref(), Some("string"));
    assert_eq!(plugin.children[1].signature.as_deref(), Some("string"));
    assert!(plugin.children[1].optional);
}

#[test]
fn function_valued_interface_property_extracts_params_and_returns() {
    let source = r"
/**
 * Options for parsing.
 */
export interface ArgSchema {
    /**
     * Parse a value.
     */
    parse?: (value: string) => any;
}
";

    let extractor = DocExtractor::new();
    let items = extractor.extract_source(source, "options.ts", SourceType::ts()).unwrap();
    let schema = items.iter().find(|item| item.name == "ArgSchema").unwrap();
    let parse = schema.children.iter().find(|member| member.name == "parse").unwrap();

    assert_eq!(parse.kind, DocItemKind::Property);
    assert_eq!(parse.signature.as_deref(), Some("(value: string) => any"));
    assert_eq!(parse.params.len(), 1);
    assert_eq!(parse.params[0].name, "value");
    assert_eq!(parse.params[0].type_annotation.as_deref(), Some("string"));
    assert_eq!(parse.return_type.as_deref(), Some("any"));
}

#[test]
fn function_valued_class_property_extracts_params_and_returns() {
    let source = r"
/**
 * Argument parser.
 */
export class ArgParser {
    /**
     * Parse a value.
     */
    parse: (value: string) => any;
}
";

    let extractor = DocExtractor::new();
    let items = extractor.extract_source(source, "parser.ts", SourceType::ts()).unwrap();
    let parser = items.iter().find(|item| item.name == "ArgParser").unwrap();
    let parse = parser.children.iter().find(|member| member.name == "parse").unwrap();

    assert_eq!(parse.kind, DocItemKind::Property);
    assert_eq!(parse.signature.as_deref(), Some("(value: string) => any"));
    assert_eq!(parse.params[0].name, "value");
    assert_eq!(parse.params[0].type_annotation.as_deref(), Some("string"));
    assert_eq!(parse.return_type.as_deref(), Some("any"));
}

#[test]
fn readonly_type_and_parenthesized_union_preserve_types() {
    let source = r"
/**
 * Command arguments.
 */
export interface ArgSchema {
    /**
     * Parse a value.
     */
    choices?: string[] | readonly string[];
}

/**
 * Example rendering hooks.
 */
export interface SubCommandable {
    examples?: string | ((...args: any[]) => any);
}
";

    let extractor = DocExtractor::new();
    let items = extractor.extract_source(source, "schemas.ts", SourceType::ts()).unwrap();
    let arg_schema = items.iter().find(|item| item.name == "ArgSchema").unwrap();
    let sub = items.iter().find(|item| item.name == "SubCommandable").unwrap();

    let choices = arg_schema.children.iter().find(|member| member.name == "choices").unwrap();
    assert_eq!(choices.signature.as_deref(), Some("string[] | readonly string[]"));

    let examples = sub.children.iter().find(|member| member.name == "examples").unwrap();
    assert_eq!(examples.signature.as_deref(), Some("string | ((...args: any[]) => any)"));
}
