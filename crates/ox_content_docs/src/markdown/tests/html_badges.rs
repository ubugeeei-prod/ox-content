use super::*;

#[test]
fn typedoc_html_members_fallback_keeps_kind_column() {
    // A non class/interface/type/enum entry falls back to a mixed-kind `Members`
    // group, where the Kind column stays meaningful.
    let mut entry = test_entry("config", "variable", "/repo/src/config.ts", "Config.");
    entry.members = vec![member("name", "property", false), member("load", "method", false)];
    let out = generate_markdown(&lifecycle_module(entry), &html_typedoc_options());
    assert_markdown_map_snapshot("typedoc_html_members_fallback_keeps_kind_column", &out);
}

#[test]
fn typedoc_html_overloads_render_all_call_signatures() {
    let docs = overload_module(vec![
        overload_entry(
            "plugin",
            "/repo/src/plugin.ts",
            "Define a plugin with extension.",
            "export function plugin<E>(options: WithExt): PluginWithExtension<E>",
            false,
        ),
        overload_entry(
            "plugin",
            "/repo/src/plugin.ts",
            "Define a plugin without extension.",
            "export function plugin(options: WithoutExt): PluginWithoutExtension",
            false,
        ),
        overload_entry(
            "plugin",
            "/repo/src/plugin.ts",
            "Define a plugin",
            "export function plugin(options: any = {}): any",
            true,
        ),
    ]);
    let out = generate_markdown(&docs, &html_typedoc_options());
    assert_markdown_map_snapshot("typedoc_html_overloads_render_all_call_signatures", &out);
    let page = out.get("default/functions/plugin.md").unwrap();

    // Both public overloads survive on one html page; the implementation is hidden.
    assert_eq!(page.matches("<h4>Call Signature</h4>").count(), 2);
}

#[test]
fn typedoc_html_badges_include_experimental_and_version() {
    let mut entry = test_entry("widget", "function", "/repo/src/w.ts", "A widget.");
    entry.tags = vec![
        ApiDocTag { tag: "experimental".to_string(), ..Default::default() },
        ApiDocTag { tag: "version".to_string(), value: "1.2.3".to_string() },
    ];
    let out = generate_markdown(&lifecycle_module(entry), &html_typedoc_options());
    assert_markdown_map_snapshot("typedoc_html_badges_include_experimental_and_version", &out);

    // Every tag is structured, so no generic tag list is emitted.
}

#[test]
fn typedoc_html_dedups_structured_tags_from_tag_list() {
    let mut entry = test_entry("run", "function", "/repo/src/run.ts", "Run.");
    entry.tags = vec![
        ApiDocTag { tag: "see".to_string(), value: "related".to_string() },
        ApiDocTag { tag: "deprecated".to_string(), value: "use other".to_string() },
        ApiDocTag { tag: "since".to_string(), value: "1.0.0".to_string() },
    ];
    let out = generate_markdown(&lifecycle_module(entry), &html_typedoc_options());
    assert_markdown_map_snapshot("typedoc_html_dedups_structured_tags_from_tag_list", &out);

    // Structured tags become badges (not duplicated in the tag list); `@see` stays.
}

#[test]
fn typedoc_html_member_table_shows_lifecycle_and_since_markers() {
    let mut entry = test_entry("Options", "interface", "/repo/src/o.ts", "Options.");
    entry.members = vec![ApiDocMember {
        name: "mode".to_string(),
        kind: "property".to_string(),
        description: "The mode.".to_string(),
        type_annotation: Some("string".to_string()),
        tags: vec![
            ApiDocTag { tag: "deprecated".to_string(), ..Default::default() },
            ApiDocTag { tag: "since".to_string(), value: "1.0.0".to_string() },
        ],
        ..ApiDocMember::default()
    }];
    let out = generate_markdown(&lifecycle_module(entry), &html_typedoc_options());
    assert_markdown_map_snapshot(
        "typedoc_html_member_table_shows_lifecycle_and_since_markers",
        &out,
    );
    assert_markdown_map_snapshot(
        "typedoc_html_member_table_shows_lifecycle_and_since_markers",
        &out,
    );
}

#[test]
fn typedoc_html_property_members_format_table_renders_owned_members() {
    let mut http = member("http", "property", false);
    http.description = "HTTP options.".to_string();
    http.type_annotation =
        Some("{ timeout?: number; headers: Record<string, string> }".to_string());
    let mut timeout = member("timeout", "property", false);
    timeout.description = "Request timeout.".to_string();
    timeout.type_annotation = Some("number".to_string());
    timeout.optional = true;
    http.members = vec![timeout];

    let mut entry = test_entry("Options", "interface", "/repo/src/o.ts", "Options.");
    entry.members = vec![http];
    let out = generate_markdown(
        &lifecycle_module(entry),
        &MarkdownDocsOptions {
            interface_properties_format: MarkdownDisplayFormat::Table,
            property_members_format: MarkdownDisplayFormat::Table,
            ..html_typedoc_options()
        },
    );
    assert_markdown_map_snapshot(
        "typedoc_html_property_members_format_table_renders_owned_members",
        &out,
    );
    assert_markdown_map_snapshot(
        "typedoc_html_property_members_format_table_renders_owned_members",
        &out,
    );
}

#[test]
fn typedoc_html_property_members_format_list_renders_owned_members() {
    let mut http = member("http", "property", false);
    http.description = "HTTP options.".to_string();
    http.type_annotation = Some("{ timeout?: number }".to_string());
    let mut timeout = member("timeout", "property", false);
    timeout.description = "Request timeout.".to_string();
    timeout.type_annotation = Some("number".to_string());
    timeout.optional = true;
    http.members = vec![timeout];

    let mut entry = test_entry("Options", "interface", "/repo/src/o.ts", "Options.");
    entry.members = vec![http];
    let out = generate_markdown(
        &lifecycle_module(entry),
        &MarkdownDocsOptions {
            interface_properties_format: MarkdownDisplayFormat::List,
            property_members_format: MarkdownDisplayFormat::List,
            ..html_typedoc_options()
        },
    );
    assert_markdown_map_snapshot(
        "typedoc_html_property_members_format_list_renders_owned_members",
        &out,
    );
    assert_markdown_map_snapshot(
        "typedoc_html_property_members_format_list_renders_owned_members",
        &out,
    );
}

#[test]
fn typedoc_html_property_members_format_none_omits_owned_members() {
    let mut http = member("http", "property", false);
    http.description = "HTTP options.".to_string();
    http.type_annotation = Some("{ timeout?: number }".to_string());
    http.members = vec![member("timeout", "property", false)];

    let mut entry = test_entry("Options", "interface", "/repo/src/o.ts", "Options.");
    entry.members = vec![http];
    let out = generate_markdown(
        &lifecycle_module(entry),
        &MarkdownDocsOptions {
            interface_properties_format: MarkdownDisplayFormat::Table,
            property_members_format: MarkdownDisplayFormat::None,
            ..html_typedoc_options()
        },
    );
    assert_markdown_map_snapshot(
        "typedoc_html_property_members_format_none_omits_owned_members",
        &out,
    );
    assert_markdown_map_snapshot(
        "typedoc_html_property_members_format_none_omits_owned_members",
        &out,
    );
}

#[test]
fn typedoc_html_renders_function_valued_property_return_once() {
    let mut entry = test_entry("ArgSchema", "interface", "/repo/src/schema.ts", "Argument schema.");
    entry.members = vec![function_valued_parse_member()];
    let out = generate_markdown(
        &lifecycle_module(entry),
        &MarkdownDocsOptions {
            interface_properties_format: MarkdownDisplayFormat::Table,
            parameters_format: MarkdownDisplayFormat::Table,
            ..html_typedoc_options()
        },
    );
    assert_markdown_map_snapshot("typedoc_html_renders_function_valued_property_return_once", &out);
    let page = out.get("combinators/interfaces/ArgSchema.md").unwrap();

    assert_eq!(page.matches("Parsed value.").count(), 1);
}

#[test]
fn typedoc_html_module_index_shows_lifecycle_badges() {
    let mut docs = lifecycle_module(test_entry("run", "function", "/repo/src/run.ts", "Run."));
    docs[0].tags = vec![ApiDocTag { tag: "experimental".to_string(), ..Default::default() }];
    let out = generate_markdown(&docs, &html_typedoc_options());
    assert_markdown_map_snapshot("typedoc_html_module_index_shows_lifecycle_badges", &out);
}
