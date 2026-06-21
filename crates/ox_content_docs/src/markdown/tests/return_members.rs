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
    assert_markdown_map_snapshot(
        "typedoc_html_type_declaration_format_list_renders_return_members_list",
        &out,
    );
    assert_markdown_map_snapshot(
        "typedoc_html_type_declaration_format_list_renders_return_members_list",
        &out,
    );
}
#[test]
fn typedoc_markdown_renders_index_signature_members() {
    let out = generate_markdown(&index_signature_docs(), &markdown_typedoc_options());
    assert_markdown_map_snapshot("typedoc_markdown_renders_index_signature_members", &out);
}

#[test]
fn typedoc_html_renders_index_signature_members() {
    let out = generate_markdown(&index_signature_docs(), &html_typedoc_options());
    assert_markdown_map_snapshot("typedoc_html_renders_index_signature_members", &out);
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
    assert_markdown_map_snapshot("typedoc_markdown_renders_return_index_signature_members", &out);
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
    assert_markdown_map_snapshot("typedoc_html_renders_return_index_signature_members", &out);
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
    assert_markdown_map_snapshot("typedoc_links_type_parameter_constraint_and_default", &out);

    // The type parameter's own name is never linked.
}
