use rustc_hash::FxHashSet;

use super::super::{
    effective_parameters_format, MarkdownDisplayFormat, MarkdownDocsOptions, MarkdownLinkContext,
};
use super::inline::{escape_html, render_doc_inline_html, render_type_inner_html};
use crate::model::ApiParamDoc;
use crate::string_builder::{join3, StringBuilder};

pub(super) fn render_params_list_html(
    params: &[ApiParamDoc],
    context: Option<&MarkdownLinkContext<'_>>,
) -> String {
    let mut rows = StringBuilder::new();
    for param in params {
        if !rows.is_empty() {
            rows.push_char('\n');
        }
        let description = render_member_param_description(param);
        rows.push_str("<li class=\"ox-api-entry__param\">\n  <div class=\"ox-api-entry__param-heading\">\n    <code class=\"ox-api-entry__param-name\">");
        rows.push_str(&escape_html(&param.name));
        rows.push_str("</code>\n    <code class=\"ox-api-entry__param-type\">");
        rows.push_str(&render_type_inner_html(
            &param.type_annotation,
            context,
            &FxHashSet::default(),
        ));
        rows.push_str("</code>\n  </div>\n  ");
        if !description.is_empty() {
            rows.push_str("<p class=\"ox-api-entry__param-description\">");
            rows.push_str(&render_doc_inline_html(&description, context));
            rows.push_str("</p>");
        }
        rows.push_str("\n</li>");
    }
    let rows = rows.into_string();

    let mut out = StringBuilder::with_capacity(rows.len() + 125);
    out.push_str(
        "<div class=\"ox-api-entry__section ox-api-entry__section--params\">
<h4>Parameters</h4>
<ul class=\"ox-api-entry__params\">
",
    );
    out.push_str(&rows);
    out.push_str(
        "
</ul>
</div>",
    );
    out.into_string()
}

pub(super) fn render_params_table_html(
    params: &[ApiParamDoc],
    context: Option<&MarkdownLinkContext<'_>>,
) -> String {
    let mut rows = StringBuilder::new();
    for param in params {
        if !rows.is_empty() {
            rows.push_char('\n');
        }
        let description = render_member_param_description(param);
        rows.push_str("<tr>\n  <td><code>");
        rows.push_str(&escape_html(&param.name));
        rows.push_str("</code></td>\n  <td><code>");
        rows.push_str(&render_type_inner_html(
            &param.type_annotation,
            context,
            &FxHashSet::default(),
        ));
        rows.push_str("</code></td>\n  <td>");
        rows.push_str(&render_doc_inline_html(&description, context));
        rows.push_str("</td>\n</tr>");
    }
    let rows = rows.into_string();

    let mut out = StringBuilder::with_capacity(rows.len() + 220);
    out.push_str(
        "<div class=\"ox-api-entry__section ox-api-entry__section--params\">
<h4>Parameters</h4>
<table class=\"ox-api-entry__params-table\">
<thead><tr><th>Name</th><th>Type</th><th>Description</th></tr></thead>
<tbody>
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

pub(super) fn render_member_param_description(param: &ApiParamDoc) -> String {
    let mut description = param.description.clone();
    let mut flags = String::new();
    if param.optional {
        flags.push_str("optional");
    }
    if let Some(default_value) = &param.default_value {
        if !flags.is_empty() {
            flags.push_str(" · ");
        }
        flags.push_str("default: ");
        flags.push_str(default_value);
    }
    if !flags.is_empty() {
        if !description.is_empty() {
            description.push_str(" — ");
        }
        description.push_str(&flags);
    }
    description
}

pub(super) fn render_member_params_html(
    params: &[ApiParamDoc],
    options: &MarkdownDocsOptions,
    context: Option<&MarkdownLinkContext<'_>>,
) -> String {
    if effective_parameters_format(options) == MarkdownDisplayFormat::Table {
        let mut rows = StringBuilder::new();
        for param in params {
            let description = render_member_param_description(param);
            rows.push_str("<tr><td><code>");
            rows.push_str(&escape_html(&param.name));
            rows.push_str("</code></td><td><code>");
            rows.push_str(&render_type_inner_html(
                &param.type_annotation,
                context,
                &FxHashSet::default(),
            ));
            rows.push_str("</code></td><td>");
            rows.push_str(&render_doc_inline_html(&description, context));
            rows.push_str("</td></tr>");
        }
        let rows = rows.into_string();

        return join3(
            "<table class=\"ox-api-entry__member-params-table\"><thead><tr><th>Name</th><th>Type</th><th>Description</th></tr></thead><tbody>",
            &rows,
            "</tbody></table>",
        );
    }

    let mut items = StringBuilder::new();
    for param in params {
        let description = render_member_param_description(param);
        items.push_str("<li><code>");
        items.push_str(&escape_html(&param.name));
        items.push_str("</code>");
        if !description.is_empty() {
            items.push_char(' ');
            items.push_str(&render_doc_inline_html(&description, context));
        }
        items.push_str("</li>");
    }
    join3("<ul class=\"ox-api-entry__member-params\">", &items.into_string(), "</ul>")
}
