use super::*;

pub(super) fn render_property_members_html(
    members: &[ApiDocMember],
    options: &MarkdownDocsOptions,
    link_context: Option<&MarkdownLinkContext<'_>>,
) -> String {
    if members.is_empty() {
        return String::new();
    }

    match options.property_members_format {
        MarkdownDisplayFormat::Table => render_nested_members_table_html(
            members,
            "ox-api-entry__property-members-table",
            link_context,
        ),
        MarkdownDisplayFormat::List => render_nested_members_list_html(
            members,
            "ox-api-entry__property-members-list",
            "ox-api-entry__property-member",
            "ox-api-entry__property-member-heading",
            "ox-api-entry__property-member-description",
            link_context,
        ),
        MarkdownDisplayFormat::None => String::new(),
    }
}

pub(super) fn render_nested_members_table_html(
    members: &[ApiDocMember],
    class_name: &str,
    link_context: Option<&MarkdownLinkContext<'_>>,
) -> String {
    let mut rows = StringBuilder::new();
    for member in members {
        if !rows.is_empty() {
            rows.push_char('\n');
        }
        rows.push_str("<tr>\n  <td>");
        rows.push_str(&render_nested_member_name_html(member));
        rows.push_str("</td>\n  <td>");
        rows.push_str(&render_nested_member_type_html(member, link_context));
        rows.push_str("</td>\n  <td>");
        rows.push_str(&render_nested_member_description_html(member, link_context));
        rows.push_str("</td>\n</tr>");
    }

    let mut out = StringBuilder::new();
    out.push_str("<table class=\"");
    out.push_str(class_name);
    out.push_str(
        "\"><thead><tr><th>Name</th><th>Type</th><th>Description</th></tr></thead><tbody>\n",
    );
    out.push_str(&rows.into_string());
    out.push_str("\n</tbody></table>");
    out.into_string()
}

pub(super) fn render_nested_members_list_html(
    members: &[ApiDocMember],
    list_class_name: &str,
    item_class_name: &str,
    heading_class_name: &str,
    description_class_name: &str,
    link_context: Option<&MarkdownLinkContext<'_>>,
) -> String {
    let mut items = StringBuilder::new();
    for member in members {
        if !items.is_empty() {
            items.push_char('\n');
        }
        items.push_str("<li class=\"");
        items.push_str(item_class_name);
        items.push_str("\">");
        items.push_str("<div class=\"");
        items.push_str(heading_class_name);
        items.push_str("\">");
        items.push_str(&render_nested_member_name_html(member));
        let member_type = render_nested_member_type_html(member, link_context);
        if !member_type.is_empty() {
            items.push_char(' ');
            items.push_str(&member_type);
        }
        items.push_str("</div>");
        let description = render_nested_member_description_html(member, link_context);
        if !description.is_empty() {
            items.push_str("<div class=\"");
            items.push_str(description_class_name);
            items.push_str("\">");
            items.push_str(&description);
            items.push_str("</div>");
        }
        items.push_str("</li>");
    }

    let mut out = StringBuilder::new();
    out.push_str("<ul class=\"");
    out.push_str(list_class_name);
    out.push_str("\">\n");
    out.push_str(&items.into_string());
    out.push_str("\n</ul>");
    out.into_string()
}

fn render_nested_member_name_html(member: &ApiDocMember) -> String {
    let mut out = StringBuilder::new();
    out.push_str("<code>");
    out.push_str(&escape_html(&member.name));
    out.push_str("</code>");
    out.push_str(&render_member_flags(member));
    out.into_string()
}

fn render_nested_member_type_html(
    member: &ApiDocMember,
    link_context: Option<&MarkdownLinkContext<'_>>,
) -> String {
    let Some(member_type) = member
        .type_annotation
        .as_deref()
        .or_else(|| member.returns.as_ref().map(|returns| returns.type_annotation.as_str()))
        .or(member.signature.as_deref())
        .filter(|value| !value.is_empty())
    else {
        return String::new();
    };

    let mut out = StringBuilder::new();
    out.push_str("<code class=\"ox-api-entry__member-type language-typescript\">");
    out.push_str(&render_type_inner_html(member_type, link_context, &FxHashSet::default()));
    out.push_str("</code>");
    out.into_string()
}

fn render_nested_member_description_html(
    member: &ApiDocMember,
    link_context: Option<&MarkdownLinkContext<'_>>,
) -> String {
    render_member_description_html(member, &MarkdownDocsOptions::default(), link_context)
}
