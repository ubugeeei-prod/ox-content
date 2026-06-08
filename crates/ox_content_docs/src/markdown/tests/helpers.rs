use super::*;

pub(super) fn test_entry(name: &str, kind: &str, file: &str, description: &str) -> ApiDocEntry {
    ApiDocEntry {
        name: name.to_string(),
        kind: kind.to_string(),
        description: description.to_string(),
        params: vec![],
        returns: None,
        examples: vec![],
        tags: vec![],
        private: false,
        file: file.to_string(),
        line: 1,
        end_line: 1,
        signature: Some(join3("export function ", name, "(): void")),
        extends: vec![],
        implements: vec![],
        has_body: false,
        members: vec![],
        type_parameters: vec![],
    }
}

pub(super) fn link_test_docs() -> Vec<ApiDocModule> {
    vec![
        ApiDocModule {
            description: String::new(),
            file: "/repo/src/context.ts".to_string(),
            source_path: String::new(),
            examples: vec![],
            tags: vec![],
            entries: vec![test_entry(
                "CommandContext",
                "interface",
                "/repo/src/context.ts",
                "Command context.",
            )],
        },
        ApiDocModule {
            description: String::new(),
            file: "/repo/src/command.ts".to_string(),
            source_path: String::new(),
            examples: vec![],
            tags: vec![],
            entries: vec![test_entry(
                "Command",
                "function",
                "/repo/src/command.ts",
                "Runs with [CommandContext].",
            )],
        },
    ]
}

pub(super) fn pure_test_docs() -> Vec<ApiDocModule> {
    vec![ApiDocModule {
        description: String::new(),
        file: "/repo/src/cli.ts".to_string(),
        source_path: String::new(),
        examples: vec![],
        tags: vec![],
        entries: vec![
            ApiDocEntry {
                name: "cli".to_string(),
                kind: "function".to_string(),
                description: "Runs the CLI.".to_string(),
                params: vec![ApiParamDoc {
                    name: "argv".to_string(),
                    type_annotation: "string[]".to_string(),
                    description: "Arguments.".to_string(),
                    optional: false,
                    default_value: None,
                }],
                returns: Some(ApiReturnDoc {
                    type_annotation: "void".to_string(),
                    description: "Nothing.".to_string(),
                    members: Vec::new(),
                }),
                examples: vec!["```ts\ncli([])\n```".to_string()],
                tags: vec![ApiDocTag { tag: "since".to_string(), value: "1.0.0".to_string() }],
                private: false,
                file: "/repo/src/cli.ts".to_string(),
                line: 1,
                end_line: 3,
                signature: Some("export function cli(argv: string[]): void".to_string()),
                extends: vec![],
                implements: vec![],
                has_body: false,
                members: vec![],
                type_parameters: vec![],
            },
            ApiDocEntry {
                name: "Command".to_string(),
                kind: "interface".to_string(),
                description: "A command.".to_string(),
                params: vec![],
                returns: None,
                examples: vec![],
                tags: vec![],
                private: false,
                file: "/repo/src/cli.ts".to_string(),
                line: 5,
                end_line: 8,
                signature: Some("export interface Command".to_string()),
                extends: vec![],
                implements: vec![],
                has_body: false,
                members: vec![ApiDocMember {
                    name: "run".to_string(),
                    kind: "method".to_string(),
                    description: "Runs it.".to_string(),
                    signature: Some("run(): void".to_string()),
                    type_annotation: None,
                    default_value: None,
                    params: vec![],
                    type_parameters: vec![],
                    returns: None,
                    members: vec![],
                    optional: false,
                    readonly: false,
                    r#static: false,
                    private: false,
                    tags: vec![],
                    implementation_of: vec![],
                    line: 6,
                    end_line: 6,
                }],
                type_parameters: vec![],
            },
        ],
    }]
}

pub(super) fn assert_no_api_html(markdown: &str) {
    assert!(!markdown.contains("<details"), "unexpected <details> in:\n{markdown}");
    assert!(!markdown.contains("class=\"ox-api"), "unexpected ox-api html in:\n{markdown}");
    assert!(!markdown.contains("<table"), "unexpected <table> in:\n{markdown}");
    assert!(!markdown.contains("ox-api-controls"), "unexpected controls in:\n{markdown}");
}

/// Asserts heading levels never increase by more than one (markdownlint
/// MD001), ignoring `#` lines inside fenced code blocks.
pub(super) fn assert_no_heading_level_skips(markdown: &str) {
    let mut previous = 0usize;
    let mut in_fence = false;
    for line in markdown.lines() {
        if line.trim_start().starts_with("```") {
            in_fence = !in_fence;
            continue;
        }
        if in_fence {
            continue;
        }
        let hashes = line.chars().take_while(|&ch| ch == '#').count();
        if hashes == 0 || line.as_bytes().get(hashes) != Some(&b' ') {
            continue;
        }
        if previous != 0 {
            assert!(
                hashes <= previous + 1,
                "heading level skip {previous} -> {hashes} at: {line}\nin:\n{markdown}"
            );
        }
        previous = hashes;
    }
}
pub(super) fn typedoc_title_page(entry: ApiDocEntry) -> String {
    let docs = vec![ApiDocModule {
        description: String::new(),
        file: "mod".to_string(),
        source_path: String::new(),
        examples: vec![],
        tags: vec![],
        entries: vec![entry],
    }];
    let out = generate_markdown(
        &docs,
        &MarkdownDocsOptions {
            path_strategy: MarkdownPathStrategy::TypeDoc,
            render_style: MarkdownRenderStyle::Markdown,
            ..MarkdownDocsOptions::default()
        },
    );
    out.into_iter()
        .find(|(key, _)| key.contains('/') && key.ends_with(".md") && !key.ends_with("index.md"))
        .map(|(_, page)| page)
        .expect("a per-symbol page")
}

pub(super) fn lifecycle_module(entry: ApiDocEntry) -> Vec<ApiDocModule> {
    vec![ApiDocModule {
        description: String::new(),
        file: "combinators".to_string(),
        source_path: String::new(),
        examples: vec![],
        tags: vec![],
        entries: vec![entry],
    }]
}

pub(super) fn markdown_typedoc_options() -> MarkdownDocsOptions {
    MarkdownDocsOptions {
        path_strategy: MarkdownPathStrategy::TypeDoc,
        render_style: MarkdownRenderStyle::Markdown,
        ..MarkdownDocsOptions::default()
    }
}

pub(super) fn object_literal_parameter_entry() -> ApiDocEntry {
    let mut entry = test_entry("plugin", "function", "/repo/src/plugin.ts", "Define a plugin.");
    entry.params = vec![
        ApiParamDoc {
            name: "options".to_string(),
            type_annotation:
                "{ id: Id; name?: string; setup?: (ctx: PluginContext) => Awaitable<void> }"
                    .to_string(),
            description: "Plugin options.".to_string(),
            optional: false,
            default_value: None,
        },
        ApiParamDoc {
            name: "options.id".to_string(),
            type_annotation: "Id".to_string(),
            description: "Plugin id.".to_string(),
            optional: false,
            default_value: None,
        },
        ApiParamDoc {
            name: "options.name?".to_string(),
            type_annotation: "string".to_string(),
            description: String::new(),
            optional: true,
            default_value: None,
        },
        ApiParamDoc {
            name: "options.setup?".to_string(),
            type_annotation: "(ctx: PluginContext) => Awaitable<void>".to_string(),
            description: "Setup hook.".to_string(),
            optional: true,
            default_value: None,
        },
    ];
    entry
}

pub(super) fn module_with_source_path(source_path: &str) -> Vec<ApiDocModule> {
    vec![ApiDocModule {
        description: String::new(),
        // `file` is the module route name, not a real path.
        file: "default".to_string(),
        source_path: source_path.to_string(),
        examples: vec![],
        tags: vec![],
        entries: vec![test_entry("cli", "function", "/repo/packages/x/src/cli.ts", "Run.")],
    }]
}

pub(super) fn group_order_docs() -> Vec<ApiDocModule> {
    vec![ApiDocModule {
        description: "Module description.".to_string(),
        file: "default".to_string(),
        source_path: String::new(),
        examples: vec![],
        tags: vec![],
        entries: vec![
            test_entry("alpha", "function", "/repo/src/a.ts", "A function."),
            test_entry("Config", "interface", "/repo/src/c.ts", "An interface."),
            test_entry("Engine", "class", "/repo/src/e.ts", "A class."),
            test_entry("VERSION", "variable", "/repo/src/v.ts", "A variable."),
        ],
    }]
}

pub(super) fn stats_docs() -> Vec<ApiDocModule> {
    vec![ApiDocModule {
        description: "Module description.".to_string(),
        file: "default".to_string(),
        source_path: String::new(),
        examples: vec![],
        tags: vec![],
        entries: vec![
            test_entry("cli", "function", "/repo/src/cli.ts", "Run the CLI."),
            test_entry("run", "function", "/repo/src/run.ts", "Run again."),
        ],
    }]
}

pub(super) fn overload_entry(
    name: &str,
    file: &str,
    description: &str,
    signature: &str,
    has_body: bool,
) -> ApiDocEntry {
    ApiDocEntry {
        name: name.to_string(),
        kind: "function".to_string(),
        description: description.to_string(),
        params: vec![],
        returns: None,
        examples: vec![],
        tags: vec![],
        private: false,
        file: file.to_string(),
        line: 1,
        end_line: 1,
        signature: Some(signature.to_string()),
        extends: vec![],
        implements: vec![],
        has_body,
        members: vec![],
        type_parameters: vec![],
    }
}

pub(super) fn overload_module(entries: Vec<ApiDocEntry>) -> Vec<ApiDocModule> {
    vec![ApiDocModule {
        description: String::new(),
        file: "default".to_string(),
        source_path: String::new(),
        examples: vec![],
        tags: vec![],
        entries,
    }]
}

pub(super) fn html_typedoc_options() -> MarkdownDocsOptions {
    MarkdownDocsOptions {
        path_strategy: MarkdownPathStrategy::TypeDoc,
        render_style: MarkdownRenderStyle::Html,
        ..MarkdownDocsOptions::default()
    }
}

pub(super) fn member(name: &str, kind: &str, is_static: bool) -> ApiDocMember {
    ApiDocMember {
        name: name.to_string(),
        kind: kind.to_string(),
        description: String::new(),
        signature: None,
        type_annotation: Some("unknown".to_string()),
        default_value: None,
        params: vec![],
        type_parameters: vec![],
        returns: None,
        members: vec![],
        optional: false,
        readonly: false,
        r#static: is_static,
        private: false,
        tags: vec![],
        implementation_of: vec![],
        line: 1,
        end_line: 1,
    }
}

pub(super) fn function_valued_parse_member() -> ApiDocMember {
    ApiDocMember {
        name: "parse".to_string(),
        kind: "property".to_string(),
        description: "Parses a raw value.".to_string(),
        signature: None,
        type_annotation: Some("(value: string) => string | undefined".to_string()),
        default_value: None,
        params: vec![ApiParamDoc {
            name: "value".to_string(),
            type_annotation: "string".to_string(),
            description: "Raw string value from command line.".to_string(),
            optional: false,
            default_value: None,
        }],
        type_parameters: vec![],
        returns: Some(ApiReturnDoc {
            type_annotation: "string | undefined".to_string(),
            description: "Parsed value.".to_string(),
            members: Vec::new(),
        }),
        members: vec![],
        optional: true,
        readonly: false,
        r#static: false,
        private: false,
        tags: vec![],
        implementation_of: vec![],
        line: 5,
        end_line: 10,
    }
}

pub(super) fn type_param(name: &str) -> ApiParamDoc {
    ApiParamDoc {
        name: name.to_string(),
        type_annotation: String::new(),
        description: String::new(),
        optional: false,
        default_value: None,
    }
}

/// A parameter with a name and a type annotation (no description/flags).
pub(super) fn param(name: &str, type_annotation: &str) -> ApiParamDoc {
    ApiParamDoc { type_annotation: type_annotation.to_string(), ..type_param(name) }
}

/// A `type` entry stub so its name resolves in the symbol map (for type links).
pub(super) fn type_stub(name: &str) -> ApiDocEntry {
    let mut entry = test_entry(name, "type", "/repo/src/types.ts", "");
    entry.signature = None;
    entry
}

/// A `function` entry stub (e.g. a combinator) so its name resolves in the
/// symbol map even when it collides with a primitive type name.
pub(super) fn function_stub(name: &str) -> ApiDocEntry {
    test_entry(name, "function", "/repo/src/combinators.ts", "")
}

/// A module containing `entry` plus stub `type` entries whose names are used as
/// linkable symbols inside type annotations in the type-link tests.
pub(super) fn type_link_module(entry: ApiDocEntry) -> Vec<ApiDocModule> {
    vec![ApiDocModule {
        description: String::new(),
        file: "combinators".to_string(),
        source_path: String::new(),
        examples: vec![],
        tags: vec![],
        entries: vec![
            entry,
            type_stub("RenderingOptions"),
            type_stub("SubCommandable"),
            type_stub("CommandRunner"),
            type_stub("GunshiParamsConstraint"),
            type_stub("DefaultGunshiParams"),
            type_stub("PluginExtension"),
            type_stub("ArgValues"),
            type_stub("ArgExplicitlyProvided"),
            type_stub("U"),
            // Symbols that collide with TypeScript intrinsic primitive types,
            // mirroring gunshi's `string()` / `boolean()` / `number()`
            // combinators. These must never be linked inside a type annotation.
            function_stub("string"),
            function_stub("boolean"),
            function_stub("number"),
        ],
    }]
}

pub(super) fn index_signature_docs() -> Vec<ApiDocModule> {
    let mut schema = test_entry("ArgSchema", "interface", "/repo/src/args.ts", "Value type.");
    schema.signature = Some("export interface ArgSchema".to_string());

    let mut args = test_entry("Args", "interface", "/repo/src/args.ts", "Arguments.");
    args.signature = Some("export interface Args".to_string());
    args.members =
        vec![index_signature_member("[option: string]", "option", "string", "ArgSchema", true)];

    vec![ApiDocModule {
        description: String::new(),
        file: "default".to_string(),
        source_path: String::new(),
        examples: vec![],
        tags: vec![],
        entries: vec![schema, args],
    }]
}

pub(super) fn multiline_plugin_ext_type_parameters() -> Vec<ApiTypeParamDoc> {
    vec![
        ApiTypeParamDoc {
            name: "Extension".to_string(),
            constraint: None,
            default: None,
            description: String::new(),
        },
        ApiTypeParamDoc {
            name: "ResolvedDepExtensions".to_string(),
            constraint: None,
            default: None,
            description: String::new(),
        },
        ApiTypeParamDoc {
            name: "PluginExt".to_string(),
            constraint: Some("PluginExtension<Extension, DefaultGunshiParams>".to_string()),
            default: Some(
                "PluginExtension<\n    Extension,\n    ResolvedDepExtensions\n  >".to_string(),
            ),
            description: String::new(),
        },
    ]
}

pub(super) fn return_property(name: &str, type_annotation: &str) -> ApiDocMember {
    ApiDocMember {
        name: name.to_string(),
        kind: "property".to_string(),
        description: String::new(),
        signature: None,
        type_annotation: Some(type_annotation.to_string()),
        default_value: None,
        params: vec![],
        type_parameters: vec![],
        returns: None,
        members: vec![],
        optional: false,
        readonly: false,
        r#static: false,
        private: false,
        tags: vec![],
        implementation_of: vec![],
        line: 1,
        end_line: 1,
    }
}

pub(super) fn index_signature_member(
    name: &str,
    param_name: &str,
    param_type: &str,
    value_type: &str,
    readonly: bool,
) -> ApiDocMember {
    ApiDocMember {
        name: name.to_string(),
        kind: "indexSignature".to_string(),
        description: "Argument schema by option name.".to_string(),
        signature: Some(if readonly {
            format!("readonly {name}: {value_type}")
        } else {
            format!("{name}: {value_type}")
        }),
        type_annotation: Some(value_type.to_string()),
        default_value: None,
        params: vec![param(param_name, param_type)],
        type_parameters: vec![],
        returns: None,
        members: vec![],
        optional: false,
        readonly,
        r#static: false,
        private: false,
        tags: vec![],
        implementation_of: vec![],
        line: 1,
        end_line: 1,
    }
}
