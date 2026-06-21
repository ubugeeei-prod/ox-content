use super::*;

#[test]
fn render_style_markdown_flat_emits_pure_markdown() {
    let options = MarkdownDocsOptions {
        render_style: MarkdownRenderStyle::Markdown,
        github_url: Some("https://github.com/x/y".to_string()),
        ..MarkdownDocsOptions::default()
    };
    let out = generate_markdown(&pure_test_docs(), &options);
    assert_markdown_map_snapshot("render_style_markdown_flat_emits_pure_markdown", &out);

    let page = out.get("cli.md").unwrap();
    assert_no_api_html(page);

    // Flat entry heading is H3, so body sections render at H4.
    assert!(page.lines().any(|line| line == "#### Signature"));

    // The callable interface member group renders as real detail headings.
    assert!(page.lines().any(|line| line == "#### Methods"));
    assert!(page.lines().any(|line| line == "##### run()"));

    let index = out.get("index.md").unwrap();
    assert_no_api_html(index);
}

#[test]
fn render_style_markdown_typedoc_emits_pure_per_symbol_pages() {
    let options = MarkdownDocsOptions {
        render_style: MarkdownRenderStyle::Markdown,
        path_strategy: MarkdownPathStrategy::TypeDoc,
        base_path: Some("/api".to_string()),
        ..MarkdownDocsOptions::default()
    };
    let out = generate_markdown(&pure_test_docs(), &options);
    assert_markdown_map_snapshot("render_style_markdown_typedoc_emits_pure_per_symbol_pages", &out);

    let key = out
        .keys()
        .find(|key| key.ends_with("functions/cli.md"))
        .expect("typedoc cli page should exist")
        .clone();
    let page = out.get(&key).unwrap();
    assert_no_api_html(page);
    assert!(page.starts_with("# Function: cli()"));
}

#[test]
fn render_style_markdown_category_emits_pure_markdown() {
    let options = MarkdownDocsOptions {
        render_style: MarkdownRenderStyle::Markdown,
        group_by: "category".to_string(),
        ..MarkdownDocsOptions::default()
    };
    let out = generate_markdown(&pure_test_docs(), &options);
    assert_markdown_map_snapshot("render_style_markdown_category_emits_pure_markdown", &out);

    let functions = out.get("functions.md").unwrap();
    assert_no_api_html(functions);
}

#[test]
fn typedoc_symbol_page_h1_includes_declaration_kind() {
    // Function: kind prefix + `()`, no type parameters in the title.
    let mut func = test_entry("args", "function", "/repo/src/combinators.ts", "Schema factory.");
    func.type_parameters =
        vec![ApiTypeParamDoc { name: "T".to_string(), ..ApiTypeParamDoc::default() }];
    assert!(typedoc_title_page(func).starts_with("# Function: args()"));

    // Interface with a generic parameter (names only).
    let mut iface = test_entry("Command", "interface", "/repo/src/types.ts", "A command.");
    iface.type_parameters = vec![ApiTypeParamDoc {
        name: "G".to_string(),
        constraint: Some("GunshiParams".to_string()),
        ..ApiTypeParamDoc::default()
    }];
    assert!(typedoc_title_page(iface).starts_with("# Interface: Command<G>"));

    // Type alias with a generic parameter.
    let mut alias = test_entry("Plugin", "type", "/repo/src/plugin.ts", "Plugin type.");
    alias.type_parameters =
        vec![ApiTypeParamDoc { name: "E".to_string(), ..ApiTypeParamDoc::default() }];
    assert!(typedoc_title_page(alias).starts_with("# Type Alias: Plugin<E>"));

    // Class without type parameters: kind prefix only.
    let class = test_entry("DefaultTranslation", "class", "/repo/src/i18n.ts", "Translation.");
    assert!(typedoc_title_page(class).starts_with("# Class: DefaultTranslation\n"));

    // Variable: kind prefix only, no `()` or `<>`.
    let variable =
        test_entry("CLI_OPTIONS_DEFAULT", "variable", "/repo/src/constants.ts", "Defaults.");
    assert!(typedoc_title_page(variable).starts_with("# Variable: CLI_OPTIONS_DEFAULT\n"));
}

#[test]
fn render_style_markdown_typedoc_sections_are_sequential_headings() {
    let options = MarkdownDocsOptions {
        render_style: MarkdownRenderStyle::Markdown,
        path_strategy: MarkdownPathStrategy::TypeDoc,
        github_url: Some("https://github.com/x/y".to_string()),
        ..MarkdownDocsOptions::default()
    };
    let out = generate_markdown(&pure_test_docs(), &options);
    assert_markdown_map_snapshot(
        "render_style_markdown_typedoc_sections_are_sequential_headings",
        &out,
    );
    assert_markdown_map_snapshot(
        "render_style_markdown_typedoc_sections_are_sequential_headings",
        &out,
    );

    // Function page: every section is a real H2 heading under the H1 title,
    // with no bold-as-header, no skipped levels.
    let fn_key =
        out.keys().find(|key| key.ends_with("functions/cli.md")).expect("cli page").clone();
    let page = out.get(&fn_key).unwrap();
    assert!(page.starts_with("# Function: cli()"));

    // `@since` renders as a dedicated `## Since` section, not generic `## Tags`.

    assert_no_heading_level_skips(page);

    // Returns is its own heading with the value on the following line.
    let after_returns = page.split("## Returns\n\n").nth(1).expect("returns section");
    assert!(after_returns.starts_with("`void`"), "returns value on next line:\n{page}");

    // Interface page: member group is a real H2 heading (## Methods), not a
    // `#### Properties`/`**Members**` mix.
    let if_key = out
        .keys()
        .find(|key| key.ends_with("interfaces/Command.md"))
        .expect("Command page")
        .clone();
    let page = out.get(&if_key).unwrap();

    assert_no_heading_level_skips(page);
}

#[test]
fn typedoc_markdown_return_union_pipe_is_not_escaped_inside_inline_code() {
    let mut entry = test_entry("cli", "function", "/repo/src/cli.ts", "Run the command.");
    entry.signature = Some(
        "export function cli(entry: Command<G> | CommandRunner<G>): Promise<string | undefined>"
            .to_string(),
    );
    entry.params = vec![param("entry", "Command<G> | CommandRunner<G>")];
    entry.returns = Some(ApiReturnDoc {
        type_annotation: "Promise<string | undefined>".to_string(),
        description: "A rendered usage or undefined.".to_string(),
        ..ApiReturnDoc::default()
    });
    let docs = vec![ApiDocModule {
        file: "default".to_string(),
        entries: vec![entry],
        ..ApiDocModule::default()
    }];

    let out = generate_markdown(
        &docs,
        &MarkdownDocsOptions {
            parameters_format: MarkdownDisplayFormat::Table,
            ..markdown_typedoc_options()
        },
    );
    assert_markdown_map_snapshot(
        "typedoc_markdown_return_union_pipe_is_not_escaped_inside_inline_code",
        &out,
    );
    assert_markdown_map_snapshot(
        "typedoc_markdown_return_union_pipe_is_not_escaped_inside_inline_code",
        &out,
    );
}

#[test]
fn render_style_markdown_flat_sections_render_at_h4() {
    let options = MarkdownDocsOptions {
        render_style: MarkdownRenderStyle::Markdown,
        ..MarkdownDocsOptions::default()
    };
    let out = generate_markdown(&pure_test_docs(), &options);
    assert_markdown_map_snapshot("render_style_markdown_flat_sections_render_at_h4", &out);
    let page = out.get("cli.md").unwrap();

    // Flat entry heading is H3, so its sections render at H4 (sequential).

    assert!(page.lines().any(|line| line == "#### Signature"));
    assert!(page.lines().any(|line| line == "#### Parameters"));
    assert!(page.lines().any(|line| line == "#### Returns"));
    assert!(page.lines().any(|line| line == "#### Throws"));
    assert!(!page.lines().any(|line| line == "## Signature"));
    assert_no_heading_level_skips(page);
}

#[test]
fn render_style_html_renders_throws_section() {
    let out = generate_markdown(&pure_test_docs(), &MarkdownDocsOptions::default());
    assert_markdown_map_snapshot("render_style_html_renders_throws_section", &out);
}
