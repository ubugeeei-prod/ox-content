use super::*;

#[test]
fn typedoc_html_members_fallback_keeps_kind_column() {
    // A non class/interface/type/enum entry falls back to a mixed-kind `Members`
    // group, where the Kind column stays meaningful.
    let mut entry = test_entry("config", "variable", "/repo/src/config.ts", "Config.");
    entry.members = vec![member("name", "property", false), member("load", "method", false)];
    let out = generate_markdown(&lifecycle_module(entry), &html_typedoc_options());
    let page = out.get("combinators/variables/config.md").unwrap();

    assert!(page.contains("<th>Name</th><th>Kind</th><th>Type</th><th>Description</th>"));
    assert!(page.contains("ox-api-entry__member-kind"));
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
    let page = out.get("default/functions/plugin.md").unwrap();

    // Both public overloads survive on one html page; the implementation is hidden.
    assert_eq!(page.matches("<h4>Call Signature</h4>").count(), 2);
    assert!(page.contains("PluginWithExtension"));
    assert!(page.contains("PluginWithoutExtension"));
    assert!(!page.contains("options: any = {}"));
}

#[test]
fn typedoc_html_badges_include_experimental_and_version() {
    let mut entry = test_entry("widget", "function", "/repo/src/w.ts", "A widget.");
    entry.tags = vec![
        ApiDocTag { tag: "experimental".to_string(), ..Default::default() },
        ApiDocTag { tag: "version".to_string(), value: "1.2.3".to_string() },
    ];
    let out = generate_markdown(&lifecycle_module(entry), &html_typedoc_options());
    let page = out.get("combinators/functions/widget.md").unwrap();

    assert!(page.contains(">experimental</span>"));
    assert!(page.contains("version 1.2.3"));
    // Every tag is structured, so no generic tag list is emitted.
    assert!(!page.contains("ox-api-entry__section--tags"));
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
    let page = out.get("combinators/functions/run.md").unwrap();

    // Structured tags become badges (not duplicated in the tag list); `@see` stays.
    assert!(page.contains(">deprecated</span>"));
    assert!(page.contains("since 1.0.0"));
    assert!(page.contains("@see"));
    assert!(!page.contains("@deprecated"));
    assert!(!page.contains("@since"));
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
    let page = out.get("combinators/interfaces/Options.md").unwrap();

    assert!(page.contains(">deprecated</span>"));
    assert!(page.contains("since 1.0.0"));
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
    let page = out.get("combinators/interfaces/Options.md").unwrap();

    assert!(page.contains("ox-api-entry__property-members-row"));
    assert!(page.contains("ox-api-entry__property-members-table"));
    assert!(
        page.contains("<td><code>timeout</code><span class=\"ox-api-badge\">optional</span></td>")
    );
    assert!(page.contains("Request timeout."));
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
    let page = out.get("combinators/interfaces/Options.md").unwrap();

    assert!(page.contains("ox-api-entry__property-members-list"));
    assert!(page.contains("ox-api-entry__property-member"));
    assert!(page.contains("Request timeout."));
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
    let page = out.get("combinators/interfaces/Options.md").unwrap();

    assert!(!page.contains("ox-api-entry__property-members-table"));
    assert!(!page.contains("ox-api-entry__property-members-list"));
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
    let page = out.get("combinators/interfaces/ArgSchema.md").unwrap();

    assert!(page.contains("Parses a raw value."));
    assert!(page.contains("<table class=\"ox-api-entry__member-params-table\">"));
    assert!(page.contains(
        "<div class=\"ox-api-entry__member-return\"><span>Returns</span> Parsed value.</div>"
    ));
    assert_eq!(page.matches("Parsed value.").count(), 1);
    assert!(!page.contains("ox-api-entry__member-detail-section--returns"));
}

#[test]
fn typedoc_html_module_index_shows_lifecycle_badges() {
    let mut docs = lifecycle_module(test_entry("run", "function", "/repo/src/run.ts", "Run."));
    docs[0].tags = vec![ApiDocTag { tag: "experimental".to_string(), ..Default::default() }];
    let out = generate_markdown(&docs, &html_typedoc_options());
    let module_index = out.get("combinators/index.md").unwrap();

    assert!(module_index.contains("ox-api-module__meta"));
    assert!(module_index.contains(">experimental</span>"));
}
