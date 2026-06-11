use super::*;

pub(super) fn render_member_table_html(
    entry_name: &str,
    title: &str,
    members: &[&ApiDocMember],
    options: &MarkdownDocsOptions,
    context: Option<&MarkdownLinkContext<'_>>,
) -> String {
    if members.is_empty() {
        return String::new();
    }

    let include_kind = super::member_table_includes_kind(title);

    let mut rows = StringBuilder::new();
    for member in members {
        if !rows.is_empty() {
            rows.push_char('\n');
        }
        rows.push_str("<tr id=\"");
        rows.push_str(&escape_html(&member_anchor(
            entry_name,
            member,
            context.map_or(MarkdownPathStrategy::Flat, |context| context.options.path_strategy),
        )));
        rows.push_str("\">\n  <td><code>");
        rows.push_str(&escape_html(&member.name));
        rows.push_str("</code>");
        rows.push_str(&render_member_flags(member));
        rows.push_str("</td>\n  ");
        if include_kind {
            rows.push_str("<td><span class=\"ox-api-entry__member-kind\">");
            rows.push_str(&escape_html(&member.kind));
            rows.push_str("</span></td>\n  ");
        }
        rows.push_str("<td>");
        rows.push_str(&render_member_type_html(member, context, &FxHashSet::default()));
        rows.push_str("</td>\n  <td>");
        rows.push_str(&render_member_description_html(member, options, context));
        rows.push_str("</td>\n</tr>");
        let property_members = render_property_members_html(&member.members, options, context);
        if !property_members.is_empty() {
            let colspan = if include_kind { 4 } else { 3 };
            rows.push_str("\n<tr class=\"ox-api-entry__property-members-row\"><td colspan=\"");
            rows.push_usize(colspan);
            rows.push_str("\">");
            rows.push_str(&property_members);
            rows.push_str("</td></tr>");
        }
    }
    let rows = rows.into_string();

    let title = escape_html(title);
    let mut out = StringBuilder::with_capacity(rows.len() + title.len() + 235);
    out.push_str(
        "<div class=\"ox-api-entry__member-group\">
<h5>",
    );
    out.push_str(&title);
    out.push_str(
        "</h5>
<table class=\"ox-api-entry__members-table\">
",
    );
    if include_kind {
        out.push_str(
            "<thead><tr><th>Name</th><th>Kind</th><th>Type</th><th>Description</th></tr></thead>\n",
        );
    } else {
        out.push_str("<thead><tr><th>Name</th><th>Type</th><th>Description</th></tr></thead>\n");
    }
    out.push_str(
        "<tbody>
",
    );
    out.push_str(&rows);
    out.push_str(
        "
</tbody>
</table>
</div>",
    );
    out.into_string()
}

pub(super) fn render_member_list_html(
    entry_name: &str,
    title: &str,
    members: &[&ApiDocMember],
    options: &MarkdownDocsOptions,
    context: Option<&MarkdownLinkContext<'_>>,
) -> String {
    if members.is_empty() {
        return String::new();
    }

    let mut items = StringBuilder::new();
    for member in members {
        if !items.is_empty() {
            items.push_char('\n');
        }
        items.push_str("<li id=\"");
        items.push_str(&escape_html(&member_anchor(
            entry_name,
            member,
            context.map_or(MarkdownPathStrategy::Flat, |context| context.options.path_strategy),
        )));
        items.push_str("\" class=\"ox-api-entry__member\">\n  <div class=\"ox-api-entry__member-heading\">\n    <code class=\"ox-api-entry__member-name\">");
        items.push_str(&escape_html(&member.name));
        items.push_str("</code>");
        items.push_str(&render_member_flags(member));
        items.push_str("\n    <span class=\"ox-api-entry__member-kind\">");
        items.push_str(&escape_html(&member.kind));
        items.push_str("</span>\n    ");
        items.push_str(&render_member_type_html(member, context, &FxHashSet::default()));
        items.push_str("\n  </div>\n  ");
        items.push_str(&render_member_description_html(member, options, context));
        let property_members = render_property_members_html(&member.members, options, context);
        if !property_members.is_empty() {
            items.push_char('\n');
            items.push_str(&property_members);
        }
        items.push_str("\n</li>");
    }
    let items = items.into_string();

    let title = escape_html(title);
    let mut out = StringBuilder::with_capacity(items.len() + title.len() + 135);
    out.push_str(
        "<div class=\"ox-api-entry__member-group\">
<h5>",
    );
    out.push_str(&title);
    out.push_str(
        "</h5>
<ul class=\"ox-api-entry__members-list\">
",
    );
    out.push_str(&items);
    out.push_str(
        "
</ul>
</div>",
    );
    out.into_string()
}
