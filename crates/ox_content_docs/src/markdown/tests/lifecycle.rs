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
    let page = out.get("default/functions/merge.md").unwrap();

    // No implementation exists (.d.ts); every call signature is preserved.
    assert_eq!(page.matches("## Call Signature").count(), 2);
    assert!(page.contains("merge(a: A): A"));
    assert!(page.contains("merge(a: A, b: B): A & B"));
}

#[test]
fn typedoc_renders_experimental_tag_as_warning_alert() {
    let mut entry =
        test_entry("string", "function", "/repo/src/combinators.ts", "Create a string schema.");
    entry.tags = vec![ApiDocTag { tag: "experimental".to_string(), value: String::new() }];
    let out = generate_markdown(&lifecycle_module(entry), &markdown_typedoc_options());
    let page = out.get("combinators/functions/string.md").unwrap();

    assert!(page
        .contains("> [!WARNING]\n> This API is experimental and may change in future versions."));
    // Lifecycle tags move to the alert, not the generic Tags section.
    assert!(!page.contains("## Tags"));
    assert!(!page.contains("@experimental"));
    assert!(!page.contains("**Deprecated.**"));
}

#[test]
fn typedoc_renders_deprecated_tag_as_caution_alert_with_body() {
    let mut entry = test_entry("oldDefine", "function", "/repo/src/definition.ts", "Old helper.");
    entry.tags = vec![ApiDocTag {
        tag: "deprecated".to_string(),
        value: "Use `define` instead.".to_string(),
    }];
    let out = generate_markdown(&lifecycle_module(entry), &markdown_typedoc_options());
    let page = out.get("combinators/functions/oldDefine.md").unwrap();

    assert!(page.contains("> [!CAUTION]\n> Use `define` instead."));
    assert!(!page.contains("## Tags"));
}

#[test]
fn typedoc_keeps_non_structured_tags_in_tags_section() {
    // `@see` is neither a lifecycle tag nor a `@since`/`@version` tag, so it
    // stays in the generic `## Tags` list while structured tags move out.
    let mut entry = test_entry("run", "function", "/repo/src/run.ts", "Run it.");
    entry.tags = vec![
        ApiDocTag { tag: "see".to_string(), value: "related".to_string() },
        ApiDocTag { tag: "since".to_string(), value: "1.0.0".to_string() },
        ApiDocTag { tag: "experimental".to_string(), value: String::new() },
    ];
    let out = generate_markdown(&lifecycle_module(entry), &markdown_typedoc_options());
    let page = out.get("combinators/functions/run.md").unwrap();

    assert!(page.contains("> [!WARNING]"));
    assert!(page.contains("## Since"));
    assert!(page.contains("## Tags"));
    assert!(page.contains("`@see`"));
    assert!(!page.contains("`@since`"));
    assert!(!page.contains("`@experimental`"));
}

#[test]
fn typedoc_marks_experimental_members_in_table() {
    let mut entry =
        test_entry("StringOptions", "interface", "/repo/src/combinators.ts", "String options.");
    entry.members = vec![ApiDocMember {
        name: "minLength".to_string(),
        kind: "property".to_string(),
        description: "Minimum string length.".to_string(),
        signature: None,
        type_annotation: Some("number".to_string()),
        default_value: None,
        params: vec![],
        type_parameters: vec![],
        returns: None,
        throws: vec![],
        members: vec![],
        optional: true,
        readonly: false,
        r#static: false,
        private: false,
        tags: vec![ApiDocTag { tag: "experimental".to_string(), value: String::new() }],
        implementation_of: vec![],
        line: 1,
        end_line: 1,
    }];
    let out = generate_markdown(&lifecycle_module(entry), &markdown_typedoc_options());
    let page = out.get("combinators/interfaces/StringOptions.md").unwrap();

    assert!(page.contains("**Experimental.** Minimum string length."));
}

#[test]
fn typedoc_renders_since_as_dedicated_section() {
    let mut entry = test_entry("Example", "interface", "/repo/src/example.ts", "Example API.");
    entry.tags = vec![ApiDocTag { tag: "since".to_string(), value: "v0.27.0".to_string() }];
    let out = generate_markdown(&lifecycle_module(entry), &markdown_typedoc_options());
    let page = out.get("combinators/interfaces/Example.md").unwrap();

    assert!(page.contains("## Since\n\nv0.27.0"));
    assert!(!page.contains("## Tags"));
    assert!(!page.contains("@since"));
}

#[test]
fn typedoc_normalizes_version_into_since_section() {
    let mut entry = test_entry("Example", "interface", "/repo/src/example.ts", "Example API.");
    entry.tags = vec![ApiDocTag { tag: "version".to_string(), value: "1.2.3".to_string() }];
    let out = generate_markdown(&lifecycle_module(entry), &markdown_typedoc_options());
    let page = out.get("combinators/interfaces/Example.md").unwrap();

    assert!(page.contains("## Since\n\n1.2.3"));
    assert!(!page.contains("## Version"));
    assert!(!page.contains("## Tags"));
}

#[test]
fn typedoc_combines_since_and_version_into_one_section() {
    let mut entry = test_entry("Example", "interface", "/repo/src/example.ts", "Example API.");
    entry.tags = vec![
        ApiDocTag { tag: "since".to_string(), value: "v0.27.0".to_string() },
        ApiDocTag { tag: "version".to_string(), value: "1.2.3".to_string() },
    ];
    let out = generate_markdown(&lifecycle_module(entry), &markdown_typedoc_options());
    let page = out.get("combinators/interfaces/Example.md").unwrap();

    assert!(page.contains("## Since\n\nv0.27.0\n\n1.2.3"));
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
        signature: None,
        type_annotation: Some("boolean".to_string()),
        default_value: None,
        params: vec![],
        type_parameters: vec![],
        returns: None,
        throws: vec![],
        members: vec![],
        optional: true,
        readonly: false,
        r#static: false,
        private: false,
        tags: vec![ApiDocTag { tag: "since".to_string(), value: "v0.27.0".to_string() }],
        implementation_of: vec![],
        line: 1,
        end_line: 1,
    }];
    let out = generate_markdown(&lifecycle_module(entry), &markdown_typedoc_options());
    let page = out.get("combinators/interfaces/PluginOptions.md").unwrap();

    assert!(page.contains("Whether this is an entry command. **Since** v0.27.0"));
}

#[test]
fn typedoc_renders_module_level_experimental_alert() {
    let docs = vec![ApiDocModule {
        description: "Parser combinator entry point.".to_string(),
        file: "combinators".to_string(),
        source_path: String::new(),
        examples: vec![],
        tags: vec![ApiDocTag {
            tag: "experimental".to_string(),
            value: "This module is experimental and may change in future versions.".to_string(),
        }],
        entries: vec![test_entry("string", "function", "/repo/src/combinators.ts", "S.")],
    }];
    let out = generate_markdown(&docs, &markdown_typedoc_options());
    let index = out.get("combinators/index.md").unwrap();

    // Alert sits between the title and the description.
    assert!(index.contains(
            "# combinators\n\n> [!WARNING]\n> This module is experimental and may change in future versions.\n\nParser combinator entry point."
        ));
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
    let page = out.get("combinators/functions/string.md").unwrap();

    assert!(page.contains("> [!CAUTION]"));
    assert!(page.contains("Use [integer](./integer.md) instead."));
    assert!(!page.contains("{@link"));
}
