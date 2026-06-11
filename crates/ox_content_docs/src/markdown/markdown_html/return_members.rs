use super::*;

pub(super) fn render_return_members_html(
    members: &[ApiDocMember],
    options: &MarkdownDocsOptions,
    link_context: Option<&MarkdownLinkContext<'_>>,
) -> String {
    if members.is_empty() {
        return String::new();
    }

    match options.type_declaration_format {
        MarkdownDisplayFormat::Table => {
            return render_type_declaration_members_table_html(members, link_context);
        }
        MarkdownDisplayFormat::List => {
            return render_type_declaration_members_list_html(members, link_context);
        }
        MarkdownDisplayFormat::None => {}
    }

    let mut rendered = StringBuilder::new();
    rendered.push_str("<div class=\"ox-api-entry__return-members\">");
    for member in members {
        if member.kind == "indexSignature" {
            rendered.push_str("\n<div class=\"ox-api-entry__return-member ox-api-entry__return-member--indexable\">\n<h5>Indexable</h5>\n");
            rendered.push_str(&render_index_signature_code_block_html(member, link_context));
            rendered.push_str("\n</div>");
            continue;
        }
        rendered.push_str("\n<div class=\"ox-api-entry__return-member\">\n<h5>");
        rendered.push_str(&escape_html(&member.name));
        rendered.push_str(
            "</h5>\n<code class=\"ox-api-entry__return-member-type language-typescript\">",
        );
        push_return_member_signature_html(&mut rendered, member, link_context);
        rendered.push_str("</code>");
        let description =
            render_member_description_html(member, &MarkdownDocsOptions::default(), link_context);
        if !description.is_empty() {
            rendered.push_str(&description);
        }
        rendered.push_str("\n</div>");
    }
    rendered.push_str("\n</div>");
    rendered.into_string()
}

fn render_type_declaration_members_table_html(
    members: &[ApiDocMember],
    link_context: Option<&MarkdownLinkContext<'_>>,
) -> String {
    render_nested_members_table_html(members, "ox-api-entry__type-declaration-table", link_context)
}

fn render_type_declaration_members_list_html(
    members: &[ApiDocMember],
    link_context: Option<&MarkdownLinkContext<'_>>,
) -> String {
    render_nested_members_list_html(
        members,
        "ox-api-entry__type-declaration-list",
        "ox-api-entry__type-declaration-member",
        "ox-api-entry__type-declaration-member-heading",
        "ox-api-entry__type-declaration-member-description",
        link_context,
    )
}

fn push_return_member_signature_html(
    out: &mut StringBuilder,
    member: &ApiDocMember,
    link_context: Option<&MarkdownLinkContext<'_>>,
) {
    if let Some(signature) = member.signature.as_deref().filter(|signature| !signature.is_empty()) {
        out.push_str(&escape_html(signature.trim()));
        if !signature.trim_end().ends_with(';') {
            out.push_char(';');
        }
        return;
    }

    if member.readonly {
        out.push_str("readonly ");
    }
    out.push_str(&escape_html(&member.name));
    if member.optional {
        out.push_char('?');
    }
    out.push_str(": ");
    let member_type = member
        .type_annotation
        .as_deref()
        .or_else(|| member.returns.as_ref().map(|returns| returns.type_annotation.as_str()))
        .unwrap_or("unknown");
    out.push_str(&render_type_inner_html(member_type, link_context, &FxHashSet::default()));
    out.push_char(';');
}
