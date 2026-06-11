use super::super::{is_structured_tag, parse_example_block, ExampleBlock, MarkdownLinkContext};
use super::format::{inline, linked_type_span};
use crate::model::{ApiDocEntry, ApiDocTag, ApiThrowsDoc};

pub(super) fn push_heritage_sections(
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

/// Appends a `{heading} Throws` section for exception/error docs.
pub(super) fn push_throws(
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

pub(super) fn push_throws_items(
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

/// Appends a `{heading} Examples` section, or nothing when empty.
pub(super) fn push_examples(out: &mut String, examples: &[String], heading: &str) {
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
pub(super) fn push_generic_tags(
    out: &mut String,
    tags: &[ApiDocTag],
    context: Option<&MarkdownLinkContext<'_>>,
    heading: &str,
) {
    let mut rendered_tags_heading = false;
    for tag in tags {
        if is_structured_tag(&tag.tag) {
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
