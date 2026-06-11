use super::super::{effective_members_format, member_table_includes_kind, MarkdownDisplayFormat};
use super::format::{linked_type_cell, linked_type_span, push_table_cell};
use super::member_bits::{member_description, member_name_cell, member_name_span, member_type};
use super::member_details::{is_callable_member, render_callable_member_details_pure};
use super::member_groups::MemberGroupRenderContext;
use super::member_sections::render_member_parameter_sections_pure;
use crate::model::ApiDocMember;

pub(super) fn render_member_group_pure(
    out: &mut String,
    heading: &str,
    title: &str,
    members: &[&ApiDocMember],
    context: &MemberGroupRenderContext<'_, '_>,
) {
    if members.is_empty() {
        return;
    }

    out.push_str(heading);
    out.push(' ');
    out.push_str(title);
    out.push_str("\n\n");
    if members.iter().all(|member| is_callable_member(member)) {
        render_callable_member_details_pure(out, title, members, context);
        return;
    }

    if effective_members_format(context.options, context.entry_kind, title)
        == MarkdownDisplayFormat::List
    {
        for member in members {
            // Append straight into `out`; the row is always emitted, so an
            // intermediate per-member `String` would just be an extra alloc.
            out.push_str("- ");
            out.push_str(&member_name_span(member));
            out.push_str(" `");
            out.push_str(&member.kind);
            out.push('`');
            let member_type = member_type(member);
            if !member_type.is_empty() {
                out.push(' ');
                out.push_str(&linked_type_span(member_type, context.link_context));
            }
            let description = member_description(member, context.link_context, false);
            if !description.is_empty() {
                out.push_str(" - ");
                out.push_str(&description);
            }
            out.push('\n');
        }
    } else {
        let include_kind = member_table_includes_kind(title);
        if include_kind {
            out.push_str("| Name | Kind | Type | Description |\n| --- | --- | --- | --- |\n");
        } else {
            out.push_str("| Name | Type | Description |\n| --- | --- | --- |\n");
        }
        for member in members {
            out.push_str("| ");
            out.push_str(&member_name_cell(member));
            out.push_str(" | ");
            if include_kind {
                push_table_cell(out, &member.kind);
                out.push_str(" | ");
            }
            out.push_str(&linked_type_cell(member_type(member), context.link_context));
            out.push_str(" | ");
            push_table_cell(out, &member_description(member, context.link_context, false));
            out.push_str(" |\n");
        }
    }
    out.push('\n');
    out.push_str(&render_member_parameter_sections_pure(
        members,
        context.options,
        context.link_context,
        context.parameter_section_level,
    ));
}
