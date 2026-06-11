use rustc_hash::FxHashSet;

use super::{
    escape_html, render_code_block_html, render_doc_inline_html, render_member_params_html,
    render_member_type_parameters_html, render_return_members_html, render_throws_list_html,
    render_type_inner_html,
};
use super::{member_anchor, MarkdownDocsOptions, MarkdownLinkContext, MarkdownPathStrategy};
use crate::model::{ApiDocEntry, ApiDocMember, ApiReturnDoc};
use crate::string_builder::StringBuilder;

pub(super) fn render_callable_member_group_html(
    entry: &ApiDocEntry,
    title: &str,
    members: &[&ApiDocMember],
    options: &MarkdownDocsOptions,
    context: Option<&MarkdownLinkContext<'_>>,
) -> String {
    let mut details = StringBuilder::new();
    for member in members {
        details.push_str("<section id=\"");
        details.push_str(&escape_html(&member_anchor(
            &entry.name,
            member,
            context.map_or(MarkdownPathStrategy::Flat, |context| context.options.path_strategy),
        )));
        details.push_str("\" class=\"ox-api-entry__member-detail\">\n<h5>");
        push_callable_member_heading_html(&mut details, member, title);
        details.push_str("</h5>\n");
        if let Some(signature) = callable_member_signature(member, &entry.name) {
            details.push_str(&render_code_block_html(&signature, "typescript"));
            details.push_char('\n');
        }
        details.push_str(&render_member_detail_description_html(member, context));
        details.push_str(&render_member_detail_type_parameters_html(member, options, context));
        details.push_str(&render_member_detail_params_html(member, options, context));
        if let Some(returns) = &member.returns {
            details.push_str(&render_member_detail_returns_html(returns, options, context));
        } else if member.kind == "constructor" {
            let returns = ApiReturnDoc {
                type_annotation: entry.name.clone(),
                description: String::new(),
                members: Vec::new(),
            };
            details.push_str(&render_member_detail_returns_html(&returns, options, context));
        }
        details.push_str(&render_member_detail_throws_html(member, context));
        details.push_str(&render_implementation_of_html(&member.implementation_of));
        details.push_str("</section>");
    }

    let title = escape_html(title);
    let mut out = StringBuilder::new();
    out.push_str(
        "<div class=\"ox-api-entry__member-group ox-api-entry__member-group--details\">\n<h5>",
    );
    out.push_str(&title);
    out.push_str("</h5>\n<div class=\"ox-api-entry__member-details\">\n");
    out.push_str(&details.into_string());
    out.push_str("\n</div>\n</div>");
    out.into_string()
}

pub(super) fn is_callable_member(member: &ApiDocMember) -> bool {
    matches!(member.kind.as_str(), "constructor" | "method" | "getter" | "setter")
}

fn push_callable_member_heading_html(out: &mut StringBuilder, member: &ApiDocMember, title: &str) {
    if member.kind == "constructor" {
        out.push_str("Constructor");
        return;
    }
    out.push_str(&escape_html(&member.name));
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

pub(super) fn render_member_detail_description_html(
    member: &ApiDocMember,
    context: Option<&MarkdownLinkContext<'_>>,
) -> String {
    let mut out = StringBuilder::new();
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
        out.push_str("<div class=\"ox-api-entry__member-meta\"><span class=\"ox-api-badge ox-api-badge--warning\">deprecated</span></div>");
    }
    if experimental {
        out.push_str("<div class=\"ox-api-entry__member-meta\"><span class=\"ox-api-badge ox-api-badge--warning\">experimental</span></div>");
    }
    if !member.description.is_empty() {
        out.push_str("<div class=\"ox-api-entry__member-description\">");
        out.push_str(&render_doc_inline_html(&member.description, context));
        out.push_str("</div>");
    }
    out.into_string()
}

fn render_member_detail_params_html(
    member: &ApiDocMember,
    options: &MarkdownDocsOptions,
    context: Option<&MarkdownLinkContext<'_>>,
) -> String {
    if member.params.is_empty() {
        return String::new();
    }
    let mut out = StringBuilder::new();
    out.push_str("<div class=\"ox-api-entry__member-detail-section ox-api-entry__member-detail-section--params\">\n<h6>Parameters</h6>\n");
    out.push_str(&render_member_params_html(&member.params, options, context));
    out.push_str("\n</div>");
    out.into_string()
}

fn render_member_detail_type_parameters_html(
    member: &ApiDocMember,
    options: &MarkdownDocsOptions,
    context: Option<&MarkdownLinkContext<'_>>,
) -> String {
    if member.type_parameters.is_empty() {
        return String::new();
    }
    let mut out = StringBuilder::new();
    out.push_str("<div class=\"ox-api-entry__member-detail-section ox-api-entry__member-detail-section--type-parameters\">\n<h6>Type Parameters</h6>\n");
    out.push_str(&render_member_type_parameters_html(&member.type_parameters, options, context));
    out.push_str("\n</div>");
    out.into_string()
}

fn render_member_detail_returns_html(
    returns: &ApiReturnDoc,
    options: &MarkdownDocsOptions,
    context: Option<&MarkdownLinkContext<'_>>,
) -> String {
    let mut out = StringBuilder::new();
    out.push_str("<div class=\"ox-api-entry__member-detail-section ox-api-entry__member-detail-section--returns\">\n<h6>Returns</h6>\n<code class=\"ox-api-entry__return-type\">");
    out.push_str(&render_type_inner_html(&returns.type_annotation, context, &FxHashSet::default()));
    out.push_str("</code>");
    if !returns.description.is_empty() {
        out.push_str("<p class=\"ox-api-entry__return-description\">");
        out.push_str(&render_doc_inline_html(&returns.description, context));
        out.push_str("</p>");
    }
    out.push_str(&render_return_members_html(&returns.members, options, context));
    out.push_str("\n</div>");
    out.into_string()
}

pub(super) fn render_member_detail_throws_html(
    member: &ApiDocMember,
    context: Option<&MarkdownLinkContext<'_>>,
) -> String {
    let throws = super::super::rendered_throws(&member.throws, &member.tags);
    if throws.is_empty() {
        return String::new();
    }
    let mut out = StringBuilder::new();
    out.push_str("<div class=\"ox-api-entry__member-detail-section ox-api-entry__member-detail-section--throws\">\n<h6>Throws</h6>\n");
    out.push_str(&render_throws_list_html(throws.as_ref(), context, "ox-api-entry__member-throws"));
    out.push_str("\n</div>");
    out.into_string()
}

fn render_implementation_of_html(implementation_of: &[String]) -> String {
    if implementation_of.is_empty() {
        return String::new();
    }
    let mut out = StringBuilder::new();
    out.push_str("<div class=\"ox-api-entry__member-detail-section ox-api-entry__member-detail-section--implementation-of\">\n<h6>Implementation of</h6>");
    for implementation in implementation_of {
        out.push_str("<pre><code class=\"language-ts\">");
        out.push_str(&escape_html(implementation.trim()));
        out.push_str("</code></pre>");
    }
    out.push_str("\n</div>");
    out.into_string()
}
