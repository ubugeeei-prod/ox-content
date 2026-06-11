use super::super::{process_doc_text, rendered_throws};
use super::{
    lifecycle::{push_lifecycle_alerts, render_since_section},
    member_groups::MemberGroupRenderContext,
    parameters::push_parameters,
    return_members::push_returns,
    sections::{push_generic_tags, push_throws},
    type_parameters::push_type_parameters,
};
use crate::model::{ApiDocMember, ApiReturnDoc};
use crate::string_builder::StringBuilder;

pub(super) fn render_callable_member_details_pure(
    out: &mut String,
    title: &str,
    members: &[&ApiDocMember],
    context: &MemberGroupRenderContext<'_, '_>,
) {
    let member_heading = "#".repeat(context.parameter_section_level);
    let detail_heading = "#".repeat(context.parameter_section_level + 1);

    for (index, member) in members.iter().enumerate() {
        if index > 0 {
            out.push_str("***\n\n");
        }
        out.push_str(&member_heading);
        out.push(' ');
        push_callable_member_heading(out, member, title);
        out.push_str("\n\n");

        if let Some(signature) = callable_member_signature(member, context.entry_name) {
            out.push_str("```ts\n");
            out.push_str(&signature);
            out.push_str("\n```\n\n");
        }

        push_lifecycle_alerts(out, &member.tags, context.link_context);

        let description = process_doc_text(&member.description, context.link_context);
        let description = description.trim();
        if !description.is_empty() {
            out.push_str(description);
            out.push_str("\n\n");
        }

        out.push_str(&render_since_section(&member.tags, context.link_context, &detail_heading));
        push_type_parameters(
            out,
            &member.type_parameters,
            context.options,
            context.link_context,
            &detail_heading,
        );
        push_parameters(
            out,
            &member.params,
            context.options,
            context.link_context,
            &detail_heading,
        );
        if let Some(returns) = &member.returns {
            push_returns(out, returns, context.link_context, &detail_heading);
        } else if member.kind == "constructor" {
            let returns = ApiReturnDoc {
                type_annotation: context.entry_name.to_string(),
                description: String::new(),
                members: Vec::new(),
            };
            push_returns(out, &returns, context.link_context, &detail_heading);
        }
        let throws = rendered_throws(&member.throws, &member.tags);
        push_throws(out, throws.as_ref(), context.link_context, &detail_heading);
        push_implementation_of(out, &member.implementation_of, &detail_heading);
        push_generic_tags(out, &member.tags, context.link_context, &detail_heading);
    }
}

pub(super) fn is_callable_member(member: &ApiDocMember) -> bool {
    matches!(member.kind.as_str(), "constructor" | "method" | "getter" | "setter")
}

fn push_callable_member_heading(out: &mut String, member: &ApiDocMember, title: &str) {
    if member.kind == "constructor" {
        out.push_str("Constructor");
        return;
    }
    out.push_str(&member.name);
    if !matches!(member.kind.as_str(), "getter" | "setter") && title.contains("Methods") {
        out.push_str("()");
    }
}

fn callable_member_signature(member: &ApiDocMember, entry_name: &str) -> Option<String> {
    let signature = member.signature.as_deref()?.trim();
    if signature.is_empty() {
        return None;
    }

    let signature = signature.trim_end_matches(';').trim_end();
    let mut out = StringBuilder::new();
    if member.kind == "constructor" {
        if let Some(args) = signature.strip_prefix("constructor") {
            out.push_str("new ");
            out.push_str(entry_name);
            out.push_str(args.trim());
            out.push_str(": ");
            out.push_str(entry_name);
        } else {
            out.push_str(signature);
        }
    } else {
        out.push_str(signature);
    }
    out.push_char(';');
    Some(out.into_string())
}

fn push_implementation_of(out: &mut String, implementation_of: &[String], heading: &str) {
    if implementation_of.is_empty() {
        return;
    }
    out.push_str(heading);
    out.push_str(" Implementation of\n\n");
    for implementation in implementation_of {
        out.push_str("```ts\n");
        out.push_str(implementation.trim());
        out.push_str("\n```\n\n");
    }
}
