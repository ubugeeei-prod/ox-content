use super::*;

#[test]
fn typedoc_dts_overloads_without_implementation_render_all() {
    let docs = overload_module(vec![
        overload_entry(
            "merge",
            "/repo/src/merge.d.ts",
            "Merge one source.",
            "export function merge(a: A): A",
            false,
        ),
        overload_entry(
            "merge",
            "/repo/src/merge.d.ts",
            "Merge two sources.",
            "export function merge(a: A, b: B): A & B",
            false,
        ),
    ]);
    let out = generate_markdown(&docs, &markdown_typedoc_options());
    assert_markdown_map_snapshot("typedoc_dts_overloads_without_implementation_render_all", &out);
    let page = out.get("default/functions/merge.md").unwrap();

    // No implementation exists (.d.ts); every call signature is preserved.
    assert_eq!(page.matches("## Call Signature").count(), 2);
}

#[test]
fn typedoc_renders_experimental_tag_as_warning_alert() {
    let mut entry =
        test_entry("string", "function", "/repo/src/combinators.ts", "Create a string schema.");
    entry.tags = vec![ApiDocTag { tag: "experimental".to_string(), ..Default::default() }];
    let out = generate_markdown(&lifecycle_module(entry), &markdown_typedoc_options());
    assert_markdown_map_snapshot("typedoc_renders_experimental_tag_as_warning_alert", &out);

    // Lifecycle tags move to the alert, not the generic Tags section.
}

#[test]
fn typedoc_renders_deprecated_tag_as_caution_alert_with_body() {
    let mut entry = test_entry("oldDefine", "function", "/repo/src/definition.ts", "Old helper.");
    entry.tags = vec![ApiDocTag {
        tag: "deprecated".to_string(),
        value: "Use `define` instead.".to_string(),
    }];
    let out = generate_markdown(&lifecycle_module(entry), &markdown_typedoc_options());
    assert_markdown_map_snapshot("typedoc_renders_deprecated_tag_as_caution_alert_with_body", &out);
}

#[test]
fn typedoc_keeps_non_structured_tags_in_tags_section() {
    // `@see` is neither a lifecycle tag nor a `@since`/`@version` tag, so it
    // stays in the generic `## Tags` list while structured tags move out.
    let mut entry = test_entry("run", "function", "/repo/src/run.ts", "Run it.");
    entry.tags = vec![
        ApiDocTag { tag: "see".to_string(), value: "related".to_string() },
        ApiDocTag { tag: "since".to_string(), value: "1.0.0".to_string() },
        ApiDocTag { tag: "experimental".to_string(), ..Default::default() },
    ];
    let out = generate_markdown(&lifecycle_module(entry), &markdown_typedoc_options());
    assert_markdown_map_snapshot("typedoc_keeps_non_structured_tags_in_tags_section", &out);
}

#[test]
fn typedoc_marks_experimental_members_in_table() {
    let mut entry =
        test_entry("StringOptions", "interface", "/repo/src/combinators.ts", "String options.");
    entry.members = vec![ApiDocMember {
        name: "minLength".to_string(),
        kind: "property".to_string(),
        description: "Minimum string length.".to_string(),
        type_annotation: Some("number".to_string()),
        optional: true,
        tags: vec![ApiDocTag { tag: "experimental".to_string(), ..Default::default() }],
        ..ApiDocMember::default()
    }];
    let out = generate_markdown(&lifecycle_module(entry), &markdown_typedoc_options());
    assert_markdown_map_snapshot("typedoc_marks_experimental_members_in_table", &out);
}

#[test]
fn typedoc_renders_since_as_dedicated_section() {
    let mut entry = test_entry("Example", "interface", "/repo/src/example.ts", "Example API.");
    entry.tags = vec![ApiDocTag { tag: "since".to_string(), value: "v0.27.0".to_string() }];
    let out = generate_markdown(&lifecycle_module(entry), &markdown_typedoc_options());
    assert_markdown_map_snapshot("typedoc_renders_since_as_dedicated_section", &out);
}

#[test]
fn typedoc_normalizes_version_into_since_section() {
    let mut entry = test_entry("Example", "interface", "/repo/src/example.ts", "Example API.");
    entry.tags = vec![ApiDocTag { tag: "version".to_string(), value: "1.2.3".to_string() }];
    let out = generate_markdown(&lifecycle_module(entry), &markdown_typedoc_options());
    assert_markdown_map_snapshot("typedoc_normalizes_version_into_since_section", &out);
}

#[test]
fn typedoc_combines_since_and_version_into_one_section() {
    let mut entry = test_entry("Example", "interface", "/repo/src/example.ts", "Example API.");
    entry.tags = vec![
        ApiDocTag { tag: "since".to_string(), value: "v0.27.0".to_string() },
        ApiDocTag { tag: "version".to_string(), value: "1.2.3".to_string() },
    ];
    let out = generate_markdown(&lifecycle_module(entry), &markdown_typedoc_options());
    assert_markdown_map_snapshot("typedoc_combines_since_and_version_into_one_section", &out);
    let page = out.get("combinators/interfaces/Example.md").unwrap();

    assert_eq!(page.matches("## Since").count(), 1);
}

#[test]
fn typedoc_renders_member_since_inline() {
    let mut entry =
        test_entry("PluginOptions", "interface", "/repo/src/plugin.ts", "Plugin options.");
    entry.members = vec![ApiDocMember {
        name: "entry".to_string(),
        kind: "property".to_string(),
        description: "Whether this is an entry command.".to_string(),
        type_annotation: Some("boolean".to_string()),
        optional: true,
        tags: vec![ApiDocTag { tag: "since".to_string(), value: "v0.27.0".to_string() }],
        ..ApiDocMember::default()
    }];
    let out = generate_markdown(&lifecycle_module(entry), &markdown_typedoc_options());
    assert_markdown_map_snapshot("typedoc_renders_member_since_inline", &out);
}

#[test]
fn typedoc_renders_module_level_experimental_alert() {
    let docs = vec![ApiDocModule {
        description: "Parser combinator entry point.".to_string(),
        file: "combinators".to_string(),
        tags: vec![ApiDocTag {
            tag: "experimental".to_string(),
            value: "This module is experimental and may change in future versions.".to_string(),
        }],
        entries: vec![test_entry("string", "function", "/repo/src/combinators.ts", "S.")],
        ..ApiDocModule::default()
    }];
    let out = generate_markdown(&docs, &markdown_typedoc_options());
    assert_markdown_map_snapshot("typedoc_renders_module_level_experimental_alert", &out);

    // Alert sits between the title and the description.
}

#[test]
fn typedoc_resolves_links_in_lifecycle_alert_body() {
    let mut entry = test_entry("string", "function", "/repo/src/combinators.ts", "S.");
    entry.tags = vec![ApiDocTag {
        tag: "deprecated".to_string(),
        value: "Use {@link integer} instead.".to_string(),
    }];
    let mut docs = lifecycle_module(entry);
    docs[0].entries.push(test_entry("integer", "function", "/repo/src/combinators.ts", "I."));
    let out = generate_markdown(&docs, &markdown_typedoc_options());
    assert_markdown_map_snapshot("typedoc_resolves_links_in_lifecycle_alert_body", &out);
}
