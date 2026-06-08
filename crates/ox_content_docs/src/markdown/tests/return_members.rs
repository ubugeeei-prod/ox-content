use super::*;

#[test]
fn typedoc_html_type_declaration_format_list_renders_return_members_list() {
    let mut values = return_property("values", "ArgValues<A>");
    values.description = "Resolved values.".to_string();
    let mut entry = test_entry("resolveArgs", "function", "/repo/src/resolver.ts", "Resolve.");
    entry.returns = Some(ApiReturnDoc {
        type_annotation: "object".to_string(),
        description: "Resolved args.".to_string(),
        members: vec![values],
    });
    let options = MarkdownDocsOptions {
        type_declaration_format: MarkdownDisplayFormat::List,
        ..html_typedoc_options()
    };
    let out = generate_markdown(&type_link_module(entry), &options);
    let page = out.get("combinators/functions/resolveArgs.md").unwrap();

    assert!(page.contains("ox-api-entry__type-declaration-list"));
    assert!(page.contains("ox-api-entry__type-declaration-member"));
    assert!(page.contains("Resolved values."));
    assert!(!page.contains("ox-api-entry__return-members"));
}
#[test]
fn typedoc_markdown_renders_index_signature_members() {
    let out = generate_markdown(&index_signature_docs(), &markdown_typedoc_options());
    let page = out.get("default/interfaces/Args.md").unwrap();

    assert!(page.contains("## Indexable\n\n"));
    assert!(page.contains("```ts\nreadonly [option: string]: ArgSchema\n```"));
    assert!(page.contains("Argument schema by option name."));
}

#[test]
fn typedoc_html_renders_index_signature_members() {
    let out = generate_markdown(&index_signature_docs(), &html_typedoc_options());
    let page = out.get("default/interfaces/Args.md").unwrap();

    assert!(page.contains("ox-api-entry__member-group--indexable"));
    assert!(page.contains("<h5>Indexable</h5>"));
    assert!(page.contains("readonly [option: string]: <a href=\"./ArgSchema.md\">ArgSchema</a>"));
    assert!(page.contains("Argument schema by option name."));
}

#[test]
fn typedoc_markdown_renders_return_index_signature_members() {
    let mut entry = test_entry("makeArgs", "function", "/repo/src/resolver.ts", "Make.");
    entry.returns = Some(ApiReturnDoc {
        type_annotation: "object".to_string(),
        description: "Resolved args.".to_string(),
        members: vec![index_signature_member(
            "[option: string]",
            "option",
            "string",
            "ArgSchema",
            false,
        )],
    });
    let out = generate_markdown(&type_link_module(entry), &markdown_typedoc_options());
    let page = out.get("combinators/functions/makeArgs.md").unwrap();

    assert!(page.contains("## Returns\n\n`object` — Resolved args.\n\n"));
    assert!(page.contains("### Indexable\n\n```ts\n[option: string]: ArgSchema\n```"));
}

#[test]
fn typedoc_html_renders_return_index_signature_members() {
    let mut entry = test_entry("makeArgs", "function", "/repo/src/resolver.ts", "Make.");
    entry.returns = Some(ApiReturnDoc {
        type_annotation: "object".to_string(),
        description: "Resolved args.".to_string(),
        members: vec![index_signature_member(
            "[option: string]",
            "option",
            "string",
            "ArgSchema",
            false,
        )],
    });
    let out = generate_markdown(&type_link_module(entry), &html_typedoc_options());
    let page = out.get("combinators/functions/makeArgs.md").unwrap();

    assert!(page.contains("ox-api-entry__return-member--indexable"));
    assert!(page.contains("<h5>Indexable</h5>"));
    assert!(page.contains("[option: string]: ArgSchema"));
}

#[test]
fn typedoc_links_type_parameter_constraint_and_default() {
    let mut entry = test_entry("make", "function", "/repo/src/make.ts", "Make.");
    entry.type_parameters = vec![ApiTypeParamDoc {
        name: "G".to_string(),
        constraint: Some("GunshiParamsConstraint".to_string()),
        default: Some("DefaultGunshiParams".to_string()),
        description: "The constraint.".to_string(),
    }];
    let options = MarkdownDocsOptions {
        parameters_format: MarkdownDisplayFormat::Table,
        ..markdown_typedoc_options()
    };
    let out = generate_markdown(&type_link_module(entry), &options);
    let page = out.get("combinators/functions/make.md").unwrap();

    assert!(page.contains("*extends* [`GunshiParamsConstraint`]("));
    assert!(page.contains("= [`DefaultGunshiParams`]("));
    // The type parameter's own name is never linked.
    assert!(page.contains("| `G` *extends*"));
}
