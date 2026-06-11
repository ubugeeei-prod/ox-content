use rustc_hash::FxHashSet;

use super::member_details::{
    render_member_detail_description_html, render_member_detail_throws_html,
};
use super::MarkdownLinkContext;
use super::{escape_html, render_type_inner_html};
use crate::model::ApiDocMember;
use crate::string_builder::StringBuilder;

pub(super) fn render_index_signature_group_html(
    members: &[&ApiDocMember],
    context: Option<&MarkdownLinkContext<'_>>,
) -> String {
    if members.is_empty() {
        return String::new();
    }

    let mut details = StringBuilder::new();
    for member in members {
        details.push_str("<section class=\"ox-api-entry__member-detail ox-api-entry__member-detail--indexable\">\n");
        details.push_str(&render_index_signature_code_block_html(member, context));
        details.push_str(&render_member_detail_description_html(member, context));
        details.push_str(&render_member_detail_throws_html(member, context));
        details.push_str("</section>");
    }

    let mut out = StringBuilder::new();
    out.push_str(
        "<div class=\"ox-api-entry__member-group ox-api-entry__member-group--indexable\">\n<h5>Indexable</h5>\n<div class=\"ox-api-entry__member-details\">\n",
    );
    out.push_str(&details.into_string());
    out.push_str("\n</div>\n</div>");
    out.into_string()
}

pub(super) fn render_index_signature_code_block_html(
    member: &ApiDocMember,
    context: Option<&MarkdownLinkContext<'_>>,
) -> String {
    let mut out = StringBuilder::new();
    out.push_str("<pre><code class=\"language-ts\">");
    push_index_signature_code_html(&mut out, member, context);
    out.push_str("</code></pre>");
    out.into_string()
}

fn push_index_signature_code_html(
    out: &mut StringBuilder,
    member: &ApiDocMember,
    context: Option<&MarkdownLinkContext<'_>>,
) {
    let Some(param) = member.params.first() else {
        if let Some(signature) =
            member.signature.as_deref().filter(|signature| !signature.is_empty())
        {
            out.push_str(&escape_html(signature.trim().trim_end_matches(';').trim_end()));
        }
        return;
    };

    if member.readonly {
        out.push_str("readonly ");
    }
    out.push_char('[');
    out.push_str(&escape_html(&param.name));
    out.push_str(": ");
    out.push_str(&render_type_inner_html(&param.type_annotation, context, &FxHashSet::default()));
    out.push_str("]: ");
    let member_type = member
        .type_annotation
        .as_deref()
        .or_else(|| member.returns.as_ref().map(|returns| returns.type_annotation.as_str()))
        .unwrap_or("unknown");
    out.push_str(&render_type_inner_html(member_type, context, &FxHashSet::default()));
}
