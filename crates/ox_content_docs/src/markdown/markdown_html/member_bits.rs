use super::*;

pub(super) fn render_member_flags(member: &ApiDocMember) -> String {
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

    let mut html = String::new();
    for flag in flags {
        html.push_str("<span class=\"ox-api-badge\">");
        html.push_str(flag);
        html.push_str("</span>");
    }
    html
}

pub(super) fn render_member_type_html(
    member: &ApiDocMember,
    context: Option<&MarkdownLinkContext<'_>>,
    skip: &FxHashSet<&str>,
) -> String {
    let value = member
        .signature
        .as_deref()
        .or(member.type_annotation.as_deref())
        .or_else(|| member.returns.as_ref().map(|returns| returns.type_annotation.as_str()));

    // Same `<code …>` wrapper as `render_highlighted_inline_code_html`; the inner
    // is byte-identical to `escape_html(value)` when nothing links, so unlinked
    // member types are unchanged and only linked symbols become anchors.
    value.map_or_else(String::new, |value| {
        let mut out = StringBuilder::new();
        out.push_str("<code class=\"ox-api-entry__member-type language-typescript\">");
        out.push_str(&render_type_inner_html(value, context, skip));
        out.push_str("</code>");
        out.into_string()
    })
}

pub(super) fn render_member_description_html(
    member: &ApiDocMember,
    options: &MarkdownDocsOptions,
    context: Option<&MarkdownLinkContext<'_>>,
) -> String {
    let mut blocks = Vec::new();

    // Lifecycle tags can't hold a callout inside a table cell, so surface them as
    // inline badges (matching the markdown renderer's bold markers).
    let mut markers = String::new();
    if member.tags.iter().any(|tag| tag.tag == "deprecated") {
        markers.push_str("<span class=\"ox-api-badge ox-api-badge--warning\">deprecated</span>");
    }
    if member.tags.iter().any(|tag| tag.tag == "experimental") {
        markers.push_str("<span class=\"ox-api-badge ox-api-badge--warning\">experimental</span>");
    }
    if !markers.is_empty() {
        blocks.push(join3("<div class=\"ox-api-entry__member-meta\">", &markers, "</div>"));
    }

    if !member.description.is_empty() {
        blocks.push(join3(
            "<div class=\"ox-api-entry__member-description\">",
            &render_doc_inline_html(&member.description, context),
            "</div>",
        ));
    }

    if let Some(default_value) =
        member.default_value.as_deref().map(str::trim).filter(|value| !value.is_empty())
    {
        blocks.push(render_member_default_html(default_value));
    }

    if !member.type_parameters.is_empty() {
        blocks.push(render_member_type_parameters_html(&member.type_parameters, options, context));
    }

    if !member.params.is_empty() {
        blocks.push(render_member_params_html(&member.params, options, context));
    }

    if let Some(returns) = &member.returns {
        if !returns.description.is_empty() {
            blocks.push(join3(
                "<div class=\"ox-api-entry__member-return\"><span>Returns</span> ",
                &render_doc_inline_html(&returns.description, context),
                "</div>",
            ));
        }
    }

    let throws = super::rendered_throws(&member.throws, &member.tags);
    let throws_inline = render_throws_inline_html(throws.as_ref(), context);
    if !throws_inline.is_empty() {
        blocks.push(join3(
            "<div class=\"ox-api-entry__member-throws\"><span>Throws</span> ",
            &throws_inline,
            "</div>",
        ));
    }

    // `@since` / `@version` rendered inline as a badge (matching the markdown
    // renderer's `**Since**` member marker).
    let since = member
        .tags
        .iter()
        .filter(|tag| super::SINCE_TAGS.contains(&tag.tag.as_str()))
        .map(|tag| tag.value.trim())
        .filter(|value| !value.is_empty())
        .collect::<Vec<_>>()
        .join(", ");
    if !since.is_empty() {
        let mut badge = String::from("<span class=\"ox-api-badge\">since ");
        badge.push_str(&escape_html(&since));
        badge.push_str("</span>");
        blocks.push(join3("<div class=\"ox-api-entry__member-meta\">", &badge, "</div>"));
    }

    blocks.join("")
}

fn render_member_default_html(default_value: &str) -> String {
    let mut out = StringBuilder::with_capacity(default_value.len() + 102);
    out.push_str("<div class=\"ox-api-entry__member-default\"><span>Default</span> ");
    out.push_str("<code class=\"language-typescript\">");
    out.push_str(&escape_html(default_value));
    out.push_str("</code></div>");
    out.into_string()
}
