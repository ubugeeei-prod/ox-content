//! HTML rendering (raw-HTML-laced Markdown) for generated API reference docs.
//!
//! Selected when `MarkdownDocsOptions::render_style` is `MarkdownRenderStyle::Html`
//! (the default). Child module of `markdown`; reuses the parent's
//! extraction/formatting/link helpers via `super::` and emits the ox-content theme
//! HTML structures (`<details>`, stats, member tables, prose blocks, …).

use rustc_hash::FxHashSet;

use super::{
    clean_summary_text, effective_members_format, effective_parameters_format, entry_anchor,
    format_kind_label, generate_source_href, member_anchor, normalize_signature, process_doc_text,
    MarkdownDisplayFormat, MarkdownDocsOptions, MarkdownLinkContext, MarkdownPathStrategy,
};
use crate::model::{
    ApiDocEntry, ApiDocMember, ApiDocTag, ApiParamDoc, ApiReturnDoc, ApiThrowsDoc, ApiTypeParamDoc,
};
use crate::string_builder::{join3, StringBuilder};

mod blocks;
mod examples;
mod index_signatures;
mod inline;
mod member_details;
mod modules;
mod overview;
mod parameters;
mod stats;
mod tags;
mod throws;
mod type_parameters;

use blocks::render_markdown_blocks_html;
use examples::push_examples_html;
pub(super) use examples::render_module_examples_html;
use index_signatures::{render_index_signature_code_block_html, render_index_signature_group_html};
use inline::{
    escape_html, render_code_block_html, render_doc_inline_html,
    render_highlighted_inline_code_html, render_inline_html, render_type_inner_html,
};
use member_details::{is_callable_member, render_callable_member_group_html};
pub(super) use modules::{
    render_module_index_html, render_module_lifecycle_badges_html, render_module_section_html,
};
use overview::render_entry_badges_html;
use parameters::{render_member_params_html, render_params_list_html, render_params_table_html};
pub(super) use stats::{render_details_controls_html, render_stats_html};
use tags::render_tag_list_html;
use throws::{render_throws_inline_html, render_throws_list_html};
use type_parameters::{
    render_member_type_parameters_html, render_type_parameters_list_html,
    render_type_parameters_table_html,
};

fn render_member_flags(member: &ApiDocMember) -> String {
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

fn render_member_type_html(
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

fn render_member_description_html(
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

fn render_member_table_html(
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

fn render_member_list_html(
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

fn render_members_html(
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

fn render_entry_body_html(
    entry: &ApiDocEntry,
    options: &MarkdownDocsOptions,
    link_context: Option<&MarkdownLinkContext<'_>>,
) -> String {
    let processed_description = process_doc_text(&entry.description, link_context);
    // Entries with an empty `file` (e.g. symbols re-exported from an external
    // package) have no source in the consumer's repo, so emit no source link.
    let source_href =
        options.github_url.as_ref().filter(|_| !entry.file.is_empty()).map(|github_url| {
            generate_source_href(&entry.file, github_url, Some(entry.line), Some(entry.end_line))
        });
    let mut body = String::new();

    if !processed_description.is_empty() {
        body.push_str(&render_markdown_blocks_html(&processed_description));
        body.push('\n');
    }
    push_heritage_sections_html(&mut body, entry, link_context);

    if let Some(signature) = &entry.signature {
        body.push_str(
            "<div class=\"ox-api-entry__section ox-api-entry__section--signature\">
<h4>Signature</h4>
",
        );
        body.push_str(&render_code_block_html(signature, "typescript"));
        body.push_str(
            "
</div>\n",
        );
    }

    if let Some(source_href) = source_href {
        body.push_str(
            "<p class=\"ox-api-entry__source\"><a class=\"ox-api-entry__source-link\" href=\"",
        );
        body.push_str(&escape_html(&source_href));
        body.push_str("\" target=\"_blank\" rel=\"noopener noreferrer\">View source<span class=\"ox-api-entry__source-icon\" aria-hidden=\"true\"></span></a></p>\n");
    }

    push_type_parameters_html(&mut body, &entry.type_parameters, options, link_context);

    if !entry.members.is_empty() {
        body.push_str(&render_members_html(entry, options, link_context));
        body.push('\n');
    }

    push_params_html(&mut body, &entry.params, options, link_context);

    if let Some(returns) = &entry.returns {
        push_returns_html(&mut body, returns, options, link_context);
    }

    push_throws_html(&mut body, &entry.throws, &entry.tags, link_context);

    push_examples_html(&mut body, &entry.examples);

    push_tag_list_html(&mut body, &entry.tags, link_context);

    body.trim().to_string()
}

fn push_heritage_sections_html(
    body: &mut String,
    entry: &ApiDocEntry,
    link_context: Option<&MarkdownLinkContext<'_>>,
) {
    push_heritage_section_html(body, "Extends", &entry.extends, link_context);
    push_heritage_section_html(body, "Implements", &entry.implements, link_context);
}

fn push_heritage_section_html(
    body: &mut String,
    title: &str,
    items: &[String],
    link_context: Option<&MarkdownLinkContext<'_>>,
) {
    if items.is_empty() {
        return;
    }
    body.push_str("<div class=\"ox-api-entry__section ox-api-entry__section--heritage\">\n<h4>");
    body.push_str(&escape_html(title));
    body.push_str("</h4>\n<ul class=\"ox-api-entry__heritage-list\">");
    for item in items {
        body.push_str("<li><code>");
        body.push_str(&render_type_inner_html(item, link_context, &FxHashSet::default()));
        body.push_str("</code></li>");
    }
    body.push_str("</ul>\n</div>\n");
}

/// Appends the type-parameters section (table or list), or nothing when empty.
fn push_type_parameters_html(
    body: &mut String,
    type_parameters: &[ApiTypeParamDoc],
    options: &MarkdownDocsOptions,
    link_context: Option<&MarkdownLinkContext<'_>>,
) {
    if type_parameters.is_empty() {
        return;
    }
    if effective_parameters_format(options) == MarkdownDisplayFormat::List {
        body.push_str(&render_type_parameters_list_html(type_parameters, link_context));
    } else {
        body.push_str(&render_type_parameters_table_html(type_parameters, link_context));
    }
    body.push('\n');
}

/// Appends the parameters section (table or list), or nothing when empty.
fn push_params_html(
    body: &mut String,
    params: &[ApiParamDoc],
    options: &MarkdownDocsOptions,
    link_context: Option<&MarkdownLinkContext<'_>>,
) {
    if params.is_empty() {
        return;
    }
    if effective_parameters_format(options) == MarkdownDisplayFormat::Table {
        body.push_str(&render_params_table_html(params, link_context));
    } else {
        body.push_str(&render_params_list_html(params, link_context));
    }
    body.push('\n');
}

/// Appends the returns section for a return doc.
fn push_returns_html(
    body: &mut String,
    returns: &ApiReturnDoc,
    options: &MarkdownDocsOptions,
    link_context: Option<&MarkdownLinkContext<'_>>,
) {
    body.push_str(
        "<div class=\"ox-api-entry__section ox-api-entry__section--returns\">
<h4>Returns</h4>
<div class=\"ox-api-entry__return\">
  <code class=\"ox-api-entry__return-type\">",
    );
    body.push_str(&render_type_inner_html(
        &returns.type_annotation,
        link_context,
        &FxHashSet::default(),
    ));
    body.push_str(
        "</code>
  ",
    );
    if !returns.description.is_empty() {
        body.push_str("<p class=\"ox-api-entry__return-description\">");
        body.push_str(&render_doc_inline_html(&returns.description, link_context));
        body.push_str("</p>");
    }
    body.push_str(&render_return_members_html(&returns.members, options, link_context));
    body.push_str(
        "
</div>
</div>\n",
    );
}

/// Appends the throws section, or nothing when empty.
fn push_throws_html(
    body: &mut String,
    throws: &[ApiThrowsDoc],
    tags: &[ApiDocTag],
    link_context: Option<&MarkdownLinkContext<'_>>,
) {
    let throws = super::rendered_throws(throws, tags);
    if throws.is_empty() {
        return;
    }
    let list = render_throws_list_html(throws.as_ref(), link_context, "ox-api-entry__throws");
    if list.is_empty() {
        return;
    }
    body.push_str(
        "<div class=\"ox-api-entry__section ox-api-entry__section--throws\">
<h4>Throws</h4>
",
    );
    body.push_str(&list);
    body.push_str(
        "
</div>\n",
    );
}

fn render_return_members_html(
    members: &[ApiDocMember],
    options: &MarkdownDocsOptions,
    link_context: Option<&MarkdownLinkContext<'_>>,
) -> String {
    if members.is_empty() {
        return String::new();
    }

    match options.type_declaration_format {
        MarkdownDisplayFormat::Table => {
            return render_type_declaration_members_table_html(members, link_context);
        }
        MarkdownDisplayFormat::List => {
            return render_type_declaration_members_list_html(members, link_context);
        }
        MarkdownDisplayFormat::None => {}
    }

    let mut rendered = StringBuilder::new();
    rendered.push_str("<div class=\"ox-api-entry__return-members\">");
    for member in members {
        if member.kind == "indexSignature" {
            rendered.push_str("\n<div class=\"ox-api-entry__return-member ox-api-entry__return-member--indexable\">\n<h5>Indexable</h5>\n");
            rendered.push_str(&render_index_signature_code_block_html(member, link_context));
            rendered.push_str("\n</div>");
            continue;
        }
        rendered.push_str("\n<div class=\"ox-api-entry__return-member\">\n<h5>");
        rendered.push_str(&escape_html(&member.name));
        rendered.push_str(
            "</h5>\n<code class=\"ox-api-entry__return-member-type language-typescript\">",
        );
        push_return_member_signature_html(&mut rendered, member, link_context);
        rendered.push_str("</code>");
        let description =
            render_member_description_html(member, &MarkdownDocsOptions::default(), link_context);
        if !description.is_empty() {
            rendered.push_str(&description);
        }
        rendered.push_str("\n</div>");
    }
    rendered.push_str("\n</div>");
    rendered.into_string()
}

fn render_type_declaration_members_table_html(
    members: &[ApiDocMember],
    link_context: Option<&MarkdownLinkContext<'_>>,
) -> String {
    render_nested_members_table_html(members, "ox-api-entry__type-declaration-table", link_context)
}

fn render_type_declaration_members_list_html(
    members: &[ApiDocMember],
    link_context: Option<&MarkdownLinkContext<'_>>,
) -> String {
    render_nested_members_list_html(
        members,
        "ox-api-entry__type-declaration-list",
        "ox-api-entry__type-declaration-member",
        "ox-api-entry__type-declaration-member-heading",
        "ox-api-entry__type-declaration-member-description",
        link_context,
    )
}

fn render_property_members_html(
    members: &[ApiDocMember],
    options: &MarkdownDocsOptions,
    link_context: Option<&MarkdownLinkContext<'_>>,
) -> String {
    if members.is_empty() {
        return String::new();
    }

    match options.property_members_format {
        MarkdownDisplayFormat::Table => render_nested_members_table_html(
            members,
            "ox-api-entry__property-members-table",
            link_context,
        ),
        MarkdownDisplayFormat::List => render_nested_members_list_html(
            members,
            "ox-api-entry__property-members-list",
            "ox-api-entry__property-member",
            "ox-api-entry__property-member-heading",
            "ox-api-entry__property-member-description",
            link_context,
        ),
        MarkdownDisplayFormat::None => String::new(),
    }
}

fn render_nested_members_table_html(
    members: &[ApiDocMember],
    class_name: &str,
    link_context: Option<&MarkdownLinkContext<'_>>,
) -> String {
    let mut rows = StringBuilder::new();
    for member in members {
        if !rows.is_empty() {
            rows.push_char('\n');
        }
        rows.push_str("<tr>\n  <td>");
        rows.push_str(&render_nested_member_name_html(member));
        rows.push_str("</td>\n  <td>");
        rows.push_str(&render_nested_member_type_html(member, link_context));
        rows.push_str("</td>\n  <td>");
        rows.push_str(&render_nested_member_description_html(member, link_context));
        rows.push_str("</td>\n</tr>");
    }

    let mut out = StringBuilder::new();
    out.push_str("<table class=\"");
    out.push_str(class_name);
    out.push_str(
        "\"><thead><tr><th>Name</th><th>Type</th><th>Description</th></tr></thead><tbody>\n",
    );
    out.push_str(&rows.into_string());
    out.push_str("\n</tbody></table>");
    out.into_string()
}

fn render_nested_members_list_html(
    members: &[ApiDocMember],
    list_class_name: &str,
    item_class_name: &str,
    heading_class_name: &str,
    description_class_name: &str,
    link_context: Option<&MarkdownLinkContext<'_>>,
) -> String {
    let mut items = StringBuilder::new();
    for member in members {
        if !items.is_empty() {
            items.push_char('\n');
        }
        items.push_str("<li class=\"");
        items.push_str(item_class_name);
        items.push_str("\">");
        items.push_str("<div class=\"");
        items.push_str(heading_class_name);
        items.push_str("\">");
        items.push_str(&render_nested_member_name_html(member));
        let member_type = render_nested_member_type_html(member, link_context);
        if !member_type.is_empty() {
            items.push_char(' ');
            items.push_str(&member_type);
        }
        items.push_str("</div>");
        let description = render_nested_member_description_html(member, link_context);
        if !description.is_empty() {
            items.push_str("<div class=\"");
            items.push_str(description_class_name);
            items.push_str("\">");
            items.push_str(&description);
            items.push_str("</div>");
        }
        items.push_str("</li>");
    }

    let mut out = StringBuilder::new();
    out.push_str("<ul class=\"");
    out.push_str(list_class_name);
    out.push_str("\">\n");
    out.push_str(&items.into_string());
    out.push_str("\n</ul>");
    out.into_string()
}

fn render_nested_member_name_html(member: &ApiDocMember) -> String {
    let mut out = StringBuilder::new();
    out.push_str("<code>");
    out.push_str(&escape_html(&member.name));
    out.push_str("</code>");
    out.push_str(&render_member_flags(member));
    out.into_string()
}

fn render_nested_member_type_html(
    member: &ApiDocMember,
    link_context: Option<&MarkdownLinkContext<'_>>,
) -> String {
    let Some(member_type) = member
        .type_annotation
        .as_deref()
        .or_else(|| member.returns.as_ref().map(|returns| returns.type_annotation.as_str()))
        .or(member.signature.as_deref())
        .filter(|value| !value.is_empty())
    else {
        return String::new();
    };

    let mut out = StringBuilder::new();
    out.push_str("<code class=\"ox-api-entry__member-type language-typescript\">");
    out.push_str(&render_type_inner_html(member_type, link_context, &FxHashSet::default()));
    out.push_str("</code>");
    out.into_string()
}

fn render_nested_member_description_html(
    member: &ApiDocMember,
    link_context: Option<&MarkdownLinkContext<'_>>,
) -> String {
    render_member_description_html(member, &MarkdownDocsOptions::default(), link_context)
}

fn push_return_member_signature_html(
    out: &mut StringBuilder,
    member: &ApiDocMember,
    link_context: Option<&MarkdownLinkContext<'_>>,
) {
    if let Some(signature) = member.signature.as_deref().filter(|signature| !signature.is_empty()) {
        out.push_str(&escape_html(signature.trim()));
        if !signature.trim_end().ends_with(';') {
            out.push_char(';');
        }
        return;
    }

    if member.readonly {
        out.push_str("readonly ");
    }
    out.push_str(&escape_html(&member.name));
    if member.optional {
        out.push_char('?');
    }
    out.push_str(": ");
    let member_type = member
        .type_annotation
        .as_deref()
        .or_else(|| member.returns.as_ref().map(|returns| returns.type_annotation.as_str()))
        .unwrap_or("unknown");
    out.push_str(&render_type_inner_html(member_type, link_context, &FxHashSet::default()));
    out.push_char(';');
}

/// Appends the generic tags section, excluding structured tags (lifecycle / since)
/// which are surfaced as badges instead. Emits nothing when no tags remain.
fn push_tag_list_html(
    body: &mut String,
    tags: &[ApiDocTag],
    link_context: Option<&MarkdownLinkContext<'_>>,
) {
    if tags.iter().all(|tag| super::is_structured_tag(&tag.tag)) {
        return;
    }
    body.push_str(&render_tag_list_html(tags, link_context));
    body.push('\n');
}

pub(super) fn render_entry_html(
    entry: &ApiDocEntry,
    options: &MarkdownDocsOptions,
    link_context: Option<&MarkdownLinkContext<'_>>,
) -> String {
    let processed_description = process_doc_text(&entry.description, link_context);
    let summary_signature = normalize_signature(entry.signature.as_deref());
    let body = render_entry_body_html(entry, options, link_context);

    let summary_description = clean_summary_text(
        &processed_description,
        if summary_signature.is_some() { 80 } else { 120 },
    );
    let summary_heading = if let Some(summary_signature) = summary_signature {
        render_highlighted_inline_code_html(
            &summary_signature,
            "ox-api-entry__signature ox-api-entry__signature--highlighted",
            "typescript",
        )
    } else {
        join3("<code class=\"ox-api-entry__name\">", &escape_html(&entry.name), "</code>")
    };
    let summary_description = if summary_description.is_empty() {
        String::new()
    } else {
        join3(
            "<span class=\"ox-api-entry__description\">",
            &render_inline_html(&summary_description),
            "</span>",
        )
    };
    let badges = render_entry_badges_html(entry, "ox-api-entry__meta");
    let kind = escape_html(format_kind_label(&entry.kind));
    let mut summary = StringBuilder::with_capacity(
        kind.len() + summary_heading.len() + summary_description.len() + badges.len() + 92,
    );
    summary.push_str("<span class=\"ox-api-entry__kind\">");
    summary.push_str(&kind);
    summary.push_str("</span><span class=\"ox-api-entry__summary-main\">");
    summary.push_str(&summary_heading);
    summary.push_str(&summary_description);
    summary.push_str(&badges);
    summary.push_str("</span>");
    let summary = summary.into_string();
    let anchor = entry_anchor(&entry.name);

    let mut out = StringBuilder::with_capacity(anchor.len() + summary.len() + body.len() + 120);
    out.push_str("<details id=\"");
    out.push_str(&anchor);
    out.push_str(
        "\" class=\"ox-api-entry\">
  <summary>",
    );
    out.push_str(&summary);
    out.push_str(
        "</summary>
  <div class=\"ox-api-entry__body\">
",
    );
    out.push_str(&body);
    out.push_str(
        "
  </div>
</details>

",
    );
    out.into_string()
}

pub(super) fn render_entry_page_html(
    entry: &ApiDocEntry,
    options: &MarkdownDocsOptions,
    link_context: Option<&MarkdownLinkContext<'_>>,
) -> String {
    let body = render_entry_body_html(entry, options, link_context);
    // A per-symbol page has no `<summary>`, so structured tags (lifecycle / since)
    // would otherwise be invisible once excluded from the generic tag list. Surface
    // them as a badge row at the top of the page instead.
    let badges = render_entry_badges_html(entry, "ox-api-entry__meta");
    let anchor = entry_anchor(&entry.name);
    let mut out = StringBuilder::with_capacity(anchor.len() + badges.len() + body.len() + 80);
    out.push_str("<div id=\"");
    out.push_str(&anchor);
    out.push_str(
        "\" class=\"ox-api-entry ox-api-entry--page\">
",
    );
    if !badges.is_empty() {
        out.push_str(&badges);
        out.push_char('\n');
    }
    out.push_str(&body);
    out.push_str(
        "
</div>
",
    );
    out.into_string()
}

/// Renders an overloaded function's symbol page body in HTML: a symbol-level badge
/// row + comment hoisted from the implementation, then one `Call Signature` section
/// per public overload. The implementation signature is omitted (TypeDoc parity).
pub(super) fn render_overload_body_html(
    public: &[&ApiDocEntry],
    implementation: Option<&ApiDocEntry>,
    options: &MarkdownDocsOptions,
    link_context: Option<&MarkdownLinkContext<'_>>,
) -> String {
    // Symbol-level badges/comment come from the implementation when present;
    // otherwise fall back to the first public signature.
    let symbol = implementation.or_else(|| public.first().copied());
    let anchor = symbol.map(|entry| entry_anchor(&entry.name)).unwrap_or_default();

    let mut out = String::new();
    out.push_str("<div id=\"");
    out.push_str(&anchor);
    out.push_str("\" class=\"ox-api-entry ox-api-entry--page\">\n");

    if let Some(symbol) = symbol {
        let badges = render_entry_badges_html(symbol, "ox-api-entry__meta");
        if !badges.is_empty() {
            out.push_str(&badges);
            out.push('\n');
        }
    }
    if let Some(implementation) = implementation {
        let description = process_doc_text(&implementation.description, link_context);
        if !description.is_empty() {
            out.push_str(&render_markdown_blocks_html(&description));
            out.push('\n');
        }
    }

    for signature in public {
        out.push_str(
            "<div class=\"ox-api-entry__section ox-api-entry__section--call-signature\">
<h4>Call Signature</h4>
",
        );
        if let Some(code) = &signature.signature {
            out.push_str(&render_code_block_html(code, "typescript"));
            out.push('\n');
        }
        let description = process_doc_text(&signature.description, link_context);
        if !description.is_empty() {
            out.push_str(&render_markdown_blocks_html(&description));
            out.push('\n');
        }
        push_type_parameters_html(&mut out, &signature.type_parameters, options, link_context);
        push_params_html(&mut out, &signature.params, options, link_context);
        if let Some(returns) = &signature.returns {
            push_returns_html(&mut out, returns, options, link_context);
        }
        push_throws_html(&mut out, &signature.throws, &signature.tags, link_context);
        push_examples_html(&mut out, &signature.examples);
        push_tag_list_html(&mut out, &signature.tags, link_context);
        out.push_str("</div>\n");
    }

    out.push_str("</div>\n");
    out
}
