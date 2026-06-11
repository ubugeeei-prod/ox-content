//! Pure Markdown rendering (no raw HTML) for generated API reference docs.
//!
//! Selected when `MarkdownDocsOptions::render_style` is
//! `MarkdownRenderStyle::Markdown`. This is a child module of `markdown`, so it
//! reuses the parent's extraction/formatting helpers via `super::` and emits the
//! same per-entry information as the HTML renderer — but as Markdown headings,
//! tables and fenced code blocks (no `<details>`, no theme-specific HTML).

use rustc_hash::FxHashSet;

use super::{
    effective_members_format, effective_parameters_format, generate_source_href,
    parse_example_block, process_doc_text, ExampleBlock, MarkdownDisplayFormat,
    MarkdownDocsOptions, MarkdownLinkContext,
};
use crate::model::{
    ApiDocEntry, ApiDocMember, ApiDocTag, ApiParamDoc, ApiReturnDoc, ApiThrowsDoc, ApiTypeParamDoc,
};
use crate::string_builder::{join2, StringBuilder};

mod format;
mod index_signatures;
mod lifecycle;
mod member_details;
mod stats;

use format::{
    code_cell, code_span, inline, linked_type_cell, linked_type_span, push_table_cell,
    type_param_name_cell, type_param_name_span,
};
use index_signatures::{push_index_signature_detail_pure, render_index_signature_group_pure};
pub(super) use lifecycle::push_lifecycle_alerts;
use lifecycle::render_since_section;
use member_details::{is_callable_member, render_callable_member_details_pure};
pub(super) use stats::render_stats_markdown;

fn type_parameters_have_descriptions(type_parameters: &[ApiTypeParamDoc]) -> bool {
    type_parameters.iter().any(|type_param| !type_param.description.trim().is_empty())
}

/// Renders the body of one entry (everything below its heading) as pure Markdown.
///
/// `section_level` is the heading level (number of `#`) for the entry's
/// top-level sections — `2` under a page `# Title` (typedoc per-symbol pages),
/// `4` under a flat `### Entry` heading. Sections are emitted as real Markdown
/// headings (not bold paragraphs) so they appear in the VitePress outline, get
/// anchors, and keep a sequential level hierarchy (markdownlint MD001).
pub(super) fn render_entry_body_pure(
    entry: &ApiDocEntry,
    options: &MarkdownDocsOptions,
    context: Option<&MarkdownLinkContext<'_>>,
    section_level: usize,
) -> String {
    let mut out = String::new();
    let heading = "#".repeat(section_level);

    // Lifecycle tags (`@experimental` / `@deprecated`) render as GitHub alerts
    // near the summary instead of a generic `## Tags` entry.
    push_lifecycle_alerts(&mut out, &entry.tags, context);

    let description = process_doc_text(&entry.description, context);
    let description = description.trim();
    if !description.is_empty() {
        out.push_str(description);
        out.push_str("\n\n");
    }

    // `@since` / `@version` render as a dedicated `## Since` section near the
    // summary instead of a generic `## Tags` entry.
    out.push_str(&render_since_section(&entry.tags, context, &heading));
    push_heritage_sections(&mut out, entry, context, &heading);

    if let Some(signature) = &entry.signature {
        out.push_str(&heading);
        out.push_str(" Signature\n\n```ts\n");
        out.push_str(signature.trim());
        out.push_str("\n```\n\n");
    }

    // Entries with an empty `file` (e.g. symbols re-exported from an external
    // package) have no source in the consumer's repo, so emit no source link.
    if let Some(github_url) = &options.github_url {
        if !entry.file.is_empty() {
            let href = generate_source_href(
                &entry.file,
                github_url,
                Some(entry.line),
                Some(entry.end_line),
            );
            out.push_str("[View source](");
            out.push_str(&href);
            out.push_str(")\n\n");
        }
    }

    push_type_parameters(&mut out, &entry.type_parameters, options, context, &heading);

    if !entry.members.is_empty() {
        out.push_str(&render_members_pure(entry, options, context, section_level));
    }

    push_parameters(&mut out, &entry.params, options, context, &heading);

    if let Some(returns) = &entry.returns {
        push_returns(&mut out, returns, context, &heading);
    }

    let throws = super::rendered_throws(&entry.throws, &entry.tags);
    push_throws(&mut out, throws.as_ref(), context, &heading);

    push_examples(&mut out, &entry.examples, &heading);

    // Structured tags (lifecycle alerts, `## Since`) are rendered above, so
    // exclude them from the generic list here.
    push_generic_tags(&mut out, &entry.tags, context, &heading);

    out.trim_end().to_string()
}

fn push_heritage_sections(
    out: &mut String,
    entry: &ApiDocEntry,
    context: Option<&MarkdownLinkContext<'_>>,
    heading: &str,
) {
    push_heritage_section(out, "Extends", &entry.extends, context, heading);
    push_heritage_section(out, "Implements", &entry.implements, context, heading);
}

fn push_heritage_section(
    out: &mut String,
    title: &str,
    items: &[String],
    context: Option<&MarkdownLinkContext<'_>>,
    heading: &str,
) {
    if items.is_empty() {
        return;
    }
    out.push_str(heading);
    out.push(' ');
    out.push_str(title);
    out.push_str("\n\n");
    for item in items {
        out.push_str("- ");
        out.push_str(&linked_type_span(item, context));
        out.push('\n');
    }
    out.push('\n');
}

/// Renders an overloaded function's symbol page body: an optional symbol-level
/// comment hoisted from the implementation (summary, lifecycle alerts, `## Since`)
/// followed by one `## Call Signature` block per public overload. The
/// implementation signature itself is omitted from the call-signature list,
/// matching TypeDoc. `public` must contain the signatures to render (at least
/// two); `implementation` is the body-carrying entry when present.
pub(super) fn render_overload_body_pure(
    public: &[&ApiDocEntry],
    implementation: Option<&ApiDocEntry>,
    options: &MarkdownDocsOptions,
    context: Option<&MarkdownLinkContext<'_>>,
    section_level: usize,
) -> String {
    let mut out = String::new();
    let heading = "#".repeat(section_level);
    let sub = "#".repeat(section_level + 1);

    // Symbol-level comment, hoisted from the implementation declaration (TypeDoc
    // uses the implementation's doc comment as the symbol comment).
    if let Some(entry) = implementation {
        push_lifecycle_alerts(&mut out, &entry.tags, context);
        let description = process_doc_text(&entry.description, context);
        let description = description.trim();
        if !description.is_empty() {
            out.push_str(description);
            out.push_str("\n\n");
        }
        out.push_str(&render_since_section(&entry.tags, context, &heading));
    }

    // One `## Call Signature` per public overload; its own sections nest at the
    // next heading level (`### Type Parameters` / `### Parameters` / `### Returns`).
    for entry in public {
        out.push_str(&heading);
        out.push_str(" Call Signature\n\n");
        if let Some(signature) = &entry.signature {
            out.push_str("```ts\n");
            out.push_str(signature.trim());
            out.push_str("\n```\n\n");
        }
        push_lifecycle_alerts(&mut out, &entry.tags, context);
        let description = process_doc_text(&entry.description, context);
        let description = description.trim();
        if !description.is_empty() {
            out.push_str(description);
            out.push_str("\n\n");
        }
        out.push_str(&render_since_section(&entry.tags, context, &sub));
        if let Some(github_url) = &options.github_url {
            if !entry.file.is_empty() {
                let href = generate_source_href(
                    &entry.file,
                    github_url,
                    Some(entry.line),
                    Some(entry.end_line),
                );
                out.push_str("[View source](");
                out.push_str(&href);
                out.push_str(")\n\n");
            }
        }
        push_type_parameters(&mut out, &entry.type_parameters, options, context, &sub);
        push_parameters(&mut out, &entry.params, options, context, &sub);
        if let Some(returns) = &entry.returns {
            push_returns(&mut out, returns, context, &sub);
        }
        let throws = super::rendered_throws(&entry.throws, &entry.tags);
        push_throws(&mut out, throws.as_ref(), context, &sub);
        push_examples(&mut out, &entry.examples, &sub);
        push_generic_tags(&mut out, &entry.tags, context, &sub);
    }

    out.trim_end().to_string()
}

/// Appends a `{heading} Type Parameters` section, or nothing when empty.
fn push_type_parameters(
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

/// Appends a `{heading} Parameters` section, or nothing when empty.
fn push_parameters(
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

/// Appends a `{heading} Returns` section for a return doc.
fn push_returns(
    out: &mut String,
    returns: &ApiReturnDoc,
    context: Option<&MarkdownLinkContext<'_>>,
    heading: &str,
) {
    out.push_str(heading);
    out.push_str(" Returns\n\n");
    out.push_str(&linked_type_span(&returns.type_annotation, context));
    if !returns.description.is_empty() {
        out.push_str(" — ");
        out.push_str(&inline(&returns.description, context));
    }
    out.push_str("\n\n");
    push_return_members(out, &returns.members, context, heading);
}

/// Appends a `{heading} Throws` section for exception/error docs.
fn push_throws(
    out: &mut String,
    throws: &[ApiThrowsDoc],
    context: Option<&MarkdownLinkContext<'_>>,
    heading: &str,
) {
    let mut rendered_heading = false;
    for throws_doc in throws {
        let type_annotation =
            throws_doc.type_annotation.as_deref().map(str::trim).filter(|value| !value.is_empty());
        let description = inline(&throws_doc.description, context);
        if type_annotation.is_none() && description.is_empty() {
            continue;
        }
        if !rendered_heading {
            out.push_str(heading);
            out.push_str(" Throws\n\n");
            rendered_heading = true;
        }
        out.push_str("- ");
        if let Some(type_annotation) = type_annotation {
            out.push_str(&linked_type_span(type_annotation, context));
            if !description.is_empty() {
                out.push_str(" — ");
                out.push_str(&description);
            }
        } else {
            out.push_str(&description);
        }
        out.push('\n');
    }
    if rendered_heading {
        out.push('\n');
    }
}

fn push_return_members(
    out: &mut String,
    members: &[ApiDocMember],
    context: Option<&MarkdownLinkContext<'_>>,
    heading: &str,
) {
    if members.is_empty() {
        return;
    }

    let mut rendered_indexable_heading = false;
    for member in members {
        if member.kind == "indexSignature" {
            if !rendered_indexable_heading {
                out.push_str(heading);
                out.push_str("# Indexable\n\n");
                rendered_indexable_heading = true;
            }
            let detail_heading = join2(heading, "##");
            push_index_signature_detail_pure(out, member, context, &detail_heading);
            continue;
        }
        out.push_str(heading);
        out.push('#');
        out.push(' ');
        out.push_str(&member.name);
        out.push_str("\n\n```ts\n");
        out.push_str(&return_member_signature(member));
        out.push_str("\n```\n\n");
        let description = member_description(member, context, true);
        if !description.is_empty() {
            out.push_str(&description);
            out.push_str("\n\n");
        }
    }
}

fn return_member_signature(member: &ApiDocMember) -> String {
    if let Some(signature) = member.signature.as_deref().filter(|signature| !signature.is_empty()) {
        let signature = signature.trim();
        return if signature.ends_with(';') {
            signature.to_string()
        } else {
            let mut out = StringBuilder::with_capacity(signature.len() + 1);
            out.push_str(signature);
            out.push_char(';');
            out.into_string()
        };
    }

    let member_type = member_type(member);
    let member_type = if member_type.is_empty() { "unknown" } else { member_type };
    let mut out = StringBuilder::with_capacity(member.name.len() + member_type.len() + 16);
    if member.readonly {
        out.push_str("readonly ");
    }
    out.push_str(&member.name);
    if member.optional {
        out.push_char('?');
    }
    out.push_str(": ");
    out.push_str(member_type);
    out.push_char(';');
    out.into_string()
}

/// Appends a `{heading} Examples` section, or nothing when empty.
fn push_examples(out: &mut String, examples: &[String], heading: &str) {
    if examples.is_empty() {
        return;
    }
    out.push_str(heading);
    out.push_str(" Examples\n\n");
    for example in examples {
        match parse_example_block(example) {
            ExampleBlock::Code { code, language } => {
                out.push_str("```");
                out.push_str(language);
                out.push('\n');
                out.push_str(code);
                out.push_str("\n```\n\n");
            }
            ExampleBlock::Markdown(markdown) => {
                out.push_str(markdown);
                out.push_str("\n\n");
            }
        }
    }
}

/// Appends a `{heading} Tags` list for non-structured tags, or nothing when none
/// remain after structured tags (lifecycle alerts, `Since`) are excluded.
fn push_generic_tags(
    out: &mut String,
    tags: &[ApiDocTag],
    context: Option<&MarkdownLinkContext<'_>>,
    heading: &str,
) {
    let mut rendered_tags_heading = false;
    for tag in tags {
        if super::is_structured_tag(&tag.tag) {
            continue;
        }
        if !rendered_tags_heading {
            out.push_str(heading);
            out.push_str(" Tags\n\n");
            rendered_tags_heading = true;
        }
        let value = inline(&tag.value, context);
        out.push_str("- `@");
        out.push_str(&tag.tag);
        if value.is_empty() {
            out.push_str("`\n");
        } else {
            out.push_str("` — ");
            out.push_str(&value);
            out.push('\n');
        }
    }
    if rendered_tags_heading {
        out.push('\n');
    }
}

/// Renders the member tables for an entry, grouped to match the HTML renderer.
///
/// Each member group (`Properties`, `Methods`, …) is emitted as a real heading
/// at `section_level` — the same level as the entry's other sections — matching
/// TypeDoc, which renders `## Properties` directly rather than nesting member
/// tables under a separate "Members" heading.
fn render_members_pure(
    entry: &ApiDocEntry,
    options: &MarkdownDocsOptions,
    context: Option<&MarkdownLinkContext<'_>>,
    section_level: usize,
) -> String {
    let mut out = String::new();
    let heading = "#".repeat(section_level);
    let group_context = MemberGroupRenderContext {
        entry_name: &entry.name,
        entry_kind: &entry.kind,
        options,
        link_context: context,
        parameter_section_level: section_level + 1,
    };

    match entry.kind.as_str() {
        "class" => {
            let mut constructors = Vec::new();
            let mut static_methods = Vec::new();
            let mut methods = Vec::new();
            let mut index_signatures = Vec::new();
            let mut static_properties = Vec::new();
            let mut properties = Vec::new();

            for member in &entry.members {
                match member.kind.as_str() {
                    "constructor" => constructors.push(member),
                    "method" | "getter" | "setter" if member.r#static => {
                        static_methods.push(member);
                    }
                    "method" | "getter" | "setter" => methods.push(member),
                    "indexSignature" => index_signatures.push(member),
                    "property" if member.r#static => static_properties.push(member),
                    "property" => properties.push(member),
                    _ => {}
                }
            }
            render_member_group_pure(
                &mut out,
                &heading,
                "Constructors",
                &constructors,
                &group_context,
            );
            render_member_group_pure(
                &mut out,
                &heading,
                "Static Methods",
                &static_methods,
                &group_context,
            );
            render_member_group_pure(&mut out, &heading, "Methods", &methods, &group_context);
            render_index_signature_group_pure(
                &mut out,
                &heading,
                &index_signatures,
                &group_context,
            );
            render_member_group_pure(
                &mut out,
                &heading,
                "Static Properties",
                &static_properties,
                &group_context,
            );
            render_member_group_pure(&mut out, &heading, "Properties", &properties, &group_context);
        }
        "interface" => {
            let mut properties = Vec::new();
            let mut methods = Vec::new();
            let mut index_signatures = Vec::new();

            for member in &entry.members {
                match member.kind.as_str() {
                    "indexSignature" => index_signatures.push(member),
                    "method" | "getter" | "setter" if !member.r#static => methods.push(member),
                    "property" if !member.r#static => properties.push(member),
                    _ => {}
                }
            }
            render_index_signature_group_pure(
                &mut out,
                &heading,
                &index_signatures,
                &group_context,
            );
            render_member_group_pure(&mut out, &heading, "Properties", &properties, &group_context);
            render_member_group_pure(&mut out, &heading, "Methods", &methods, &group_context);
        }
        "type" => {
            let mut properties = Vec::new();
            let mut methods = Vec::new();
            let mut index_signatures = Vec::new();
            let mut enum_members = Vec::new();

            for member in &entry.members {
                match member.kind.as_str() {
                    "indexSignature" => index_signatures.push(member),
                    "method" | "getter" | "setter" if !member.r#static => methods.push(member),
                    "property" if !member.r#static => properties.push(member),
                    "enumMember" => enum_members.push(member),
                    _ => {}
                }
            }
            render_index_signature_group_pure(
                &mut out,
                &heading,
                &index_signatures,
                &group_context,
            );
            render_member_group_pure(&mut out, &heading, "Properties", &properties, &group_context);
            render_member_group_pure(&mut out, &heading, "Methods", &methods, &group_context);
            render_member_group_pure(
                &mut out,
                &heading,
                "Enum Members",
                &enum_members,
                &group_context,
            );
        }
        "enum" => {
            let mut enum_members = Vec::new();

            for member in &entry.members {
                if member.kind == "enumMember" {
                    enum_members.push(member);
                }
            }
            render_member_group_pure(
                &mut out,
                &heading,
                "Enum Members",
                &enum_members,
                &group_context,
            );
        }
        _ => {
            let members = entry.members.iter().collect::<Vec<_>>();
            render_member_group_pure(&mut out, &heading, "Members", &members, &group_context);
        }
    }
    out
}

struct MemberGroupRenderContext<'a, 'ctx> {
    entry_name: &'a str,
    entry_kind: &'a str,
    options: &'a MarkdownDocsOptions,
    link_context: Option<&'a MarkdownLinkContext<'ctx>>,
    parameter_section_level: usize,
}

fn render_member_parameter_sections_pure(
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
            && !member.tags.iter().any(|tag| super::is_throws_tag(&tag.tag))
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

        let throws = super::rendered_throws(&member.throws, &member.tags);
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

fn push_throws_items(
    out: &mut String,
    throws: &[ApiThrowsDoc],
    context: Option<&MarkdownLinkContext<'_>>,
) {
    for throws_doc in throws {
        let type_annotation =
            throws_doc.type_annotation.as_deref().map(str::trim).filter(|value| !value.is_empty());
        let description = inline(&throws_doc.description, context);
        if type_annotation.is_none() && description.is_empty() {
            continue;
        }
        out.push_str("- ");
        if let Some(type_annotation) = type_annotation {
            out.push_str(&linked_type_span(type_annotation, context));
            if !description.is_empty() {
                out.push_str(" — ");
                out.push_str(&description);
            }
        } else {
            out.push_str(&description);
        }
        out.push('\n');
    }
    out.push('\n');
}

fn render_member_group_pure(
    out: &mut String,
    heading: &str,
    title: &str,
    members: &[&ApiDocMember],
    context: &MemberGroupRenderContext<'_, '_>,
) {
    if members.is_empty() {
        return;
    }

    out.push_str(heading);
    out.push(' ');
    out.push_str(title);
    out.push_str("\n\n");
    if members.iter().all(|member| is_callable_member(member)) {
        render_callable_member_details_pure(out, title, members, context);
        return;
    }

    if effective_members_format(context.options, context.entry_kind, title)
        == MarkdownDisplayFormat::List
    {
        for member in members {
            // Append straight into `out`; the row is always emitted, so an
            // intermediate per-member `String` would just be an extra alloc.
            out.push_str("- ");
            out.push_str(&member_name_span(member));
            out.push_str(" `");
            out.push_str(&member.kind);
            out.push('`');
            let member_type = member_type(member);
            if !member_type.is_empty() {
                out.push(' ');
                out.push_str(&linked_type_span(member_type, context.link_context));
            }
            let description = member_description(member, context.link_context, false);
            if !description.is_empty() {
                out.push_str(" - ");
                out.push_str(&description);
            }
            out.push('\n');
        }
    } else {
        let include_kind = super::member_table_includes_kind(title);
        if include_kind {
            out.push_str("| Name | Kind | Type | Description |\n| --- | --- | --- | --- |\n");
        } else {
            out.push_str("| Name | Type | Description |\n| --- | --- | --- |\n");
        }
        for member in members {
            out.push_str("| ");
            out.push_str(&member_name_cell(member));
            out.push_str(" | ");
            if include_kind {
                push_table_cell(out, &member.kind);
                out.push_str(" | ");
            }
            out.push_str(&linked_type_cell(member_type(member), context.link_context));
            out.push_str(" | ");
            push_table_cell(out, &member_description(member, context.link_context, false));
            out.push_str(" |\n");
        }
    }
    out.push('\n');
    out.push_str(&render_member_parameter_sections_pure(
        members,
        context.options,
        context.link_context,
        context.parameter_section_level,
    ));
}

fn member_name_cell(member: &ApiDocMember) -> String {
    member_name(member, code_cell)
}

fn member_name_span(member: &ApiDocMember) -> String {
    member_name(member, code_span)
}

fn member_name(member: &ApiDocMember, code: fn(&str) -> String) -> String {
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

    let name = code(&member.name);
    if flags.is_empty() {
        name
    } else {
        let flags = flags.join(", ");
        let mut out = StringBuilder::with_capacity(name.len() + flags.len() + 5);
        out.push_str(&name);
        out.push_str(" _(");
        out.push_str(&flags);
        out.push_str(")_");
        out.into_string()
    }
}

fn member_type(member: &ApiDocMember) -> &str {
    member
        .signature
        .as_deref()
        .or(member.type_annotation.as_deref())
        .or_else(|| member.returns.as_ref().map(|returns| returns.type_annotation.as_str()))
        .unwrap_or_default()
}

fn member_description(
    member: &ApiDocMember,
    context: Option<&MarkdownLinkContext<'_>>,
    include_returns: bool,
) -> String {
    let mut description = String::new();
    // Lifecycle tags cannot hold a GitHub alert inside a table cell, so surface
    // them as a short bold marker prefix instead.
    let push_part = |description: &mut String, part: &str| {
        if !description.is_empty() {
            description.push(' ');
        }
        description.push_str(part);
    };
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
        push_part(&mut description, "**Deprecated.**");
    }
    if experimental {
        push_part(&mut description, "**Experimental.**");
    }
    if !member.description.is_empty() {
        push_part(&mut description, &inline(&member.description, context));
    }
    if let Some(default_value) =
        member.default_value.as_deref().map(str::trim).filter(|value| !value.is_empty())
    {
        let default = code_span(default_value);
        if !default.is_empty() {
            let mut part = StringBuilder::with_capacity(default.len() + 14);
            part.push_str("**Default:** ");
            part.push_str(&default);
            push_part(&mut description, &part.into_string());
        }
    }
    if include_returns {
        if let Some(returns) = &member.returns {
            if !returns.description.is_empty() {
                push_part(&mut description, "Returns:");
                description.push(' ');
                description.push_str(&inline(&returns.description, context));
            }
        }
    }
    // `@since` / `@version` render inline (TypeDoc shows them in the cell); a
    // GitHub alert or section cannot live inside a table cell.
    let since = member
        .tags
        .iter()
        .filter(|tag| super::SINCE_TAGS.contains(&tag.tag.as_str()))
        .map(|tag| inline(&tag.value, context))
        .filter(|value| !value.is_empty())
        .collect::<Vec<_>>();
    if !since.is_empty() {
        push_part(&mut description, "**Since**");
        description.push(' ');
        description.push_str(&since.join(", "));
    }
    description
}

fn param_description(param: &ApiParamDoc, context: Option<&MarkdownLinkContext<'_>>) -> String {
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
