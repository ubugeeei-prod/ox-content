//! Pure Markdown rendering (no raw HTML) for generated API reference docs.
//!
//! Selected when `MarkdownDocsOptions::render_style` is
//! `MarkdownRenderStyle::Markdown`. This is a child module of `markdown`, so it
//! reuses the parent's extraction/formatting helpers via `super::` and emits the
//! same per-entry information as the HTML renderer — but as Markdown headings,
//! tables and fenced code blocks (no `<details>`, no theme-specific HTML).

use super::{
    effective_members_format, effective_parameters_format, fmt_args, generate_source_href,
    parse_example_block, process_doc_text, push_fmt, EntryStats, MarkdownDisplayFormat,
    MarkdownDocsOptions, MarkdownLinkContext,
};
use crate::model::{ApiDocEntry, ApiDocMember, ApiParamDoc, ApiTypeParamDoc};

/// Renders the per-page stats summary as a single italic Markdown line.
pub(super) fn render_stats_markdown(stats: &EntryStats, module_count: Option<usize>) -> String {
    let mut parts = Vec::new();
    if let Some(module_count) = module_count {
        parts.push(fmt_args(format_args!("{module_count} modules")));
    }
    parts.push(fmt_args(format_args!("{} symbols", stats.entries)));
    for kind in super::DOC_KIND_ORDER {
        if let Some(count) = stats.by_kind.get(kind).copied().filter(|count| *count > 0) {
            parts.push(fmt_args(format_args!("{count} {}", super::doc_kind_plural(kind))));
        }
    }
    if stats.params > 0 {
        parts.push(fmt_args(format_args!("{} parameters", stats.params)));
    }
    if stats.members > 0 {
        parts.push(fmt_args(format_args!("{} members", stats.members)));
    }
    if stats.returns > 0 {
        parts.push(fmt_args(format_args!("{} returns", stats.returns)));
    }
    if stats.examples > 0 {
        parts.push(fmt_args(format_args!("{} examples", stats.examples)));
    }
    if stats.deprecated > 0 {
        parts.push(fmt_args(format_args!("{} deprecated", stats.deprecated)));
    }
    fmt_args(format_args!("_{}_", parts.join(" · ")))
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

    if entry.tags.iter().any(|tag| tag.tag == "deprecated") {
        out.push_str("**Deprecated.**\n\n");
    }

    let description = process_doc_text(&entry.description, context);
    let description = description.trim();
    if !description.is_empty() {
        out.push_str(description);
        out.push_str("\n\n");
    }

    if let Some(signature) = &entry.signature {
        push_fmt(
            &mut out,
            format_args!("{heading} Signature\n\n```ts\n{}\n```\n\n", signature.trim()),
        );
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
            push_fmt(&mut out, format_args!("[View source]({href})\n\n"));
        }
    }

    if !entry.type_parameters.is_empty() {
        push_fmt(&mut out, format_args!("{heading} Type Parameters\n\n"));
        match effective_parameters_format(options) {
            MarkdownDisplayFormat::Table => {
                out.push_str("| Name | Description |\n| --- | --- |\n");
                for type_param in &entry.type_parameters {
                    push_fmt(
                        &mut out,
                        format_args!(
                            "| {} | {} |\n",
                            type_param_name_cell(type_param),
                            table_cell(&inline(&type_param.description, context)),
                        ),
                    );
                }
            }
            _ => {
                for type_param in &entry.type_parameters {
                    let description = inline(&type_param.description, context);
                    if description.is_empty() {
                        push_fmt(
                            &mut out,
                            format_args!("- {}\n", type_param_name_span(type_param)),
                        );
                    } else {
                        push_fmt(
                            &mut out,
                            format_args!(
                                "- {} - {description}\n",
                                type_param_name_span(type_param)
                            ),
                        );
                    }
                }
            }
        }
        out.push('\n');
    }

    if !entry.members.is_empty() {
        out.push_str(&render_members_pure(entry, options, context, section_level));
    }

    if !entry.params.is_empty() {
        push_fmt(&mut out, format_args!("{heading} Parameters\n\n"));
        match effective_parameters_format(options) {
            MarkdownDisplayFormat::Table => {
                out.push_str("| Name | Type | Description |\n| --- | --- | --- |\n");
                for param in &entry.params {
                    push_fmt(
                        &mut out,
                        format_args!(
                            "| {} | {} | {} |\n",
                            code_cell(&param.name),
                            code_cell(&param.type_annotation),
                            table_cell(&param_description(param, context)),
                        ),
                    );
                }
            }
            _ => {
                for param in &entry.params {
                    let mut line = fmt_args(format_args!("- {}", code_span(&param.name)));
                    if !param.type_annotation.is_empty() {
                        push_fmt(
                            &mut line,
                            format_args!(" ({})", code_span(&param.type_annotation)),
                        );
                    }
                    let description = param_description(param, context);
                    if !description.is_empty() {
                        push_fmt(&mut line, format_args!(" - {description}"));
                    }
                    out.push_str(&line);
                    out.push('\n');
                }
            }
        }
        out.push('\n');
    }

    if let Some(returns) = &entry.returns {
        push_fmt(&mut out, format_args!("{heading} Returns\n\n"));
        out.push_str(&code_cell(&returns.type_annotation));
        if !returns.description.is_empty() {
            push_fmt(&mut out, format_args!(" — {}", inline(&returns.description, context)));
        }
        out.push_str("\n\n");
    }

    if !entry.examples.is_empty() {
        push_fmt(&mut out, format_args!("{heading} Examples\n\n"));
        for example in &entry.examples {
            let (code, language) = parse_example_block(example);
            push_fmt(&mut out, format_args!("```{language}\n{code}\n```\n\n"));
        }
    }

    if !entry.tags.is_empty() {
        push_fmt(&mut out, format_args!("{heading} Tags\n\n"));
        for tag in &entry.tags {
            let value = inline(&tag.value, context);
            if value.is_empty() {
                push_fmt(&mut out, format_args!("- `@{}`\n", tag.tag));
            } else {
                push_fmt(&mut out, format_args!("- `@{}` — {value}\n", tag.tag));
            }
        }
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
    // Bucket the members lazily: each `match` arm below only uses a subset of
    // these groups (the default arm uses none of them), so computing every
    // bucket up front wasted a full `members` pass + `Vec` per unused group.
    // Mirrors the same optimization in the HTML renderer's
    // `render_members_table_html`.
    let methods = |is_static: bool| {
        members_of(entry, move |member| {
            member.r#static == is_static
                && matches!(member.kind.as_str(), "method" | "getter" | "setter")
        })
    };
    let properties = |is_static: bool| {
        members_of(entry, move |member| member.r#static == is_static && member.kind == "property")
    };

    let groups: Vec<(&str, Vec<&ApiDocMember>)> = match entry.kind.as_str() {
        "class" => vec![
            ("Constructors", members_of(entry, |member| member.kind == "constructor")),
            ("Static Methods", methods(true)),
            ("Methods", methods(false)),
            ("Static Properties", properties(true)),
            ("Properties", properties(false)),
        ],
        "interface" => vec![("Properties", properties(false)), ("Methods", methods(false))],
        "type" => vec![
            ("Properties", properties(false)),
            ("Methods", methods(false)),
            ("Enum Members", members_of(entry, |member| member.kind == "enumMember")),
        ],
        "enum" => vec![("Enum Members", members_of(entry, |member| member.kind == "enumMember"))],
        _ => vec![("Members", entry.members.iter().collect())],
    };

    let mut out = String::new();
    let heading = "#".repeat(section_level);
    for (title, members) in groups {
        if members.is_empty() {
            continue;
        }
        push_fmt(&mut out, format_args!("{heading} {title}\n\n"));
        if effective_members_format(options, &entry.kind, title) == MarkdownDisplayFormat::List {
            for member in members {
                let mut line =
                    fmt_args(format_args!("- {} `{}`", member_name_span(member), member.kind));
                let member_type = member_type(member);
                if !member_type.is_empty() {
                    push_fmt(&mut line, format_args!(" {}", code_span(&member_type)));
                }
                let description = member_description(member, context);
                if !description.is_empty() {
                    push_fmt(&mut line, format_args!(" - {description}"));
                }
                out.push_str(&line);
                out.push('\n');
            }
        } else {
            out.push_str("| Name | Kind | Type | Description |\n| --- | --- | --- | --- |\n");
            for member in members {
                push_fmt(
                    &mut out,
                    format_args!(
                        "| {} | {} | {} | {} |\n",
                        member_name_cell(member),
                        table_cell(&member.kind),
                        code_cell(&member_type(member)),
                        table_cell(&member_description(member, context)),
                    ),
                );
            }
        }
        out.push('\n');
    }
    out
}

fn members_of<'a>(
    entry: &'a ApiDocEntry,
    predicate: impl Fn(&&'a ApiDocMember) -> bool,
) -> Vec<&'a ApiDocMember> {
    entry.members.iter().filter(predicate).collect()
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
        fmt_args(format_args!("{name} _({})_", flags.join(", ")))
    }
}

fn member_type(member: &ApiDocMember) -> String {
    member
        .signature
        .as_deref()
        .or(member.type_annotation.as_deref())
        .or_else(|| member.returns.as_ref().map(|returns| returns.type_annotation.as_str()))
        .unwrap_or_default()
        .to_string()
}

fn member_description(member: &ApiDocMember, context: Option<&MarkdownLinkContext<'_>>) -> String {
    let mut parts = Vec::new();
    if !member.description.is_empty() {
        parts.push(inline(&member.description, context));
    }
    if let Some(returns) = &member.returns {
        if !returns.description.is_empty() {
            parts
                .push(fmt_args(format_args!("Returns: {}", inline(&returns.description, context))));
        }
    }
    parts.join(" ")
}

fn param_description(param: &ApiParamDoc, context: Option<&MarkdownLinkContext<'_>>) -> String {
    let mut description = inline(&param.description, context);
    let mut flags = Vec::new();
    if param.optional {
        flags.push("optional".to_string());
    }
    if let Some(default_value) = &param.default_value {
        flags.push(fmt_args(format_args!("default: {default_value}")));
    }
    if !flags.is_empty() {
        let flags = flags.join(", ");
        description = if description.is_empty() {
            fmt_args(format_args!("_{flags}_"))
        } else {
            fmt_args(format_args!("{description} _({flags})_"))
        };
    }
    description
}

/// Collapses runs of whitespace (including newlines) into single spaces.
fn collapse_whitespace(text: &str) -> String {
    text.split_whitespace().collect::<Vec<_>>().join(" ")
}

/// Inline Markdown for a doc-text fragment (resolves `{@link}`), single-line.
fn inline(text: &str, context: Option<&MarkdownLinkContext<'_>>) -> String {
    collapse_whitespace(&process_doc_text(text, context))
}

/// Escapes a value for use inside a Markdown table cell.
fn table_cell(text: &str) -> String {
    text.replace('|', "\\|")
}

/// Inline code for normal Markdown text; empty string if blank.
fn code_span(value: &str) -> String {
    let value = collapse_whitespace(value);
    if value.is_empty() {
        String::new()
    } else {
        fmt_args(format_args!("`{value}`"))
    }
}

/// Inline code for a Markdown table cell (`|` escaped); empty string if blank.
fn code_cell(value: &str) -> String {
    let value = collapse_whitespace(value);
    if value.is_empty() {
        String::new()
    } else {
        fmt_args(format_args!("`{}`", value.replace('|', "\\|")))
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
        push_fmt(&mut cell, format_args!(" *extends* {}", code(constraint)));
    }
    if let Some(default) = &type_param.default {
        push_fmt(&mut cell, format_args!(" = {}", code(default)));
    }
    cell
}
