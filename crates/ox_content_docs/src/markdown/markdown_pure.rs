//! Pure Markdown rendering (no raw HTML) for generated API reference docs.
//!
//! Selected when `MarkdownDocsOptions::render_style` is
//! `MarkdownRenderStyle::Markdown`. This is a child module of `markdown`, so it
//! reuses the parent's extraction/formatting helpers via `super::` and emits the
//! same per-entry information as the HTML renderer — but as Markdown headings,
//! tables and fenced code blocks (no `<details>`, no theme-specific HTML).

use super::{
    effective_members_format, effective_parameters_format, generate_source_href,
    parse_example_block, process_doc_text, EntryStats, MarkdownDisplayFormat, MarkdownDocsOptions,
    MarkdownLinkContext,
};
use crate::model::{ApiDocEntry, ApiDocMember, ApiDocTag, ApiParamDoc, ApiTypeParamDoc};
use crate::string_builder::StringBuilder;

/// JSDoc lifecycle tags rendered as GitHub alerts rather than generic `## Tags`
/// entries: `@experimental` → `> [!WARNING]`, `@deprecated` → `> [!CAUTION]`.
/// Appends GitHub alert blocks for lifecycle tags (`@experimental`,
/// `@deprecated`) present in `tags`, in source order. Uses the tag's own text as
/// the alert body (with `{@link}` resolved), falling back to a default message.
pub(super) fn push_lifecycle_alerts(
    out: &mut String,
    tags: &[ApiDocTag],
    context: Option<&MarkdownLinkContext<'_>>,
) {
    for tag in tags {
        let (kind, default) = match tag.tag.as_str() {
            "deprecated" => {
                ("CAUTION", "This API is deprecated and may be removed in a future version.")
            }
            "experimental" => {
                ("WARNING", "This API is experimental and may change in future versions.")
            }
            _ => continue,
        };
        let body_storage;
        let body = if tag.value.trim().is_empty() {
            default
        } else {
            body_storage = inline(&tag.value, context);
            if body_storage.is_empty() {
                default
            } else {
                body_storage.as_str()
            }
        };
        out.push_str("> [!");
        out.push_str(kind);
        out.push_str("]\n");
        for line in body.lines() {
            out.push_str("> ");
            out.push_str(line);
            out.push('\n');
        }
        out.push('\n');
    }
}

fn is_lifecycle_tag(tag: &str) -> bool {
    matches!(tag, "deprecated" | "experimental")
}

/// Renders the per-page stats summary as a single italic Markdown line.
pub(super) fn render_stats_markdown(stats: &EntryStats, module_count: Option<usize>) -> String {
    let mut out = StringBuilder::new();
    let mut has_parts = false;
    out.push_char('_');
    if let Some(module_count) = module_count {
        push_stat_part(&mut out, &mut has_parts, module_count, "modules");
    }
    push_stat_part(&mut out, &mut has_parts, stats.entries, "symbols");
    for (index, kind) in super::DOC_KIND_ORDER.iter().enumerate() {
        let count = stats.by_kind[index];
        if count > 0 {
            push_stat_part(&mut out, &mut has_parts, count, super::doc_kind_plural(kind));
        }
    }
    if stats.params > 0 {
        push_stat_part(&mut out, &mut has_parts, stats.params, "parameters");
    }
    if stats.members > 0 {
        push_stat_part(&mut out, &mut has_parts, stats.members, "members");
    }
    if stats.returns > 0 {
        push_stat_part(&mut out, &mut has_parts, stats.returns, "returns");
    }
    if stats.examples > 0 {
        push_stat_part(&mut out, &mut has_parts, stats.examples, "examples");
    }
    if stats.deprecated > 0 {
        push_stat_part(&mut out, &mut has_parts, stats.deprecated, "deprecated");
    }
    out.push_char('_');
    out.into_string()
}

fn push_stat_part(out: &mut StringBuilder, has_parts: &mut bool, count: usize, label: &str) {
    if *has_parts {
        out.push_str(" · ");
    }
    out.push_usize(count);
    out.push_char(' ');
    out.push_str(label);
    *has_parts = true;
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

    if !entry.type_parameters.is_empty() {
        out.push_str(&heading);
        out.push_str(" Type Parameters\n\n");
        match effective_parameters_format(options) {
            MarkdownDisplayFormat::Table => {
                out.push_str("| Name | Description |\n| --- | --- |\n");
                for type_param in &entry.type_parameters {
                    out.push_str("| ");
                    out.push_str(&type_param_name_cell(type_param));
                    out.push_str(" | ");
                    out.push_str(&table_cell(&inline(&type_param.description, context)));
                    out.push_str(" |\n");
                }
            }
            _ => {
                for type_param in &entry.type_parameters {
                    let description = inline(&type_param.description, context);
                    out.push_str("- ");
                    out.push_str(&type_param_name_span(type_param));
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

    if !entry.members.is_empty() {
        out.push_str(&render_members_pure(entry, options, context, section_level));
    }

    if !entry.params.is_empty() {
        out.push_str(&heading);
        out.push_str(" Parameters\n\n");
        match effective_parameters_format(options) {
            MarkdownDisplayFormat::Table => {
                out.push_str("| Name | Type | Description |\n| --- | --- | --- |\n");
                for param in &entry.params {
                    out.push_str("| ");
                    out.push_str(&code_cell(&param.name));
                    out.push_str(" | ");
                    out.push_str(&code_cell(&param.type_annotation));
                    out.push_str(" | ");
                    out.push_str(&table_cell(&param_description(param, context)));
                    out.push_str(" |\n");
                }
            }
            _ => {
                for param in &entry.params {
                    let mut line = String::new();
                    line.push_str("- ");
                    line.push_str(&code_span(&param.name));
                    if !param.type_annotation.is_empty() {
                        line.push_str(" (");
                        line.push_str(&code_span(&param.type_annotation));
                        line.push(')');
                    }
                    let description = param_description(param, context);
                    if !description.is_empty() {
                        line.push_str(" - ");
                        line.push_str(&description);
                    }
                    out.push_str(&line);
                    out.push('\n');
                }
            }
        }
        out.push('\n');
    }

    if let Some(returns) = &entry.returns {
        out.push_str(&heading);
        out.push_str(" Returns\n\n");
        out.push_str(&code_cell(&returns.type_annotation));
        if !returns.description.is_empty() {
            out.push_str(" — ");
            out.push_str(&inline(&returns.description, context));
        }
        out.push_str("\n\n");
    }

    if !entry.examples.is_empty() {
        out.push_str(&heading);
        out.push_str(" Examples\n\n");
        for example in &entry.examples {
            let (code, language) = parse_example_block(example);
            out.push_str("```");
            out.push_str(language);
            out.push('\n');
            out.push_str(code);
            out.push_str("\n```\n\n");
        }
    }

    // Lifecycle tags are rendered as alerts above, so exclude them here.
    let mut rendered_tags_heading = false;
    for tag in &entry.tags {
        if is_lifecycle_tag(&tag.tag) {
            continue;
        }
        if !rendered_tags_heading {
            out.push_str(&heading);
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

    out.trim_end().to_string()
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
            let mut static_properties = Vec::new();
            let mut properties = Vec::new();

            for member in &entry.members {
                match member.kind.as_str() {
                    "constructor" => constructors.push(member),
                    "method" | "getter" | "setter" if member.r#static => {
                        static_methods.push(member);
                    }
                    "method" | "getter" | "setter" => methods.push(member),
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

            for member in &entry.members {
                match member.kind.as_str() {
                    "method" | "getter" | "setter" if !member.r#static => methods.push(member),
                    "property" if !member.r#static => properties.push(member),
                    _ => {}
                }
            }
            render_member_group_pure(&mut out, &heading, "Properties", &properties, &group_context);
            render_member_group_pure(&mut out, &heading, "Methods", &methods, &group_context);
        }
        "type" => {
            let mut properties = Vec::new();
            let mut methods = Vec::new();
            let mut enum_members = Vec::new();

            for member in &entry.members {
                match member.kind.as_str() {
                    "method" | "getter" | "setter" if !member.r#static => methods.push(member),
                    "property" if !member.r#static => properties.push(member),
                    "enumMember" => enum_members.push(member),
                    _ => {}
                }
            }
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
        if member.params.is_empty() {
            continue;
        }

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
                    out.push_str(&code_cell(&param.type_annotation));
                    out.push_str(" | ");
                    out.push_str(&table_cell(&param_description(param, context)));
                    out.push_str(" |\n");
                }
            }
            _ => {
                for param in &member.params {
                    let mut line = String::new();
                    line.push_str("- ");
                    line.push_str(&code_span(&param.name));
                    if !param.type_annotation.is_empty() {
                        line.push_str(" (");
                        line.push_str(&code_span(&param.type_annotation));
                        line.push(')');
                    }
                    let description = param_description(param, context);
                    if !description.is_empty() {
                        line.push_str(" - ");
                        line.push_str(&description);
                    }
                    out.push_str(&line);
                    out.push('\n');
                }
            }
        }
        out.push('\n');
    }

    out
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
    if effective_members_format(context.options, context.entry_kind, title)
        == MarkdownDisplayFormat::List
    {
        for member in members {
            let mut line = String::new();
            line.push_str("- ");
            line.push_str(&member_name_span(member));
            line.push_str(" `");
            line.push_str(&member.kind);
            line.push('`');
            let member_type = member_type(member);
            if !member_type.is_empty() {
                line.push(' ');
                line.push_str(&code_span(member_type));
            }
            let description = member_description(member, context.link_context);
            if !description.is_empty() {
                line.push_str(" - ");
                line.push_str(&description);
            }
            out.push_str(&line);
            out.push('\n');
        }
    } else {
        out.push_str("| Name | Kind | Type | Description |\n| --- | --- | --- | --- |\n");
        for member in members {
            out.push_str("| ");
            out.push_str(&member_name_cell(member));
            out.push_str(" | ");
            out.push_str(&table_cell(&member.kind));
            out.push_str(" | ");
            out.push_str(&code_cell(member_type(member)));
            out.push_str(" | ");
            out.push_str(&table_cell(&member_description(member, context.link_context)));
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

fn member_description(member: &ApiDocMember, context: Option<&MarkdownLinkContext<'_>>) -> String {
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
    if let Some(returns) = &member.returns {
        if !returns.description.is_empty() {
            push_part(&mut description, "Returns:");
            description.push(' ');
            description.push_str(&inline(&returns.description, context));
        }
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

/// Collapses runs of whitespace (including newlines) into single spaces.
fn collapse_whitespace(text: &str) -> String {
    let text = text.trim();
    if text.is_empty() {
        return String::new();
    }
    if !text.chars().any(char::is_whitespace) {
        return text.to_string();
    }

    let mut out = String::with_capacity(text.len());
    let mut pending_space = false;
    for ch in text.chars() {
        if ch.is_whitespace() {
            pending_space = !out.is_empty();
        } else {
            if pending_space {
                out.push(' ');
                pending_space = false;
            }
            out.push(ch);
        }
    }
    out
}

/// Inline Markdown for a doc-text fragment (resolves `{@link}`), single-line.
fn inline(text: &str, context: Option<&MarkdownLinkContext<'_>>) -> String {
    collapse_whitespace(&process_doc_text(text, context))
}

/// Escapes a value for use inside a Markdown table cell.
fn table_cell(text: &str) -> String {
    if text.contains('|') {
        text.replace('|', "\\|")
    } else {
        text.to_string()
    }
}

/// Inline code for normal Markdown text; empty string if blank.
fn code_span(value: &str) -> String {
    let value = collapse_whitespace(value);
    if value.is_empty() {
        String::new()
    } else {
        let mut code = StringBuilder::with_capacity(value.len() + 2);
        code.push_char('`');
        code.push_str(&value);
        code.push_char('`');
        code.into_string()
    }
}

/// Inline code for a Markdown table cell (`|` escaped); empty string if blank.
fn code_cell(value: &str) -> String {
    let value = collapse_whitespace(value);
    if value.is_empty() {
        String::new()
    } else {
        let cell = table_cell(&value);
        let mut code = StringBuilder::with_capacity(cell.len() + 2);
        code.push_char('`');
        code.push_str(&cell);
        code.push_char('`');
        code.into_string()
    }
}

/// Builds the Name cell for a type parameter: `` `T` `` plus optional `*extends*`
/// constraint and `=` default, each rendered as inline code.
fn type_param_name_cell(type_param: &ApiTypeParamDoc) -> String {
    type_param_name(type_param, code_cell)
}

fn type_param_name_span(type_param: &ApiTypeParamDoc) -> String {
    type_param_name(type_param, code_span)
}

fn type_param_name(type_param: &ApiTypeParamDoc, code: fn(&str) -> String) -> String {
    let mut cell = code(&type_param.name);
    if let Some(constraint) = &type_param.constraint {
        cell.push_str(" *extends* ");
        cell.push_str(&code(constraint));
    }
    if let Some(default) = &type_param.default {
        cell.push_str(" = ");
        cell.push_str(&code(default));
    }
    cell
}
