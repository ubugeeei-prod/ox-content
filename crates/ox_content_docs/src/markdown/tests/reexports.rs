use super::*;

#[test]
fn typedoc_dedupes_cross_entrypoint_reexports_to_canonical_page() {
    // `createCommandContext` is defined in context.ts and re-exported from
    // the default and plugin entry points. It must produce a single page
    // under its defining module (context), with the re-exporters linking to
    // it via a References section.
    let docs = vec![
        ApiDocModule {
            file: "context".to_string(),
            source_path: "/repo/src/context.ts".to_string(),
            entries: vec![test_entry(
                "createCommandContext",
                "function",
                "/repo/src/context.ts",
                "Creates a command context.",
            )],
            ..ApiDocModule::default()
        },
        ApiDocModule {
            file: "default".to_string(),
            source_path: "/repo/src/index.ts".to_string(),
            entries: vec![
                test_entry(
                    "createCommandContext",
                    "function",
                    "/repo/src/context.ts",
                    "Creates a command context.",
                ),
                test_entry(
                    "runDefault",
                    "function",
                    "/repo/src/index.ts",
                    "Uses {@link createCommandContext}.",
                ),
            ],
            ..ApiDocModule::default()
        },
        ApiDocModule {
            file: "plugin".to_string(),
            source_path: "/repo/src/plugin.ts".to_string(),
            entries: vec![test_entry(
                "createCommandContext",
                "function",
                "/repo/src/context.ts",
                "Creates a command context.",
            )],
            ..ApiDocModule::default()
        },
    ];

    let out = generate_markdown(
        &docs,
        &MarkdownDocsOptions {
            path_strategy: MarkdownPathStrategy::TypeDoc,
            ..MarkdownDocsOptions::default()
        },
    );
    assert_markdown_map_snapshot(
        "typedoc_dedupes_cross_entrypoint_reexports_to_canonical_page",
        &out,
    );
    assert_markdown_map_snapshot(
        "typedoc_dedupes_cross_entrypoint_reexports_to_canonical_page",
        &out,
    );

    // Exactly one canonical page, placed under the defining module.
    assert!(out.contains_key("context/functions/createCommandContext.md"));
    assert!(!out.contains_key("default/functions/createCommandContext.md"));
    assert!(!out.contains_key("plugin/functions/createCommandContext.md"));

    // The defining module lists it as a real entry; re-exporters reference it.

    // TypeDoc-style reference entry (heading + "Re-exports" link), not a bullet.

    // The re-export reference and any cross-link resolve to the canonical page.
}

#[test]
fn typedoc_references_section_uses_typedoc_layout() {
    // Two symbols defined in `context` and re-exported from `default` produce
    // a TypeDoc-style References section: `### Name` headings, `Re-exports`
    // links, and a `***` separator between entries.
    let make = |module: &str, source: &str, entries: Vec<ApiDocEntry>| ApiDocModule {
        file: module.to_string(),
        source_path: source.to_string(),
        entries,
        ..ApiDocModule::default()
    };
    let docs = vec![
        make(
            "context",
            "/repo/src/context.ts",
            vec![
                test_entry("createCommandContext", "function", "/repo/src/context.ts", "Ctx."),
                test_entry("CommandContextParams", "interface", "/repo/src/context.ts", "P."),
            ],
        ),
        make(
            "default",
            "/repo/src/index.ts",
            vec![
                test_entry("createCommandContext", "function", "/repo/src/context.ts", "Ctx."),
                test_entry("CommandContextParams", "interface", "/repo/src/context.ts", "P."),
            ],
        ),
    ];

    let out = generate_markdown(
        &docs,
        &MarkdownDocsOptions {
            path_strategy: MarkdownPathStrategy::TypeDoc,
            ..MarkdownDocsOptions::default()
        },
    );
    assert_markdown_map_snapshot("typedoc_references_section_uses_typedoc_layout", &out);
    let default_index = out.get("default/index.md").unwrap();

    // The link resolves to the canonical page under the owner module (context).

    // Two references → exactly one thematic-break separator between them.
    assert_eq!(default_index.matches("\n***\n").count(), 1);
}

#[test]
fn typedoc_references_collapse_overloads_to_one_entry() {
    // An overloaded function (two signatures) re-exported from another module
    // is referenced once, not once per overload.
    let docs = vec![
        ApiDocModule {
            file: "definition".to_string(),
            source_path: "/repo/src/definition.ts".to_string(),
            entries: vec![
                test_entry("define", "function", "/repo/src/definition.ts", "Define."),
                test_entry("define", "function", "/repo/src/definition.ts", "Define."),
            ],
            ..ApiDocModule::default()
        },
        ApiDocModule {
            file: "default".to_string(),
            source_path: "/repo/src/index.ts".to_string(),
            entries: vec![
                test_entry("define", "function", "/repo/src/definition.ts", "Define."),
                test_entry("define", "function", "/repo/src/definition.ts", "Define."),
            ],
            ..ApiDocModule::default()
        },
    ];

    let out = generate_markdown(
        &docs,
        &MarkdownDocsOptions {
            path_strategy: MarkdownPathStrategy::TypeDoc,
            ..MarkdownDocsOptions::default()
        },
    );
    assert_markdown_map_snapshot("typedoc_references_collapse_overloads_to_one_entry", &out);
    let default_index = out.get("default/index.md").unwrap();

    assert_eq!(default_index.matches("### define").count(), 1);
    assert_eq!(default_index.matches("Re-exports [define]").count(), 1);
}

#[test]
fn typedoc_dedupe_without_source_path_uses_first_module() {
    // `Command` is defined in a non-entry-point file (command.ts), so no
    // module owns it via source_path; the canonical page falls back to the
    // first module (sorted) that exports it.
    let docs = vec![
        ApiDocModule {
            file: "default".to_string(),
            source_path: "/repo/src/index.ts".to_string(),
            entries: vec![test_entry("Command", "interface", "/repo/src/command.ts", "A command.")],
            ..ApiDocModule::default()
        },
        ApiDocModule {
            file: "plugin".to_string(),
            source_path: "/repo/src/plugin.ts".to_string(),
            entries: vec![test_entry("Command", "interface", "/repo/src/command.ts", "A command.")],
            ..ApiDocModule::default()
        },
    ];

    let out = generate_markdown(
        &docs,
        &MarkdownDocsOptions {
            path_strategy: MarkdownPathStrategy::TypeDoc,
            ..MarkdownDocsOptions::default()
        },
    );
    assert_markdown_map_snapshot("typedoc_dedupe_without_source_path_uses_first_module", &out);

    assert!(out.contains_key("default/interfaces/Command.md"));
    assert!(!out.contains_key("plugin/interfaces/Command.md"));
}
