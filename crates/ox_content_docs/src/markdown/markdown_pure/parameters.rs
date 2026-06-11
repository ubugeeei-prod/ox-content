use super::super::{
    effective_parameters_format, MarkdownDisplayFormat, MarkdownDocsOptions, MarkdownLinkContext,
};
use super::format::{
    code_cell, code_span, inline, linked_type_cell, linked_type_span, push_table_cell,
};
use crate::model::ApiParamDoc;
use crate::string_builder::StringBuilder;

/// Appends a `{heading} Parameters` section, or nothing when empty.
pub(super) fn push_parameters(
    out: &mut String,
    params: &[ApiParamDoc],
    options: &MarkdownDocsOptions,
    context: Option<&MarkdownLinkContext<'_>>,
    heading: &str,
) {
    if params.is_empty() {
        return;
    }
    out.push_str(heading);
    out.push_str(" Parameters\n\n");
    match effective_parameters_format(options) {
        MarkdownDisplayFormat::Table => {
            out.push_str("| Name | Type | Description |\n| --- | --- | --- |\n");
            for param in params {
                out.push_str("| ");
                out.push_str(&code_cell(&param.name));
                out.push_str(" | ");
                out.push_str(&linked_type_cell(&param.type_annotation, context));
                out.push_str(" | ");
                push_table_cell(out, &param_description(param, context));
                out.push_str(" |\n");
            }
        }
        _ => {
            for param in params {
                out.push_str("- ");
                out.push_str(&code_span(&param.name));
                if !param.type_annotation.is_empty() {
                    out.push_str(" (");
                    out.push_str(&linked_type_span(&param.type_annotation, context));
                    out.push(')');
                }
                let description = param_description(param, context);
                if !description.is_empty() {
                    out.push_str(" - ");
                    out.push_str(&description);
                }
                out.push('\n');
            }
        }
    }
    out.push('\n');
}

pub(super) fn param_description(
    param: &ApiParamDoc,
    context: Option<&MarkdownLinkContext<'_>>,
) -> String {
    let mut description = inline(&param.description, context);
    let mut flags = Vec::new();
    if param.optional {
        flags.push("optional".to_string());
    }
    if let Some(default_value) = &param.default_value {
        let mut flag = StringBuilder::with_capacity("default: ".len() + default_value.len());
        flag.push_str("default: ");
        flag.push_str(default_value);
        flags.push(flag.into_string());
    }
    if !flags.is_empty() {
        let flags = flags.join(", ");
        description = if description.is_empty() {
            let mut out = StringBuilder::with_capacity(flags.len() + 2);
            out.push_char('_');
            out.push_str(&flags);
            out.push_char('_');
            out.into_string()
        } else {
            let mut out = StringBuilder::with_capacity(description.len() + flags.len() + 5);
            out.push_str(&description);
            out.push_str(" _(");
            out.push_str(&flags);
            out.push_str(")_");
            out.into_string()
        };
    }
    description
}
