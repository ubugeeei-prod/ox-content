use super::super::{MarkdownLinkContext, SINCE_TAGS};
use super::format::{code_cell, code_span, inline};
use crate::model::ApiDocMember;
use crate::string_builder::StringBuilder;

pub(super) fn member_name_cell(member: &ApiDocMember) -> String {
    member_name(member, code_cell)
}

pub(super) fn member_name_span(member: &ApiDocMember) -> String {
    member_name(member, code_span)
}

fn member_name(member: &ApiDocMember, code: fn(&str) -> String) -> String {
    let mut flags = Vec::new();
    if member.optional {
        flags.push("optional");
    }
    if member.readonly {
        flags.push("readonly");
    }
    if member.r#static {
        flags.push("static");
    }
    if member.private {
        flags.push("private");
    }

    let name = code(&member.name);
    if flags.is_empty() {
        name
    } else {
        let flags = flags.join(", ");
        let mut out = StringBuilder::with_capacity(name.len() + flags.len() + 5);
        out.push_str(&name);
        out.push_str(" _(");
        out.push_str(&flags);
        out.push_str(")_");
        out.into_string()
    }
}

pub(super) fn member_type(member: &ApiDocMember) -> &str {
    member
        .signature
        .as_deref()
        .or(member.type_annotation.as_deref())
        .or_else(|| member.returns.as_ref().map(|returns| returns.type_annotation.as_str()))
        .unwrap_or_default()
}

pub(super) fn member_description(
    member: &ApiDocMember,
    context: Option<&MarkdownLinkContext<'_>>,
    include_returns: bool,
) -> String {
    let mut description = String::new();
    // Lifecycle tags cannot hold a GitHub alert inside a table cell, so surface
    // them as a short bold marker prefix instead.
    let push_part = |description: &mut String, part: &str| {
        if !description.is_empty() {
            description.push(' ');
        }
        description.push_str(part);
    };
    let mut deprecated = false;
    let mut experimental = false;
    for tag in &member.tags {
        match tag.tag.as_str() {
            "deprecated" => deprecated = true,
            "experimental" => experimental = true,
            _ => {}
        }
        if deprecated && experimental {
            break;
        }
    }
    if deprecated {
        push_part(&mut description, "**Deprecated.**");
    }
    if experimental {
        push_part(&mut description, "**Experimental.**");
    }
    if !member.description.is_empty() {
        push_part(&mut description, &inline(&member.description, context));
    }
    if let Some(default_value) =
        member.default_value.as_deref().map(str::trim).filter(|value| !value.is_empty())
    {
        let default = code_span(default_value);
        if !default.is_empty() {
            let mut part = StringBuilder::with_capacity(default.len() + 14);
            part.push_str("**Default:** ");
            part.push_str(&default);
            push_part(&mut description, &part.into_string());
        }
    }
    if include_returns {
        if let Some(returns) = &member.returns {
            if !returns.description.is_empty() {
                push_part(&mut description, "Returns:");
                description.push(' ');
                description.push_str(&inline(&returns.description, context));
            }
        }
    }
    // `@since` / `@version` render inline (TypeDoc shows them in the cell); a
    // GitHub alert or section cannot live inside a table cell.
    let since = member
        .tags
        .iter()
        .filter(|tag| SINCE_TAGS.contains(&tag.tag.as_str()))
        .map(|tag| inline(&tag.value, context))
        .filter(|value| !value.is_empty())
        .collect::<Vec<_>>();
    if !since.is_empty() {
        push_part(&mut description, "**Since**");
        description.push(' ');
        description.push_str(&since.join(", "));
    }
    description
}
