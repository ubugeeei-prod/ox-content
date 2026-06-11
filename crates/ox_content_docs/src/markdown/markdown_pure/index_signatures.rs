use super::{
    push_generic_tags, push_lifecycle_alerts, push_throws, render_since_section,
    MarkdownLinkContext, MemberGroupRenderContext,
};
use crate::model::ApiDocMember;

pub(super) fn push_index_signature_detail_pure(
    out: &mut String,
    member: &ApiDocMember,
    context: Option<&MarkdownLinkContext<'_>>,
    detail_heading: &str,
) {
    out.push_str("```ts\n");
    push_index_signature_code_pure(out, member);
    out.push_str("\n```\n\n");

    push_lifecycle_alerts(out, &member.tags, context);
    let description = super::process_doc_text(&member.description, context);
    let description = description.trim();
    if !description.is_empty() {
        out.push_str(description);
        out.push_str("\n\n");
    }
    out.push_str(&render_since_section(&member.tags, context, detail_heading));
    let throws = super::super::rendered_throws(&member.throws, &member.tags);
    push_throws(out, throws.as_ref(), context, detail_heading);
    push_generic_tags(out, &member.tags, context, detail_heading);
}

pub(super) fn render_index_signature_group_pure(
    out: &mut String,
    heading: &str,
    members: &[&ApiDocMember],
    context: &MemberGroupRenderContext<'_, '_>,
) {
    if members.is_empty() {
        return;
    }
    let detail_heading = "#".repeat(context.parameter_section_level);
    out.push_str(heading);
    out.push_str(" Indexable\n\n");
    for member in members {
        push_index_signature_detail_pure(out, member, context.link_context, &detail_heading);
    }
}

fn push_index_signature_code_pure(out: &mut String, member: &ApiDocMember) {
    if let Some(signature) = member.signature.as_deref().filter(|signature| !signature.is_empty()) {
        out.push_str(signature.trim().trim_end_matches(';').trim_end());
        return;
    }
    if member.readonly {
        out.push_str("readonly ");
    }
    out.push_str(&member.name);
    out.push_str(": ");
    let member_type = member
        .type_annotation
        .as_deref()
        .or_else(|| member.returns.as_ref().map(|returns| returns.type_annotation.as_str()))
        .unwrap_or("unknown");
    out.push_str(member_type);
}
