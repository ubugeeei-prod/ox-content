use super::*;

#[test]
fn generate_docs_markdown_dedupes_cross_entrypoint_reexports() {
    // The same symbol re-exported from two entry points should yield a single
    // canonical page placed under its defining module via `sourcePath`.
    let entry = |name: &str| JsDocsMarkdownEntry {
        name: name.to_string(),
        kind: "function".to_string(),
        description: "Creates a command context.".to_string(),
        file: "/repo/src/context.ts".to_string(),
        signature: Some("export function createCommandContext(): void".to_string()),
        ..Default::default()
    };
    let docs = vec![
        JsDocsMarkdownModule {
            file: "context".to_string(),
            source_path: Some("/repo/src/context.ts".to_string()),
            entries: vec![entry("createCommandContext")],
            ..Default::default()
        },
        JsDocsMarkdownModule {
            file: "default".to_string(),
            source_path: Some("/repo/src/index.ts".to_string()),
            entries: vec![entry("createCommandContext")],
            ..Default::default()
        },
    ];

    let markdown = generate_docs_markdown(
        docs,
        Some(JsDocsMarkdownOptions {
            group_by: Some("file".to_string()),
            link_style: Some("markdown".to_string()),
            path_strategy: Some("typedoc".to_string()),
            ..Default::default()
        }),
    );
    assert_string_map_snapshot(
        "generate_docs_markdown_dedupes_cross_entrypoint_reexports",
        &markdown,
    );

    assert!(markdown.contains_key("context/functions/createCommandContext.md"));
    assert!(!markdown.contains_key("default/functions/createCommandContext.md"));
}

#[test]
fn generate_docs_markdown_renders_type_parameters() {
    let docs = vec![JsDocsMarkdownModule {
        file: "default".to_string(),
        entries: vec![JsDocsMarkdownEntry {
            name: "make".to_string(),
            kind: "function".to_string(),
            description: "Make a thing.".to_string(),
            file: "/repo/src/make.ts".to_string(),
            signature: Some("export function make<G>(): G".to_string()),
            type_parameters: Some(vec![JsTypeParam {
                name: "G".to_string(),
                constraint: Some("Base".to_string()),
                r#default: Some("Default".to_string()),
                description: "The thing type.".to_string(),
            }]),
            ..Default::default()
        }],
        ..Default::default()
    }];

    let markdown = generate_docs_markdown(
        docs,
        Some(JsDocsMarkdownOptions {
            group_by: Some("file".to_string()),
            link_style: Some("markdown".to_string()),
            path_strategy: Some("typedoc".to_string()),
            render_style: Some("markdown".to_string()),
            ..Default::default()
        }),
    );
    assert_string_map_snapshot("generate_docs_markdown_renders_type_parameters", &markdown);
}

#[test]
fn generate_docs_markdown_collapses_multiline_linked_type_parameter_defaults() {
    fn entry(name: &str, kind: &str, signature: &str) -> JsDocsMarkdownEntry {
        JsDocsMarkdownEntry {
            name: name.to_string(),
            kind: kind.to_string(),
            file: format!("/repo/src/{name}.ts"),
            signature: Some(signature.to_string()),
            ..Default::default()
        }
    }

    let mut plugin = entry("plugin", "function", "export function plugin(): void");
    plugin.type_parameters = Some(vec![
        JsTypeParam { name: "Extension".to_string(), ..Default::default() },
        JsTypeParam { name: "ResolvedDepExtensions".to_string(), ..Default::default() },
        JsTypeParam {
            name: "PluginExt".to_string(),
            constraint: Some("PluginExtension<Extension, DefaultGunshiParams>".to_string()),
            r#default: Some(
                "PluginExtension<\n    Extension,\n    ResolvedDepExtensions\n  >".to_string(),
            ),
            ..Default::default()
        },
    ]);

    let docs = vec![JsDocsMarkdownModule {
        file: "default".to_string(),
        entries: vec![
            plugin,
            entry("PluginExtension", "type", "export type PluginExtension = unknown"),
            entry("DefaultGunshiParams", "type", "export type DefaultGunshiParams = unknown"),
        ],
        ..Default::default()
    }];

    let markdown = generate_docs_markdown(
        docs,
        Some(JsDocsMarkdownOptions {
            group_by: Some("file".to_string()),
            link_style: Some("markdown".to_string()),
            path_strategy: Some("typedoc".to_string()),
            render_style: Some("markdown".to_string()),
            parameters_format: Some("table".to_string()),
            ..Default::default()
        }),
    );
    assert_string_map_snapshot(
        "generate_docs_markdown_collapses_multiline_linked_type_parameter_defaults",
        &markdown,
    );
}
