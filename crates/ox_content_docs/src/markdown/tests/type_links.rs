use super::*;

#[test]
fn typedoc_links_known_symbols_in_param_types() {
    let mut entry = test_entry("make", "function", "/repo/src/make.ts", "Make.");
    entry.params = vec![param("options", "RenderingOptions<G>"), param("count", "number")];
    let options = MarkdownDocsOptions {
        parameters_format: MarkdownDisplayFormat::Table,
        ..markdown_typedoc_options()
    };
    let out = generate_markdown(&type_link_module(entry), &options);
    let page = out.get("combinators/functions/make.md").unwrap();

    // Known symbol links (label is the symbol in inline code), `.md` link style.
    assert!(page.contains("[`RenderingOptions`]("));
    assert!(page.contains("RenderingOptions.md"));
    // Generic arg `G` (a type parameter) and primitive `number` stay plain code.
    assert!(page.contains("`G`"));
    assert!(!page.contains("[`G`]"));
    assert!(page.contains("`number`"));
    assert!(!page.contains("[`number`]"));
}

#[test]
fn typedoc_links_known_symbol_in_return_type() {
    let mut entry = test_entry("make", "function", "/repo/src/make.ts", "Make.");
    entry.returns = Some(ApiReturnDoc {
        type_annotation: "CommandRunner<G>".to_string(),
        description: String::new(),
        members: Vec::new(),
    });
    let out = generate_markdown(&type_link_module(entry), &markdown_typedoc_options());
    let page = out.get("combinators/functions/make.md").unwrap();

    assert!(page.contains("[`CommandRunner`]("));
}

#[test]
fn typedoc_markdown_renders_return_type_literal_members() {
    let mut entry = test_entry("resolveArgs", "function", "/repo/src/resolver.ts", "Resolve.");
    entry.returns = Some(ApiReturnDoc {
        type_annotation: "object".to_string(),
        description: "Resolved args.".to_string(),
        members: vec![
            return_property("values", "ArgValues<A>"),
            return_property("positionals", "string[]"),
            return_property("error", "AggregateError | undefined"),
            return_property("explicit", "ArgExplicitlyProvided<A>"),
        ],
    });
    let out = generate_markdown(&type_link_module(entry), &markdown_typedoc_options());
    let page = out.get("combinators/functions/resolveArgs.md").unwrap();

    assert!(page.contains("## Returns\n\n`object` — Resolved args.\n\n"));
    assert!(page.contains("### error\n\n```ts\nerror: AggregateError | undefined;\n```"));
    assert!(page.contains("### explicit\n\n```ts\nexplicit: ArgExplicitlyProvided<A>;\n```"));
    assert!(page.contains("### values\n\n```ts\nvalues: ArgValues<A>;\n```"));
}

#[test]
fn typedoc_html_renders_return_type_literal_members() {
    let mut entry = test_entry("resolveArgs", "function", "/repo/src/resolver.ts", "Resolve.");
    entry.returns = Some(ApiReturnDoc {
        type_annotation: "object".to_string(),
        description: "Resolved args.".to_string(),
        members: vec![return_property("values", "ArgValues<A>")],
    });
    let out = generate_markdown(&type_link_module(entry), &html_typedoc_options());
    let page = out.get("combinators/functions/resolveArgs.md").unwrap();

    assert!(page.contains("ox-api-entry__return-members"));
    assert!(page.contains("<h5>values</h5>"));
    assert!(
        page.contains("values: <a href=\"../type-aliases/ArgValues.md\">ArgValues</a>&lt;A&gt;;")
    );
}

#[test]
fn typedoc_html_type_declaration_format_table_renders_return_members_table() {
    let mut values = return_property("values", "ArgValues<A>");
    values.description = "Resolved values.".to_string();
    let mut entry = test_entry("resolveArgs", "function", "/repo/src/resolver.ts", "Resolve.");
    entry.returns = Some(ApiReturnDoc {
        type_annotation: "object".to_string(),
        description: "Resolved args.".to_string(),
        members: vec![values],
    });
    let options = MarkdownDocsOptions {
        type_declaration_format: MarkdownDisplayFormat::Table,
        ..html_typedoc_options()
    };
    let out = generate_markdown(&type_link_module(entry), &options);
    let page = out.get("combinators/functions/resolveArgs.md").unwrap();

    assert!(page.contains("ox-api-entry__type-declaration-table"));
    assert!(page.contains("<td><code>values</code></td>"));
    assert!(page.contains("<a href=\"../type-aliases/ArgValues.md\">ArgValues</a>&lt;A&gt;"));
    assert!(page.contains("Resolved values."));
    assert!(!page.contains("ox-api-entry__return-members"));
}
