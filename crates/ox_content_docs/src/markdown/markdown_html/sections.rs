use super::*;
use crate::markdown::detail_tags::detail_sections;

pub(super) fn push_heritage_sections_html(
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
pub(super) fn push_type_parameters_html(
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
pub(super) fn push_params_html(
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
pub(super) fn push_returns_html(
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
    body.push_str("</code>");
    if !returns.description.is_empty() {
        body.push_str("\n  <p class=\"ox-api-entry__return-description\">");
        body.push_str(&render_doc_inline_html(&returns.description, link_context));
        body.push_str("</p>");
    }
    let members = render_return_members_html(&returns.members, options, link_context);
    if !members.is_empty() {
        body.push('\n');
        body.push_str(&members);
    }
    body.push_str(
        "
</div>
</div>\n",
    );
}

/// Appends the throws section, or nothing when empty.
pub(super) fn push_throws_html(
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

pub(super) fn push_detail_sections_html(
    body: &mut String,
    tags: &[ApiDocTag],
    link_context: Option<&MarkdownLinkContext<'_>>,
) {
    push_detail_section_blocks(body, tags, link_context, "h4");
}

/// The detail sections of a module page: `h2`, like its examples, and ended by a
/// blank line because Markdown follows them.
pub(in crate::markdown) fn render_module_detail_sections_html(
    tags: &[ApiDocTag],
    link_context: Option<&MarkdownLinkContext<'_>>,
) -> String {
    let mut out = String::new();
    push_detail_section_blocks(&mut out, tags, link_context, "h2");
    if !out.is_empty() {
        out.push('\n');
    }
    out
}

fn push_detail_section_blocks(
    body: &mut String,
    tags: &[ApiDocTag],
    link_context: Option<&MarkdownLinkContext<'_>>,
    heading: &str,
) {
    for section in detail_sections(tags, link_context) {
        body.push_str("<div class=\"ox-api-entry__section ox-api-entry__section--");
        body.push_str(section.modifier);
        body.push_str("\">\n<");
        body.push_str(heading);
        body.push('>');
        body.push_str(section.title);
        body.push_str("</");
        body.push_str(heading);
        body.push_str(">\n");
        body.push_str(&render_detail_blocks_html(&section.markdown));
        body.push_str("\n</div>\n");
    }
}

pub(super) fn push_tag_list_html(
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
