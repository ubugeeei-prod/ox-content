use rustc_hash::FxHashSet;

use super::super::{
    effective_parameters_format, is_throws_tag, rendered_throws, MarkdownDisplayFormat,
    MarkdownDocsOptions, MarkdownLinkContext,
};
use super::format::{
    code_cell, code_span, inline, linked_type_cell, linked_type_span, push_table_cell,
    type_param_name_cell, type_param_name_span,
};
use super::parameters::param_description;
use super::return_members::push_return_members;
use super::sections::push_throws_items;
use super::type_parameters::type_parameters_have_descriptions;
use crate::model::ApiDocMember;

pub(super) fn render_member_parameter_sections_pure(
    members: &[&ApiDocMember],
    options: &MarkdownDocsOptions,
    context: Option<&MarkdownLinkContext<'_>>,
    section_level: usize,
) -> String {
    let mut out = String::new();
    let heading = "#".repeat(section_level);

    for member in members {
        if member.type_parameters.is_empty()
            && member.params.is_empty()
            && member.returns.is_none()
            && member.throws.is_empty()
            && !member.tags.iter().any(|tag| is_throws_tag(&tag.tag))
        {
            continue;
        }

        if !member.type_parameters.is_empty() {
            out.push_str(&heading);
            out.push(' ');
            out.push_str(&member.name);
            out.push_str(" Type Parameters\n\n");
            let skip: FxHashSet<&str> =
                member.type_parameters.iter().map(|param| param.name.as_str()).collect();
            match effective_parameters_format(options) {
                MarkdownDisplayFormat::Table => {
                    let has_description =
                        type_parameters_have_descriptions(&member.type_parameters);
                    if has_description {
                        out.push_str("| Name | Description |\n| --- | --- |\n");
                    } else {
                        out.push_str("| Name |\n| --- |\n");
                    }
                    for type_param in &member.type_parameters {
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
                                    push_table_cell(&mut out, &description);
                                }
                            }
                        }
                        out.push_str(" |\n");
                    }
                }
                _ => {
                    for type_param in &member.type_parameters {
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

        if !member.params.is_empty() {
            out.push_str(&heading);
            out.push(' ');
            out.push_str(&member.name);
            out.push_str(" Parameters\n\n");
            match effective_parameters_format(options) {
                MarkdownDisplayFormat::Table => {
                    out.push_str("| Name | Type | Description |\n| --- | --- | --- |\n");
                    for param in &member.params {
                        out.push_str("| ");
                        out.push_str(&code_cell(&param.name));
                        out.push_str(" | ");
                        out.push_str(&linked_type_cell(&param.type_annotation, context));
                        out.push_str(" | ");
                        push_table_cell(&mut out, &param_description(param, context));
                        out.push_str(" |\n");
                    }
                }
                _ => {
                    for param in &member.params {
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

        if let Some(returns) = &member.returns {
            out.push_str(&heading);
            out.push(' ');
            out.push_str(&member.name);
            out.push_str(" Returns\n\n");
            out.push_str(&linked_type_span(&returns.type_annotation, context));
            if !returns.description.is_empty() {
                out.push_str(" — ");
                out.push_str(&inline(&returns.description, context));
            }
            out.push_str("\n\n");
            push_return_members(&mut out, &returns.members, context, &heading);
        }

        let throws = rendered_throws(&member.throws, &member.tags);
        if !throws.is_empty() {
            out.push_str(&heading);
            out.push(' ');
            out.push_str(&member.name);
            out.push_str(" Throws\n\n");
            push_throws_items(&mut out, throws.as_ref(), context);
        }
    }

    out
}
