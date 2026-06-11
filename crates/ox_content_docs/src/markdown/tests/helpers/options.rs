use super::super::*;
use super::fixtures::test_entry;

pub(in crate::markdown::tests) fn typedoc_title_page(entry: ApiDocEntry) -> String {
    let docs = vec![ApiDocModule {
        file: "mod".to_string(),
        entries: vec![entry],
        ..ApiDocModule::default()
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

pub(in crate::markdown::tests) fn markdown_typedoc_options() -> MarkdownDocsOptions {
    MarkdownDocsOptions {
        path_strategy: MarkdownPathStrategy::TypeDoc,
        render_style: MarkdownRenderStyle::Markdown,
        ..MarkdownDocsOptions::default()
    }
}

pub(in crate::markdown::tests) fn html_typedoc_options() -> MarkdownDocsOptions {
    MarkdownDocsOptions {
        path_strategy: MarkdownPathStrategy::TypeDoc,
        render_style: MarkdownRenderStyle::Html,
        ..MarkdownDocsOptions::default()
    }
}

pub(in crate::markdown::tests) fn module_with_source_path(source_path: &str) -> Vec<ApiDocModule> {
    vec![ApiDocModule {
        // `file` is the module route name, not a real path.
        file: "default".to_string(),
        source_path: source_path.to_string(),
        entries: vec![test_entry("cli", "function", "/repo/packages/x/src/cli.ts", "Run.")],
        ..ApiDocModule::default()
    }]
}

pub(in crate::markdown::tests) fn overload_entry(
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
        file: file.to_string(),
        signature: Some(signature.to_string()),
        has_body,
        ..ApiDocEntry::default()
    }
}

pub(in crate::markdown::tests) fn overload_module(entries: Vec<ApiDocEntry>) -> Vec<ApiDocModule> {
    vec![ApiDocModule { file: "default".to_string(), entries, ..ApiDocModule::default() }]
}
