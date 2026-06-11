use super::super::{
    generate_source_href, process_doc_text, rendered_throws, MarkdownDocsOptions,
    MarkdownLinkContext,
};
use super::lifecycle::{push_lifecycle_alerts, render_since_section};
use super::member_groups::render_members_pure;
use super::parameters::push_parameters;
use super::return_members::push_returns;
use super::sections::{push_examples, push_generic_tags, push_heritage_sections, push_throws};
use super::type_parameters::push_type_parameters;
use crate::model::ApiDocEntry;

/// Renders the body of one entry (everything below its heading) as pure Markdown.
///
/// `section_level` is the heading level (number of `#`) for the entry's
/// top-level sections — `2` under a page `# Title` (typedoc per-symbol pages),
/// `4` under a flat `### Entry` heading. Sections are emitted as real Markdown
/// headings (not bold paragraphs) so they appear in the VitePress outline, get
/// anchors, and keep a sequential level hierarchy (markdownlint MD001).
pub(in crate::markdown) fn render_entry_body_pure(
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

    let throws = rendered_throws(&entry.throws, &entry.tags);
    push_throws(&mut out, throws.as_ref(), context, &heading);

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
pub(in crate::markdown) fn render_overload_body_pure(
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
        let throws = rendered_throws(&entry.throws, &entry.tags);
        push_throws(&mut out, throws.as_ref(), context, &sub);
        push_examples(&mut out, &entry.examples, &sub);
        push_generic_tags(&mut out, &entry.tags, context, &sub);
    }

    out.trim_end().to_string()
}
