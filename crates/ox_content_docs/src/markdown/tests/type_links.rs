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

/// A module with `make(<name>: <type>, …)` next to exported symbols named like
/// the names that the types declare (`required()`, `value()`, …), and the types
/// `Options` and `Base`.
fn declared_names_module(params: &[(&str, &str)]) -> Vec<ApiDocModule> {
    let mut entry = test_entry("make", "function", "/repo/src/make.ts", "Make.");
    entry.params =
        params.iter().map(|(name, type_annotation)| param(name, type_annotation)).collect();
    let mut entries = vec![entry, type_stub("Options"), type_stub("Base")];
    for name in ["required", "value", "key", "first", "label", "input", "parse", "Mixin"] {
        entries.push(function_stub(name));
    }
    vec![ApiDocModule { file: "combinators".to_string(), entries, ..ApiDocModule::default() }]
}

#[test]
fn typedoc_does_not_link_names_declared_in_types() {
    // Property, parameter, index signature key, tuple member, method and type
    // predicate names stay plain code, although functions with those names exist.
    let modules = declared_names_module(&[
        ("a", "{ description?: string; required: R }"),
        ("b", "T & { required: true }"),
        ("c", "(value: string) => void"),
        ("d", "(value?: string, ...input: string[]) => void"),
        ("e", "{ [key: string]: number }"),
        ("f", "[first: string, label?: number]"),
        ("g", "{ readonly required: boolean }"),
        ("h", "{| required: boolean; +value: string |}"),
        ("i", "{ required: boolean, value: string }"),
        ("j", "{ label(): string; parse?<T>(input: T): T }"),
        ("k", "{ get label(): string }"),
        ("l", "asserts value"),
    ]);
    let options = MarkdownDocsOptions {
        parameters_format: MarkdownDisplayFormat::Table,
        ..markdown_typedoc_options()
    };
    let out = generate_markdown(&modules, &options);
    assert_markdown_map_snapshot("typedoc_does_not_link_names_declared_in_types", &out);
}

#[test]
fn typedoc_links_references_next_to_declared_names() {
    // Only `Options` and `Base` are references; the names around them stay code.
    let modules = declared_names_module(&[
        ("a", "{ parse(input: string): Options }"),
        ("b", "value is Options"),
        ("c", "asserts value is Options"),
        ("d", "Array<{ required: Options }>"),
        ("e", "Map<string, { value: Options }>"),
        ("f", "new (value: Options) => Base"),
        ("g", "{ [key: string]: Options }"),
    ]);
    let options = MarkdownDocsOptions {
        parameters_format: MarkdownDisplayFormat::Table,
        ..markdown_typedoc_options()
    };
    let out = generate_markdown(&modules, &options);
    assert_markdown_map_snapshot("typedoc_links_references_next_to_declared_names", &out);
}

#[test]
fn typedoc_keeps_linking_references_before_a_colon_or_a_call() {
    // A conditional type's true branch is followed by `:`, and a heritage clause
    // calls a mixin, but these are references, so they stay links.
    let modules = declared_names_module(&[
        ("a", "typeof required"),
        ("b", "Options extends object ? Options : never"),
        ("c", "Options extends Base ? Options extends object ? Options : Options : Options"),
        ("d", "T extends Base ? keyof Options : never"),
        ("e", "T extends Base ? Options | Base : never"),
        ("f", "T extends Base ? () => Options : never"),
        ("g", "T extends Base ? (x: T) => x is Options : never"),
        ("h", "Mixin(Base)"),
        ("i", "T extends Base ? (Options) : never"),
    ]);
    let options = MarkdownDocsOptions {
        parameters_format: MarkdownDisplayFormat::Table,
        ..markdown_typedoc_options()
    };
    let out = generate_markdown(&modules, &options);
    assert_markdown_map_snapshot("typedoc_keeps_linking_references_before_a_colon_or_a_call", &out);
}

#[test]
fn typedoc_html_does_not_link_names_declared_in_types() {
    let modules = declared_names_module(&[
        ("a", "{ description?: string; required: R }"),
        ("b", "(value: string) => void"),
        ("c", "{ parse(input: string): Options }"),
        ("d", "value is Options"),
    ]);
    let out = generate_markdown(&modules, &html_typedoc_options());
    assert_markdown_map_snapshot("typedoc_html_does_not_link_names_declared_in_types", &out);
}
