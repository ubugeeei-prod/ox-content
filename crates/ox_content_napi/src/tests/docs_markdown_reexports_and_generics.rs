use super::*;

#[test]
fn generate_docs_markdown_dedupes_cross_entrypoint_reexports() {
    // The same symbol re-exported from two entry points should yield a single
    // canonical page placed under its defining module via `sourcePath`.
    let entry = |name: &str| JsDocsMarkdownEntry {
        name: name.to_string(),
        kind: "function".to_string(),
        description: "Creates a command context.".to_string(),
        params: None,
        returns: None,
        examples: None,
        tags: None,
        private: false,
        file: "/repo/src/context.ts".to_string(),
        line: 1,
        end_line: 1,
        signature: Some("export function createCommandContext(): void".to_string()),
        extends: None,
        implements: None,
        has_body: None,
        members: None,
        type_parameters: None,
    };
    let docs = vec![
        JsDocsMarkdownModule {
            description: None,
            file: "context".to_string(),
            source_path: Some("/repo/src/context.ts".to_string()),
            examples: None,
            tags: None,
            entries: vec![entry("createCommandContext")],
        },
        JsDocsMarkdownModule {
            description: None,
            file: "default".to_string(),
            source_path: Some("/repo/src/index.ts".to_string()),
            examples: None,
            tags: None,
            entries: vec![entry("createCommandContext")],
        },
    ];

    let markdown = generate_docs_markdown(
        docs,
        Some(JsDocsMarkdownOptions {
            group_by: Some("file".to_string()),
            github_url: None,
            link_style: Some("markdown".to_string()),
            base_path: None,
            path_strategy: Some("typedoc".to_string()),
            render_style: None,
            ..Default::default()
        }),
    );

    assert!(markdown.contains_key("context/functions/createCommandContext.md"));
    assert!(!markdown.contains_key("default/functions/createCommandContext.md"));
    assert!(markdown.get("default/index.md").unwrap().contains("Re-exports"));
}

#[test]
fn generate_docs_markdown_renders_type_parameters() {
    let docs = vec![JsDocsMarkdownModule {
        description: None,
        file: "default".to_string(),
        source_path: None,
        examples: None,
        tags: None,
        entries: vec![JsDocsMarkdownEntry {
            name: "make".to_string(),
            kind: "function".to_string(),
            description: "Make a thing.".to_string(),
            params: None,
            returns: None,
            examples: None,
            tags: None,
            private: false,
            file: "/repo/src/make.ts".to_string(),
            line: 1,
            end_line: 1,
            signature: Some("export function make<G>(): G".to_string()),
            extends: None,
            implements: None,
            has_body: None,
            members: None,
            type_parameters: Some(vec![JsTypeParam {
                name: "G".to_string(),
                constraint: Some("Base".to_string()),
                r#default: Some("Default".to_string()),
                description: "The thing type.".to_string(),
            }]),
        }],
    }];

    let markdown = generate_docs_markdown(
        docs,
        Some(JsDocsMarkdownOptions {
            group_by: Some("file".to_string()),
            github_url: None,
            link_style: Some("markdown".to_string()),
            base_path: None,
            path_strategy: Some("typedoc".to_string()),
            render_style: Some("markdown".to_string()),
            ..Default::default()
        }),
    );
    let page = markdown.get("default/functions/make.md").unwrap();

    assert!(page.contains("## Type Parameters"));
    assert!(!page.contains("**Type Parameters**"));
    assert!(page.contains("`G` *extends* `Base` = `Default`"));
    assert!(page.contains("The thing type."));
}

#[test]
fn generate_docs_markdown_collapses_multiline_linked_type_parameter_defaults() {
    fn entry(name: &str, kind: &str, signature: &str) -> JsDocsMarkdownEntry {
        JsDocsMarkdownEntry {
            name: name.to_string(),
            kind: kind.to_string(),
            description: String::new(),
            params: None,
            returns: None,
            examples: None,
            tags: None,
            private: false,
            file: format!("/repo/src/{name}.ts"),
            line: 1,
            end_line: 1,
            signature: Some(signature.to_string()),
            extends: None,
            implements: None,
            has_body: None,
            members: None,
            type_parameters: None,
        }
    }

    let mut plugin = entry("plugin", "function", "export function plugin(): void");
    plugin.type_parameters = Some(vec![
        JsTypeParam {
            name: "Extension".to_string(),
            constraint: None,
            r#default: None,
            description: String::new(),
        },
        JsTypeParam {
            name: "ResolvedDepExtensions".to_string(),
            constraint: None,
            r#default: None,
            description: String::new(),
        },
        JsTypeParam {
            name: "PluginExt".to_string(),
            constraint: Some("PluginExtension<Extension, DefaultGunshiParams>".to_string()),
            r#default: Some(
                "PluginExtension<\n    Extension,\n    ResolvedDepExtensions\n  >".to_string(),
            ),
            description: String::new(),
        },
    ]);

    let docs = vec![JsDocsMarkdownModule {
        description: None,
        file: "default".to_string(),
        source_path: None,
        examples: None,
        tags: None,
        entries: vec![
            plugin,
            entry("PluginExtension", "type", "export type PluginExtension = unknown"),
            entry("DefaultGunshiParams", "type", "export type DefaultGunshiParams = unknown"),
        ],
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
    let page = markdown.get("default/functions/plugin.md").unwrap();

    assert!(page.contains("| Name |\n| --- |"));
    assert!(!page.contains("| Name | Description |"));
    assert!(page.contains("| `PluginExt` *extends* [`PluginExtension`](../type-aliases/PluginExtension.md)\\<`Extension`, [`DefaultGunshiParams`](../type-aliases/DefaultGunshiParams.md)\\> = [`PluginExtension`](../type-aliases/PluginExtension.md)\\<`Extension`, `ResolvedDepExtensions`\\> |"));
    assert!(!page.contains("\\<\n"));
    assert!(!page.contains("ResolvedDepExtensions`\n"));
}
