use super::*;

fn render_member_group_html(
    entry: &ApiDocEntry,
    title: &str,
    members: &[&ApiDocMember],
    options: &MarkdownDocsOptions,
    context: Option<&MarkdownLinkContext<'_>>,
) -> String {
    if members.is_empty() {
        return String::new();
    }
    if members.iter().all(|member| is_callable_member(member)) {
        return render_callable_member_group_html(entry, title, members, options, context);
    }
    if effective_members_format(options, &entry.kind, title) == MarkdownDisplayFormat::List {
        render_member_list_html(&entry.name, title, members, options, context)
    } else {
        render_member_table_html(&entry.name, title, members, options, context)
    }
}

pub(super) fn render_members_html(
    entry: &ApiDocEntry,
    options: &MarkdownDocsOptions,
    context: Option<&MarkdownLinkContext<'_>>,
) -> String {
    if entry.members.is_empty() {
        return String::new();
    }

    // Bucket the members lazily: each `match` arm below only uses a subset of
    // these groups (the default arm uses none of them), so computing every
    // bucket up front wasted a full `members` pass + `Vec` per unused group.
    let members = entry.members.as_slice();
    let methods = |is_static: bool| {
        members
            .iter()
            .filter(|member| {
                member.r#static == is_static
                    && matches!(member.kind.as_str(), "method" | "getter" | "setter")
            })
            .collect::<Vec<_>>()
    };
    let properties = |is_static: bool| {
        members
            .iter()
            .filter(|member| member.r#static == is_static && member.kind == "property")
            .collect::<Vec<_>>()
    };
    let index_signatures =
        || members.iter().filter(|member| member.kind == "indexSignature").collect::<Vec<_>>();

    let mut groups = Vec::new();
    match entry.kind.as_str() {
        "class" => {
            let constructors =
                members.iter().filter(|member| member.kind == "constructor").collect::<Vec<_>>();
            groups.push(render_member_group_html(
                entry,
                "Constructors",
                &constructors,
                options,
                context,
            ));
            groups.push(render_member_group_html(
                entry,
                "Static Methods",
                &methods(true),
                options,
                context,
            ));
            groups.push(render_member_group_html(
                entry,
                "Methods",
                &methods(false),
                options,
                context,
            ));
            groups.push(render_index_signature_group_html(&index_signatures(), context));
            groups.push(render_member_group_html(
                entry,
                "Static Properties",
                &properties(true),
                options,
                context,
            ));
            groups.push(render_member_group_html(
                entry,
                "Properties",
                &properties(false),
                options,
                context,
            ));
        }
        "interface" => {
            groups.push(render_index_signature_group_html(&index_signatures(), context));
            groups.push(render_member_group_html(
                entry,
                "Properties",
                &properties(false),
                options,
                context,
            ));
            groups.push(render_member_group_html(
                entry,
                "Methods",
                &methods(false),
                options,
                context,
            ));
        }
        "type" => {
            let enum_members =
                members.iter().filter(|member| member.kind == "enumMember").collect::<Vec<_>>();
            groups.push(render_index_signature_group_html(&index_signatures(), context));
            groups.push(render_member_group_html(
                entry,
                "Properties",
                &properties(false),
                options,
                context,
            ));
            groups.push(render_member_group_html(
                entry,
                "Methods",
                &methods(false),
                options,
                context,
            ));
            groups.push(render_member_group_html(
                entry,
                "Enum Members",
                &enum_members,
                options,
                context,
            ));
        }
        "enum" => {
            let enum_members =
                members.iter().filter(|member| member.kind == "enumMember").collect::<Vec<_>>();
            groups.push(render_member_group_html(
                entry,
                "Enum Members",
                &enum_members,
                options,
                context,
            ));
        }
        _ => groups.push(render_member_group_html(
            entry,
            "Members",
            &members.iter().collect::<Vec<_>>(),
            options,
            context,
        )),
    }

    let groups = groups.into_iter().filter(|group| !group.is_empty()).collect::<Vec<_>>();
    if groups.is_empty() {
        return String::new();
    }

    join3(
        "<div class=\"ox-api-entry__section ox-api-entry__section--members\">
<h4>Members</h4>
",
        &groups.join("\n"),
        "
</div>",
    )
}
