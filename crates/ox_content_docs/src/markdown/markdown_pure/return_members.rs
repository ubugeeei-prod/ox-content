use super::super::MarkdownLinkContext;
use super::format::{inline, linked_type_span};
use super::index_signatures::push_index_signature_detail_pure;
use super::member_bits::{member_description, member_type};
use crate::model::{ApiDocMember, ApiReturnDoc};
use crate::string_builder::{join2, StringBuilder};

/// Appends a `{heading} Returns` section for a return doc.
pub(super) fn push_returns(
    out: &mut String,
    returns: &ApiReturnDoc,
    context: Option<&MarkdownLinkContext<'_>>,
    heading: &str,
) {
    out.push_str(heading);
    out.push_str(" Returns\n\n");
    out.push_str(&linked_type_span(&returns.type_annotation, context));
    if !returns.description.is_empty() {
        out.push_str(" — ");
        out.push_str(&inline(&returns.description, context));
    }
    out.push_str("\n\n");
    push_return_members(out, &returns.members, context, heading);
}

pub(super) fn push_return_members(
    out: &mut String,
    members: &[ApiDocMember],
    context: Option<&MarkdownLinkContext<'_>>,
    heading: &str,
) {
    if members.is_empty() {
        return;
    }

    let mut rendered_indexable_heading = false;
    for member in members {
        if member.kind == "indexSignature" {
            if !rendered_indexable_heading {
                out.push_str(heading);
                out.push_str("# Indexable\n\n");
                rendered_indexable_heading = true;
            }
            let detail_heading = join2(heading, "##");
            push_index_signature_detail_pure(out, member, context, &detail_heading);
            continue;
        }
        out.push_str(heading);
        out.push('#');
        out.push(' ');
        out.push_str(&member.name);
        out.push_str("\n\n```ts\n");
        out.push_str(&return_member_signature(member));
        out.push_str("\n```\n\n");
        let description = member_description(member, context, true);
        if !description.is_empty() {
            out.push_str(&description);
            out.push_str("\n\n");
        }
    }
}

fn return_member_signature(member: &ApiDocMember) -> String {
    if let Some(signature) = member.signature.as_deref().filter(|signature| !signature.is_empty()) {
        let signature = signature.trim();
        return if signature.ends_with(';') {
            signature.to_string()
        } else {
            let mut out = StringBuilder::with_capacity(signature.len() + 1);
            out.push_str(signature);
            out.push_char(';');
            out.into_string()
        };
    }

    let member_type = member_type(member);
    let member_type = if member_type.is_empty() { "unknown" } else { member_type };
    let mut out = StringBuilder::with_capacity(member.name.len() + member_type.len() + 16);
    if member.readonly {
        out.push_str("readonly ");
    }
    out.push_str(&member.name);
    if member.optional {
        out.push_char('?');
    }
    out.push_str(": ");
    out.push_str(member_type);
    out.push_char(';');
    out.into_string()
}
