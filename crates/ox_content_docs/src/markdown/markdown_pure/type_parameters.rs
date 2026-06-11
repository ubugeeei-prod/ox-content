use rustc_hash::FxHashSet;

use super::super::{
    effective_parameters_format, MarkdownDisplayFormat, MarkdownDocsOptions, MarkdownLinkContext,
};
use super::format::{inline, push_table_cell, type_param_name_cell, type_param_name_span};
use crate::model::ApiTypeParamDoc;

pub(super) fn type_parameters_have_descriptions(type_parameters: &[ApiTypeParamDoc]) -> bool {
    type_parameters.iter().any(|type_param| !type_param.description.trim().is_empty())
}

/// Appends a `{heading} Type Parameters` section, or nothing when empty.
pub(super) fn push_type_parameters(
    out: &mut String,
    type_parameters: &[ApiTypeParamDoc],
    options: &MarkdownDocsOptions,
    context: Option<&MarkdownLinkContext<'_>>,
    heading: &str,
) {
    if type_parameters.is_empty() {
        return;
    }
    out.push_str(heading);
    out.push_str(" Type Parameters\n\n");
    let skip: FxHashSet<&str> = type_parameters.iter().map(|param| param.name.as_str()).collect();
    match effective_parameters_format(options) {
        MarkdownDisplayFormat::Table => {
            let has_description = type_parameters_have_descriptions(type_parameters);
            if has_description {
                out.push_str("| Name | Description |\n| --- | --- |\n");
            } else {
                out.push_str("| Name |\n| --- |\n");
            }
            for type_param in type_parameters {
                out.push_str("| ");
                out.push_str(&type_param_name_cell(type_param, context, &skip));
                if has_description {
                    out.push_str(" | ");
                    if type_param.description.trim().is_empty() {
                        out.push('-');
                    } else {
                        let description = inline(&type_param.description, context);
                        if description.is_empty() {
                            out.push('-');
                        } else {
                            push_table_cell(out, &description);
                        }
                    }
                }
                out.push_str(" |\n");
            }
        }
        _ => {
            for type_param in type_parameters {
                let description = inline(&type_param.description, context);
                out.push_str("- ");
                out.push_str(&type_param_name_span(type_param, context, &skip));
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
