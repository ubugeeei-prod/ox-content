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
    assert_markdown_map_snapshot("typedoc_links_known_symbols_in_param_types", &out);

    // Known symbol links (label is the symbol in inline code), `.md` link style.

    // Generic arg `G` (a type parameter) and primitive `number` stay plain code.
}

#[test]
fn typedoc_links_known_symbol_in_return_type() {
    let mut entry = test_entry("make", "function", "/repo/src/make.ts", "Make.");
    entry.returns = Some(ApiReturnDoc {
        type_annotation: "CommandRunner<G>".to_string(),
        ..ApiReturnDoc::default()
    });
    let out = generate_markdown(&type_link_module(entry), &markdown_typedoc_options());
    assert_markdown_map_snapshot("typedoc_links_known_symbol_in_return_type", &out);
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
    assert_markdown_map_snapshot("typedoc_markdown_renders_return_type_literal_members", &out);
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
    assert_markdown_map_snapshot("typedoc_html_renders_return_type_literal_members", &out);
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
    assert_markdown_map_snapshot(
        "typedoc_html_type_declaration_format_table_renders_return_members_table",
        &out,
    );
    assert_markdown_map_snapshot(
        "typedoc_html_type_declaration_format_table_renders_return_members_table",
        &out,
    );
}
