use super::*;

#[test]
fn typedoc_overloads_render_all_call_signatures() {
    let mut with_extension = overload_entry(
        "plugin",
        "/repo/src/plugin.ts",
        "Define a plugin with extension.",
        "export function plugin<E>(options: WithExt): Promise<string | undefined>",
        false,
    );
    with_extension.returns = Some(ApiReturnDoc {
        type_annotation: "Promise<string | undefined>".to_string(),
        description: "A rendered usage or undefined.".to_string(),
        members: Vec::new(),
    });
    let docs = overload_module(vec![
        with_extension,
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
    let out = generate_markdown(&docs, &markdown_typedoc_options());
    let page = out.get("default/functions/plugin.md").unwrap();

    assert!(page.contains("# Function: plugin()"));
    // Both public overloads survive on one page (not overwritten by the last).
    assert_eq!(page.matches("## Call Signature").count(), 2);
    assert!(page.contains("Promise<string | undefined>"));
    assert!(page.contains("PluginWithoutExtension"));
    assert!(page
        .contains("### Returns\n\n`Promise<string | undefined>` — A rendered usage or undefined."));
    assert!(!page.contains("`Promise<string \\| undefined>`"));
}

#[test]
fn typedoc_overloads_omit_implementation_signature() {
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
    let out = generate_markdown(&docs, &markdown_typedoc_options());
    let page = out.get("default/functions/plugin.md").unwrap();

    // The implementation signature is hidden, not rendered as a call signature.
    assert!(!page.contains("options: any = {}"));
    assert!(!page.contains("## Signature"));
}

#[test]
fn typedoc_overload_page_hoists_implementation_summary_and_since() {
    let mut implementation = overload_entry(
        "plugin",
        "/repo/src/plugin.ts",
        "Define a plugin",
        "export function plugin(options: any = {}): any",
        true,
    );
    implementation.tags =
        vec![ApiDocTag { tag: "since".to_string(), value: "v0.27.0".to_string() }];
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
        implementation,
    ]);
    let out = generate_markdown(&docs, &markdown_typedoc_options());
    let page = out.get("default/functions/plugin.md").unwrap();

    // The implementation's summary and `## Since` are hoisted above the first
    // call signature (TypeDoc treats the implementation comment as the symbol
    // comment).
    assert!(page.contains("Define a plugin\n\n## Since\n\nv0.27.0"));
    let since = page.find("## Since").unwrap();
    let call = page.find("## Call Signature").unwrap();
    assert!(since < call);
}

#[test]
fn typedoc_single_public_overload_renders_inline() {
    let docs = overload_module(vec![
            overload_entry(
                "define",
                "/repo/src/definition.ts",
                "Define a command.",
                "export function define<G>(definition: CommandDefinition<G>): CommandDefinitionResult<G>",
                false,
            ),
            overload_entry(
                "define",
                "/repo/src/definition.ts",
                "Define a command.",
                "export function define(definition: any): any",
                true,
            ),
        ]);
    let out = generate_markdown(&docs, &markdown_typedoc_options());
    let page = out.get("default/functions/define.md").unwrap();

    // A single public overload collapses to a normal symbol page (no
    // `## Call Signature` wrapper) showing the typed signature, not `any`.
    assert!(!page.contains("## Call Signature"));
    assert!(page.contains("## Signature"));
    assert!(page.contains("CommandDefinition<G>"));
    assert!(!page.contains("definition: any"));
}
