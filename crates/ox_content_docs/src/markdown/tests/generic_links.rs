use super::*;

#[test]
fn typedoc_markdown_table_collapses_multiline_linked_type_parameter_defaults() {
    let mut entry = test_entry("make", "function", "/repo/src/make.ts", "Make.");
    entry.type_parameters = multiline_plugin_ext_type_parameters();
    let options = MarkdownDocsOptions {
        parameters_format: MarkdownDisplayFormat::Table,
        ..markdown_typedoc_options()
    };
    let out = generate_markdown(&type_link_module(entry), &options);
    let page = out.get("combinators/functions/make.md").unwrap();

    assert!(page.contains("| Name |\n| --- |"));
    assert!(!page.contains("| Name | Description |"));
    assert!(page.contains("| `PluginExt` *extends* [`PluginExtension`](../type-aliases/PluginExtension.md)\\<`Extension`, [`DefaultGunshiParams`](../type-aliases/DefaultGunshiParams.md)\\> = [`PluginExtension`](../type-aliases/PluginExtension.md)\\<`Extension`, `ResolvedDepExtensions`\\> |"));
    assert!(!page.contains("\\<\n"));
    assert!(!page.contains("ResolvedDepExtensions`\n"));
}

#[test]
fn typedoc_markdown_list_collapses_multiline_linked_type_parameter_defaults() {
    let mut entry = test_entry("make", "function", "/repo/src/make.ts", "Make.");
    entry.type_parameters = multiline_plugin_ext_type_parameters();
    let out = generate_markdown(&type_link_module(entry), &markdown_typedoc_options());
    let page = out.get("combinators/functions/make.md").unwrap();

    assert!(page.contains("- `PluginExt` *extends* [`PluginExtension`](../type-aliases/PluginExtension.md)\\<`Extension`, [`DefaultGunshiParams`](../type-aliases/DefaultGunshiParams.md)\\> = [`PluginExtension`](../type-aliases/PluginExtension.md)\\<`Extension`, `ResolvedDepExtensions`\\>"));
    assert!(!page.contains("\\<\n"));
    assert!(!page.contains("`Extension`,\n"));
}

#[test]
fn typedoc_does_not_link_sibling_type_parameter_names() {
    let mut entry = test_entry("make", "function", "/repo/src/make.ts", "Make.");
    // `U` is both a sibling type parameter and an exported symbol stub; the
    // constraint must keep it as code, not a link.
    entry.type_parameters = vec![
        ApiTypeParamDoc {
            name: "T".to_string(),
            constraint: Some("U".to_string()),
            ..ApiTypeParamDoc::default()
        },
        ApiTypeParamDoc { name: "U".to_string(), ..ApiTypeParamDoc::default() },
    ];
    let out = generate_markdown(&type_link_module(entry), &markdown_typedoc_options());
    let page = out.get("combinators/functions/make.md").unwrap();

    assert!(!page.contains("[`U`]"));
}

#[test]
fn typedoc_links_symbols_in_generic_and_union_types() {
    let mut entry = test_entry("Command", "interface", "/repo/src/types.ts", "A command.");
    let mut sub = member("subCommands", "property", false);
    sub.type_annotation =
        Some("Record<string, SubCommandable> | Map<string, SubCommandable>".to_string());
    entry.members = vec![sub];
    let options = MarkdownDocsOptions {
        interface_properties_format: MarkdownDisplayFormat::Table,
        ..markdown_typedoc_options()
    };
    let out = generate_markdown(&type_link_module(entry), &options);
    let page = out.get("combinators/interfaces/Command.md").unwrap();

    // Both occurrences of the symbol link; the built-ins stay plain code.
    assert_eq!(page.matches("[`SubCommandable`](").count(), 2);
    assert!(page.contains("`Record`"));
    assert!(page.contains("`Map`"));
    assert!(page.contains("`string`"));
    assert!(!page.contains("[`Record`]"));
    // `string` is intrinsic even though a `string()` symbol exists.
    assert!(!page.contains("[`string`]"));
}

#[test]
fn typedoc_does_not_link_primitive_types_with_colliding_symbols() {
    // gunshi exports `string()` / `boolean()` / `number()` combinators, so those
    // names resolve in the symbol map. Primitive type annotations must still
    // render as plain code, never linking to the combinator pages.
    let mut entry = test_entry("make", "function", "/repo/src/make.ts", "Make.");
    entry.params =
        vec![param("flag", "boolean"), param("name", "string"), param("count", "number")];
    let options = MarkdownDocsOptions {
        parameters_format: MarkdownDisplayFormat::Table,
        ..markdown_typedoc_options()
    };
    let out = generate_markdown(&type_link_module(entry), &options);
    let page = out.get("combinators/functions/make.md").unwrap();

    for primitive in ["boolean", "string", "number"] {
        assert!(page.contains(&format!("`{primitive}`")), "expected `{primitive}` as plain code");
        assert!(!page.contains(&format!("[`{primitive}`]")), "`{primitive}` must not be linked");
    }
}

#[test]
fn typedoc_keeps_unlinkable_type_as_single_code_span() {
    // Regression guard: a type with no resolvable symbol is unchanged.
    let mut entry = test_entry("make", "function", "/repo/src/make.ts", "Make.");
    entry.params = vec![param("value", "string | number")];
    let out = generate_markdown(&type_link_module(entry), &markdown_typedoc_options());
    let page = out.get("combinators/functions/make.md").unwrap();

    assert!(page.contains("`string | number`"));
}

#[test]
fn typedoc_does_not_link_symbols_inside_string_literal_types() {
    let mut entry = test_entry("make", "function", "/repo/src/make.ts", "Make.");
    // `RenderingOptions` exists as a symbol, but inside a string literal type it
    // must not be linked.
    entry.params = vec![param("mode", "\"RenderingOptions\"")];
    let out = generate_markdown(&type_link_module(entry), &markdown_typedoc_options());
    let page = out.get("combinators/functions/make.md").unwrap();

    assert!(!page.contains("[`RenderingOptions`]"));
    assert!(page.contains("`\"RenderingOptions\"`"));
}

#[test]
fn typedoc_html_links_known_symbols_in_types() {
    let mut entry = test_entry("Command", "interface", "/repo/src/types.ts", "A command.");
    let mut rendering = member("rendering", "property", false);
    rendering.type_annotation = Some("RenderingOptions<G>".to_string());
    entry.members = vec![rendering];
    let out = generate_markdown(&type_link_module(entry), &html_typedoc_options());
    let page = out.get("combinators/interfaces/Command.md").unwrap();

    // Anchor lives inside the member-type <code> wrapper; `G` stays escaped text.
    assert!(page.contains("ox-api-entry__member-type language-typescript"));
    assert!(page.contains("<a href=\""));
    assert!(page.contains(">RenderingOptions</a>"));
    assert!(!page.contains(">G</a>"));
}

#[test]
fn typedoc_html_keeps_unlinkable_type_unchanged() {
    let mut entry = test_entry("make", "function", "/repo/src/make.ts", "Make.");
    entry.params = vec![param("value", "string | number")];
    let out = generate_markdown(&type_link_module(entry), &html_typedoc_options());
    let page = out.get("combinators/functions/make.md").unwrap();

    // No anchor in the type cell; escaped union pipe preserved.
    assert!(page.contains("string | number"));
    assert!(!page.contains("<a href=\"./type-aliases"));
}

#[test]
fn typedoc_html_collapses_multiline_linked_type_parameter_defaults() {
    let mut entry = test_entry("make", "function", "/repo/src/make.ts", "Make.");
    entry.type_parameters = multiline_plugin_ext_type_parameters();
    let out = generate_markdown(&type_link_module(entry), &html_typedoc_options());
    let page = out.get("combinators/functions/make.md").unwrap();

    assert!(page.contains("<thead><tr><th>Name</th></tr></thead>"));
    assert!(!page.contains("<th>Description</th>"));
    assert!(page.contains("= <code><a href=\"../type-aliases/PluginExtension.md\">PluginExtension</a>&lt;Extension, ResolvedDepExtensions&gt;</code>"));
    assert!(!page.contains("&lt;\n"));
    assert!(!page.contains("ResolvedDepExtensions\n"));
}
