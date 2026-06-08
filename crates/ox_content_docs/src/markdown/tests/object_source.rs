use super::*;

#[test]
fn typedoc_markdown_renders_object_literal_parameter_members() {
    let options = MarkdownDocsOptions {
        parameters_format: MarkdownDisplayFormat::Table,
        ..markdown_typedoc_options()
    };
    let out = generate_markdown(&type_link_module(object_literal_parameter_entry()), &options);
    let page = out.get("combinators/functions/plugin.md").unwrap();

    assert!(!page.contains("{ ... }"));
    assert!(page.contains("| `options` | `{ id: Id; name?: string; setup?: (ctx: PluginContext) => Awaitable<void> }` | Plugin options. |"));
    assert!(page.contains("| `options.id` | `Id` | Plugin id. |"));
    assert!(page.contains("| `options.name?` | `string` | _optional_ |"));
    assert!(page.contains(
            "| `options.setup?` | `(ctx: PluginContext) => Awaitable<void>` | Setup hook. _(optional)_ |"
        ));
}

#[test]
fn typedoc_html_renders_object_literal_parameter_members() {
    let options = MarkdownDocsOptions {
        parameters_format: MarkdownDisplayFormat::Table,
        ..html_typedoc_options()
    };
    let out = generate_markdown(&type_link_module(object_literal_parameter_entry()), &options);
    let page = out.get("combinators/functions/plugin.md").unwrap();

    assert!(!page.contains("{ ... }"));
    assert!(page.contains(
        "{ id: Id; name?: string; setup?: (ctx: PluginContext) =&gt; Awaitable&lt;void&gt; }"
    ));
    assert!(page.contains("options.id"));
    assert!(page.contains("Plugin id."));
    assert!(page.contains("options.setup?"));
    assert!(page.contains("Setup hook."));
}

#[test]
fn typedoc_module_index_source_link_uses_source_path() {
    let options = MarkdownDocsOptions {
        github_url: Some("https://github.com/x/y".to_string()),
        ..markdown_typedoc_options()
    };
    let out =
        generate_markdown(&module_with_source_path("/repo/packages/x/src/index.ts"), &options);
    let index = out.get("default/index.md").unwrap();

    // Links to the real entry-point file, never the dead `blob/main/default`.
    assert!(
        index.contains("**[Source](https://github.com/x/y/blob/main/packages/x/src/index.ts)**")
    );
    assert!(!index.contains("blob/main/default"));
}

#[test]
fn typedoc_module_index_omits_source_link_without_source_path() {
    let options = MarkdownDocsOptions {
        github_url: Some("https://github.com/x/y".to_string()),
        ..markdown_typedoc_options()
    };
    let out = generate_markdown(&module_with_source_path(""), &options);
    let index = out.get("default/index.md").unwrap();

    // No source path -> no module source line (and no dead link).
    assert!(!index.contains("**[Source]"));
    assert!(!index.contains("blob/main/default"));
}
