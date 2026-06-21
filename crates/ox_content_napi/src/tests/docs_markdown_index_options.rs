use super::*;

#[test]
fn generate_docs_markdown_group_order_reorders_module_index() {
    fn entry(name: &str, kind: &str) -> JsDocsMarkdownEntry {
        JsDocsMarkdownEntry {
            name: name.to_string(),
            kind: kind.to_string(),
            file: format!("/repo/src/{name}.ts"),
            signature: Some(format!("export declare const {name}: unknown")),
            ..Default::default()
        }
    }
    let docs = vec![JsDocsMarkdownModule {
        file: "default".to_string(),
        entries: vec![entry("alpha", "function"), entry("VERSION", "variable")],
        ..Default::default()
    }];

    let markdown = generate_docs_markdown(
        docs,
        Some(JsDocsMarkdownOptions {
            path_strategy: Some("typedoc".to_string()),
            render_style: Some("markdown".to_string()),
            group_order: Some(vec!["Variables".to_string(), "Functions".to_string()]),
            ..Default::default()
        }),
    );
    assert_string_map_snapshot(
        "generate_docs_markdown_group_order_reorders_module_index",
        &markdown,
    );
    let index = markdown.get("default/index.md").unwrap();

    assert!(index.find("## Variables").unwrap() < index.find("## Functions").unwrap());
}

fn docs_markdown_module() -> Vec<JsDocsMarkdownModule> {
    vec![JsDocsMarkdownModule {
        file: "default".to_string(),
        entries: vec![JsDocsMarkdownEntry {
            name: "cli".to_string(),
            kind: "function".to_string(),
            description: "Run.".to_string(),
            file: "/repo/src/cli.ts".to_string(),
            signature: Some("export function cli(): void".to_string()),
            ..Default::default()
        }],
        ..Default::default()
    }]
}

#[test]
fn generate_docs_markdown_render_stats_option_toggles_stats_summary() {
    fn options(render_stats: Option<bool>) -> JsDocsMarkdownOptions {
        JsDocsMarkdownOptions {
            path_strategy: Some("typedoc".to_string()),
            render_style: Some("markdown".to_string()),
            render_stats,
            ..Default::default()
        }
    }

    // Default (None -> true) keeps the stats summary.
    let with_stats = generate_docs_markdown(docs_markdown_module(), Some(options(None)));
    assert_string_map_snapshot(
        "generate_docs_markdown_render_stats_option_toggles_stats_summary__with_stats",
        &with_stats,
    );

    // Explicit false omits it.
    let without_stats = generate_docs_markdown(docs_markdown_module(), Some(options(Some(false))));
    assert_string_map_snapshot(
        "generate_docs_markdown_render_stats_option_toggles_stats_summary__without_stats",
        &without_stats,
    );
}

#[test]
fn generate_docs_markdown_render_generated_by_option_toggles_attribution() {
    fn options(render_generated_by: Option<bool>) -> JsDocsMarkdownOptions {
        JsDocsMarkdownOptions {
            path_strategy: Some("typedoc".to_string()),
            render_style: Some("markdown".to_string()),
            render_stats: Some(false),
            render_generated_by,
            ..Default::default()
        }
    }

    // Default (None -> true) keeps the generated-by attribution.
    let with_generated_by = generate_docs_markdown(docs_markdown_module(), Some(options(None)));
    assert_string_map_snapshot(
        "generate_docs_markdown_render_generated_by_option_toggles_attribution__with_generated_by",
        &with_generated_by,
    );

    // Explicit false omits it and leaves the Modules heading directly after
    // the H1 when stats are also disabled.
    let without_generated_by =
        generate_docs_markdown(docs_markdown_module(), Some(options(Some(false))));
    assert_string_map_snapshot("generate_docs_markdown_render_generated_by_option_toggles_attribution__without_generated_by", &without_generated_by);
    let root = without_generated_by.get("index.md").unwrap();

    assert!(root.starts_with("# API Documentation\n\n## Modules\n\n"));
}
