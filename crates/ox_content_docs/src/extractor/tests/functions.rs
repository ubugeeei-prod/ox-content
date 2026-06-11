use super::super::*;

#[test]
fn extract_overloaded_function_marks_only_implementation_has_body() {
    let source = r"
/**
 * Define a plugin with extension.
 */
export function plugin<E>(options: WithExt): WithExtResult;
/**
 * Define a plugin without extension.
 */
export function plugin(options: WithoutExt): WithoutExtResult;
/**
 * Define a plugin.
 */
export function plugin(options: any = {}): any {
    return options;
}
";

    let extractor = DocExtractor::new();
    let items = extractor.extract_source(source, "plugin.ts", SourceType::ts()).unwrap();
    let plugins = items.iter().filter(|item| item.name == "plugin").collect::<Vec<_>>();

    assert_eq!(plugins.len(), 3);
    // Overload signatures carry no body; only the implementation does.
    assert!(!plugins[0].has_body);
    assert!(!plugins[1].has_body);
    assert!(plugins[2].has_body);
}

#[test]
fn test_extract_function() {
    let source = r"
/**
 * Adds two numbers together.
 * @param a - The first number
 * @param b - The second number
 * @returns The sum of a and b
 */
export function add(a: number, b: number): number {
    return a + b;
}
";

    let extractor = DocExtractor::new();
    let items = extractor.extract_source(source, "test.ts", SourceType::ts()).unwrap();

    assert_eq!(items.len(), 1);
    assert_eq!(items[0].name, "add");
    assert_eq!(items[0].kind, DocItemKind::Function);
    assert!(items[0].exported);
    assert!(items[0].doc.as_ref().unwrap().contains("Adds two numbers"));
    assert_eq!(items[0].params.len(), 2);
}

#[test]
fn function_object_literal_parameter_preserves_type_and_members() {
    let source = r"
/**
 * Define a plugin.
 *
 * @param options - Plugin options.
 * @param options.id - Plugin id.
 * @param options.name - Plugin display name.
 * @param options.setup - Setup hook.
 */
export function plugin<Id, Deps, PluginExt, MergedExtensions>(options: {
    id: Id;
    name?: string;
    dependencies?: Deps;
    setup?: (
        ctx: Readonly<
            PluginContext<MergedExtensions>
        >
    ) => Awaitable<void>;
    extension: PluginExt;
    onExtension?: OnPluginExtension<MergedExtensions>;
}): PluginWithExtension<PluginExt>;
";

    let extractor = DocExtractor::new();
    let items = extractor.extract_source(source, "plugin.ts", SourceType::ts()).unwrap();
    let plugin = items.iter().find(|item| item.name == "plugin").unwrap();

    assert_eq!(plugin.params.len(), 7);
    assert_eq!(plugin.params[0].name, "options");
    let parent_type = plugin.params[0].type_annotation.as_deref().unwrap();
    assert_ne!(parent_type, "{ ... }");
    assert!(parent_type.contains("id: Id"));
    assert!(parent_type.contains("name?: string"));
    assert!(parent_type
        .contains("setup?: (ctx: Readonly<PluginContext<MergedExtensions>>) => Awaitable<void>"));
    assert_eq!(plugin.params[0].description.as_deref(), Some("Plugin options."));

    let names = plugin.params.iter().map(|param| param.name.as_str()).collect::<Vec<_>>();
    assert_eq!(
        names,
        [
            "options",
            "options.id",
            "options.name?",
            "options.dependencies?",
            "options.setup?",
            "options.extension",
            "options.onExtension?",
        ]
    );
    assert_eq!(plugin.params[1].type_annotation.as_deref(), Some("Id"));
    assert_eq!(plugin.params[1].description.as_deref(), Some("Plugin id."));
    assert_eq!(plugin.params[2].description.as_deref(), Some("Plugin display name."));
    assert_eq!(
        plugin.params[4].type_annotation.as_deref(),
        Some("(ctx: Readonly<PluginContext<MergedExtensions>>) => Awaitable<void>")
    );
    assert_eq!(plugin.params[4].description.as_deref(), Some("Setup hook."));
    assert!(plugin.params[2].optional);
    assert!(plugin.params[3].optional);
    assert!(plugin.params[4].optional);
    assert!(plugin.params[6].optional);
}

#[test]
fn destructured_parameter_uses_jsdoc_name_and_extracted_type() {
    let source = r"
/**
 * Resolve command line arguments.
 *
 * @param args - Argument schema.
 * @param tokens - Parsed tokens.
 * @param resolveArgs - Resolve options.
 */
export declare function resolveArgs<A extends Args>(
    args: A,
    tokens: ArgToken[],
    { shortGrouping, skipPositional, toKebab }?: ResolveArgs
): void;
";

    let extractor = DocExtractor::new();
    let items = extractor.extract_source(source, "resolver.ts", SourceType::ts()).unwrap();
    let resolve = items.iter().find(|item| item.name == "resolveArgs").unwrap();

    assert_eq!(
        resolve.params.iter().map(|param| param.name.as_str()).collect::<Vec<_>>(),
        ["args", "tokens", "resolveArgs"]
    );
    assert_eq!(resolve.params[2].type_annotation.as_deref(), Some("ResolveArgs"));
    assert!(resolve.params[2].optional);
    assert_eq!(resolve.params[2].description.as_deref(), Some("Resolve options."));
}

#[test]
fn destructured_parameter_without_jsdoc_name_keeps_param_fallback() {
    let source = r"
/**
 * Run a command.
 */
export declare function run({ cwd }: RunOptions): void;
";

    let extractor = DocExtractor::new();
    let items = extractor.extract_source(source, "runner.ts", SourceType::ts()).unwrap();
    let run = items.iter().find(|item| item.name == "run").unwrap();

    assert_eq!(run.params.len(), 1);
    assert_eq!(run.params[0].name, "param");
    assert_eq!(run.params[0].type_annotation.as_deref(), Some("RunOptions"));
    assert_eq!(run.params[0].description, None);
}

#[test]
fn destructured_parameter_keeps_nested_param_tags_on_members() {
    let source = r"
/**
 * Run a command.
 *
 * @param options - Runtime options.
 * @param options.cwd - Working directory.
 */
export declare function run({ cwd }: { cwd: string }): void;
";

    let extractor = DocExtractor::new();
    let items = extractor.extract_source(source, "runner.ts", SourceType::ts()).unwrap();
    let run = items.iter().find(|item| item.name == "run").unwrap();

    assert_eq!(
        run.params.iter().map(|param| param.name.as_str()).collect::<Vec<_>>(),
        ["options", "options.cwd"]
    );
    assert_eq!(run.params[0].type_annotation.as_deref(), Some("{ cwd: string }"));
    assert_eq!(run.params[0].description.as_deref(), Some("Runtime options."));
    assert_eq!(run.params[1].type_annotation.as_deref(), Some("string"));
    assert_eq!(run.params[1].description.as_deref(), Some("Working directory."));
}

#[test]
fn function_return_type_literal_emits_return_members() {
    let source = r"
/**
 * Resolve arguments.
 * @returns Resolved args.
 */
export function resolveArgs<A extends Args>(): {
    values: ArgValues<A>;
    positionals: string[];
    error: AggregateError | undefined;
} {
    return {} as any;
}
";

    let extractor = DocExtractor::new();
    let items = extractor.extract_source(source, "resolver.ts", SourceType::ts()).unwrap();

    assert_eq!(items.len(), 1);
    assert_eq!(items[0].return_type.as_deref(), Some("object"));
    assert_eq!(
        items[0].return_members.iter().map(|member| member.name.as_str()).collect::<Vec<_>>(),
        ["values", "positionals", "error"]
    );
    assert_eq!(items[0].return_members[0].signature.as_deref(), Some("ArgValues<A>"));
    assert_eq!(items[0].return_members[1].signature.as_deref(), Some("string[]"));
    assert_eq!(items[0].return_members[2].signature.as_deref(), Some("AggregateError | undefined"));
}
