//! Pure Markdown rendering (no raw HTML) for generated API reference docs.
//!
//! Selected when `MarkdownDocsOptions::render_style` is
//! `MarkdownRenderStyle::Markdown`. This is a child module of `markdown`, so it
//! reuses the parent's extraction/formatting helpers via `super::` and emits the
//! same per-entry information as the HTML renderer — but as Markdown headings,
//! tables and fenced code blocks (no `<details>`, no theme-specific HTML).

use std::collections::HashSet;

use super::{
    collapse_inline_whitespace, collapse_type_annotation_whitespace, effective_members_format,
    effective_parameters_format, generate_source_href, parse_example_block, process_doc_text,
    resolve_type_fragments, EntryStats, ExampleBlock, MarkdownDisplayFormat, MarkdownDocsOptions,
    MarkdownLinkContext, TypeFragment,
};
use crate::model::{
    ApiDocEntry, ApiDocMember, ApiDocTag, ApiParamDoc, ApiReturnDoc, ApiTypeParamDoc,
};
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

/// Renders a `## Since` section from `@since` / `@version` tags (both folded into
/// one section, matching TypeDoc), or "" when none carry a value. `heading` is
/// the section prefix (`##` on typedoc per-symbol pages).
fn render_since_section(
    tags: &[ApiDocTag],
    context: Option<&MarkdownLinkContext<'_>>,
    heading: &str,
) -> String {
    let values = tags
        .iter()
        .filter(|tag| super::SINCE_TAGS.contains(&tag.tag.as_str()))
        .map(|tag| inline(&tag.value, context))
        .filter(|value| !value.is_empty())
        .collect::<Vec<_>>();
    if values.is_empty() {
        return String::new();
    }
    let mut out = String::new();
    out.push_str(heading);
    out.push_str(" Since\n\n");
    out.push_str(&values.join("\n\n"));
    out.push_str("\n\n");
    out
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

    // `@since` / `@version` render as a dedicated `## Since` section near the
    // summary instead of a generic `## Tags` entry.
    out.push_str(&render_since_section(&entry.tags, context, &heading));

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

    push_examples(&mut out, &entry.examples, &heading);

    // Structured tags (lifecycle alerts, `## Since`) are rendered above, so
    // exclude them from the generic list here.
    push_generic_tags(&mut out, &entry.tags, context, &heading);

    out.trim_end().to_string()
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
    let skip: HashSet<&str> = type_parameters.iter().map(|param| param.name.as_str()).collect();
    match effective_parameters_format(options) {
        MarkdownDisplayFormat::Table => {
            out.push_str("| Name | Description |\n| --- | --- |\n");
            for type_param in type_parameters {
                out.push_str("| ");
                out.push_str(&type_param_name_cell(type_param, context, &skip));
                out.push_str(" | ");
                push_table_cell(out, &inline(&type_param.description, context));
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
    out.push_str(&linked_type_cell(&returns.type_annotation, context));
    if !returns.description.is_empty() {
        out.push_str(" — ");
        out.push_str(&inline(&returns.description, context));
    }
    out.push_str("\n\n");
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
            let description = member_description(member, context.link_context);
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
            push_table_cell(out, &member_description(member, context.link_context));
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

/// Inline Markdown for a doc-text fragment (resolves `{@link}`), single-line.
fn inline(text: &str, context: Option<&MarkdownLinkContext<'_>>) -> String {
    collapse_inline_whitespace(&process_doc_text(text, context)).into_owned()
}

/// Escapes a value for use inside a Markdown table cell.
fn table_cell(text: &str) -> String {
    if text.contains('|') {
        text.replace('|', "\\|")
    } else {
        text.to_string()
    }
}

/// Append a table-cell value directly to `out` (pipes escaped), avoiding the
/// intermediate `String` that [`table_cell`] allocates for every cell.
fn push_table_cell(out: &mut String, text: &str) {
    if text.contains('|') {
        let mut rest = text;
        while let Some(index) = rest.find('|') {
            out.push_str(&rest[..index]);
            out.push_str("\\|");
            rest = &rest[index + 1..];
        }
        out.push_str(rest);
    } else {
        out.push_str(text);
    }
}

/// Inline code for normal Markdown text; empty string if blank.
fn code_span(value: &str) -> String {
    let value = collapse_inline_whitespace(value);
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
    let value = collapse_inline_whitespace(value);
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

/// Escapes Markdown-significant characters in a type annotation's non-identifier
/// text (generics, unions, arrays, …) so they render literally. Pipes are only
/// escaped inside table cells.
fn escape_type_text(text: &str, in_cell: bool) -> String {
    let mut out = String::with_capacity(text.len());
    for ch in text.chars() {
        match ch {
            '\\' => out.push_str("\\\\"),
            '`' => out.push_str("\\`"),
            '<' => out.push_str("\\<"),
            '>' => out.push_str("\\>"),
            '[' => out.push_str("\\["),
            ']' => out.push_str("\\]"),
            '|' if in_cell => out.push_str("\\|"),
            _ => out.push(ch),
        }
    }
    out
}

/// Renders a TypeScript type annotation, linking known symbols. When no identifier
/// resolves to a symbol page the type is returned unchanged as a single inline-code
/// span (`code(value)`); otherwise it is fragmented TypeDoc-style: each identifier
/// is its own inline-code span (linked when resolvable) and punctuation is escaped.
fn linked_type(
    value: &str,
    context: Option<&MarkdownLinkContext<'_>>,
    skip: &HashSet<&str>,
    code: fn(&str) -> String,
    in_cell: bool,
) -> String {
    let value = collapse_type_annotation_whitespace(value);
    match resolve_type_fragments(&value, context, skip) {
        None => code(&value),
        Some(fragments) => {
            let mut out = String::new();
            for fragment in fragments {
                match fragment {
                    TypeFragment::Text(text) => out.push_str(&escape_type_text(&text, in_cell)),
                    TypeFragment::Code(text) => out.push_str(&code(&text)),
                    TypeFragment::Link { name, href } => {
                        out.push('[');
                        out.push_str(&code(&name));
                        out.push_str("](");
                        out.push_str(&href);
                        out.push(')');
                    }
                }
            }
            out
        }
    }
}

fn linked_type_cell(value: &str, context: Option<&MarkdownLinkContext<'_>>) -> String {
    linked_type(value, context, &HashSet::new(), code_cell, true)
}

fn linked_type_span(value: &str, context: Option<&MarkdownLinkContext<'_>>) -> String {
    linked_type(value, context, &HashSet::new(), code_span, false)
}

/// Builds the Name cell for a type parameter: `` `T` `` plus optional `*extends*`
/// constraint and `=` default. The constraint/default link known symbols; the
/// parameter's own name and its siblings (`skip`) are never linked.
fn type_param_name_cell(
    type_param: &ApiTypeParamDoc,
    context: Option<&MarkdownLinkContext<'_>>,
    skip: &HashSet<&str>,
) -> String {
    type_param_name(type_param, context, skip, code_cell, true)
}

fn type_param_name_span(
    type_param: &ApiTypeParamDoc,
    context: Option<&MarkdownLinkContext<'_>>,
    skip: &HashSet<&str>,
) -> String {
    type_param_name(type_param, context, skip, code_span, false)
}

fn type_param_name(
    type_param: &ApiTypeParamDoc,
    context: Option<&MarkdownLinkContext<'_>>,
    skip: &HashSet<&str>,
    code: fn(&str) -> String,
    in_cell: bool,
) -> String {
    let mut cell = code(&type_param.name);
    if let Some(constraint) = &type_param.constraint {
        cell.push_str(" *extends* ");
        cell.push_str(&linked_type(constraint, context, skip, code, in_cell));
    }
    if let Some(default) = &type_param.default {
        cell.push_str(" = ");
        cell.push_str(&linked_type(default, context, skip, code, in_cell));
    }
    cell
}
