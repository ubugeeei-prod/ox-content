use rustc_hash::FxHashSet;

use super::super::{
    effective_parameters_format, MarkdownDisplayFormat, MarkdownDocsOptions, MarkdownLinkContext,
};
use super::inline::{escape_html, render_doc_inline_html, render_type_inner_html};
use crate::model::ApiTypeParamDoc;
use crate::string_builder::{join3, StringBuilder};

fn render_type_parameter_name_html(
    type_param: &ApiTypeParamDoc,
    context: Option<&MarkdownLinkContext<'_>>,
    skip: &FxHashSet<&str>,
) -> String {
    let mut name = StringBuilder::new();
    name.push_str("<code>");
    name.push_str(&escape_html(&type_param.name));
    name.push_str("</code>");
    if let Some(constraint) = &type_param.constraint {
        name.push_str(" <em>extends</em> <code>");
        name.push_str(&render_type_inner_html(constraint, context, skip));
        name.push_str("</code>");
    }
    if let Some(default) = &type_param.default {
        name.push_str(" = <code>");
        name.push_str(&render_type_inner_html(default, context, skip));
        name.push_str("</code>");
    }
    name.into_string()
}

/// Names of all type parameters in a list (never linked inside their own siblings'
/// constraints/defaults).
fn type_parameter_skip_set(type_parameters: &[ApiTypeParamDoc]) -> FxHashSet<&str> {
    type_parameters.iter().map(|param| param.name.as_str()).collect()
}

fn type_parameters_have_descriptions(type_parameters: &[ApiTypeParamDoc]) -> bool {
    type_parameters.iter().any(|type_param| !type_param.description.trim().is_empty())
}

pub(super) fn render_type_parameters_table_html(
    type_parameters: &[ApiTypeParamDoc],
    context: Option<&MarkdownLinkContext<'_>>,
) -> String {
    let skip = type_parameter_skip_set(type_parameters);
    let has_description = type_parameters_have_descriptions(type_parameters);
    let mut rows = StringBuilder::new();
    for type_param in type_parameters {
        if !rows.is_empty() {
            rows.push_char('\n');
        }
        rows.push_str("<tr>\n  <td>");
        rows.push_str(&render_type_parameter_name_html(type_param, context, &skip));
        if has_description {
            rows.push_str("</td>\n  <td>");
            if type_param.description.trim().is_empty() {
                rows.push_char('-');
            } else {
                let description = render_doc_inline_html(&type_param.description, context);
                if description.is_empty() {
                    rows.push_char('-');
                } else {
                    rows.push_str(&description);
                }
            }
        }
        rows.push_str("</td>\n</tr>");
    }
    let rows = rows.into_string();

    let mut out = StringBuilder::with_capacity(rows.len() + 235);
    out.push_str(
        "<div class=\"ox-api-entry__section ox-api-entry__section--type-parameters\">
<h4>Type Parameters</h4>
<table class=\"ox-api-entry__type-parameters-table\">
<thead><tr><th>Name</th>",
    );
    if has_description {
        out.push_str("<th>Description</th>");
    }
    out.push_str(
        "</tr></thead>
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

pub(super) fn render_type_parameters_list_html(
    type_parameters: &[ApiTypeParamDoc],
    context: Option<&MarkdownLinkContext<'_>>,
) -> String {
    let skip = type_parameter_skip_set(type_parameters);
    let mut items = StringBuilder::new();
    for type_param in type_parameters {
        if !items.is_empty() {
            items.push_char('\n');
        }
        items.push_str("<li class=\"ox-api-entry__type-parameter\">\n  <div class=\"ox-api-entry__type-parameter-heading\">");
        items.push_str(&render_type_parameter_name_html(type_param, context, &skip));
        items.push_str("</div>\n  ");
        if !type_param.description.is_empty() {
            items.push_str("<p class=\"ox-api-entry__type-parameter-description\">");
            items.push_str(&render_doc_inline_html(&type_param.description, context));
            items.push_str("</p>");
        }
        items.push_str("\n</li>");
    }
    let items = items.into_string();

    let mut out = StringBuilder::with_capacity(items.len() + 145);
    out.push_str(
        "<div class=\"ox-api-entry__section ox-api-entry__section--type-parameters\">
<h4>Type Parameters</h4>
<ul class=\"ox-api-entry__type-parameters\">
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

pub(super) fn render_member_type_parameters_html(
    type_parameters: &[ApiTypeParamDoc],
    options: &MarkdownDocsOptions,
    context: Option<&MarkdownLinkContext<'_>>,
) -> String {
    let skip = type_parameter_skip_set(type_parameters);
    if effective_parameters_format(options) == MarkdownDisplayFormat::Table {
        let has_description = type_parameters_have_descriptions(type_parameters);
        let mut rows = StringBuilder::new();
        for type_param in type_parameters {
            rows.push_str("<tr><td>");
            rows.push_str(&render_type_parameter_name_html(type_param, context, &skip));
            if has_description {
                rows.push_str("</td><td>");
                if type_param.description.trim().is_empty() {
                    rows.push_char('-');
                } else {
                    let description = render_doc_inline_html(&type_param.description, context);
                    if description.is_empty() {
                        rows.push_char('-');
                    } else {
                        rows.push_str(&description);
                    }
                }
            }
            rows.push_str("</td></tr>");
        }
        let mut table = StringBuilder::new();
        table.push_str(
            "<table class=\"ox-api-entry__type-parameters-table\"><thead><tr><th>Name</th>",
        );
        if has_description {
            table.push_str("<th>Description</th>");
        }
        table.push_str("</tr></thead><tbody>");
        table.push_str(&rows.into_string());
        table.push_str("</tbody></table>");
        return table.into_string();
    }

    let mut items = StringBuilder::new();
    for type_param in type_parameters {
        items.push_str("<li class=\"ox-api-entry__type-parameter\"><div class=\"ox-api-entry__type-parameter-heading\">");
        items.push_str(&render_type_parameter_name_html(type_param, context, &skip));
        items.push_str("</div>");
        if !type_param.description.is_empty() {
            items.push_str("<p class=\"ox-api-entry__type-parameter-description\">");
            items.push_str(&render_doc_inline_html(&type_param.description, context));
            items.push_str("</p>");
        }
        items.push_str("</li>");
    }
    join3("<ul class=\"ox-api-entry__type-parameters\">", &items.into_string(), "</ul>")
}
