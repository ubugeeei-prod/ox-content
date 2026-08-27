//! Numbered cross-references: `@fig-1`, `@tbl-2`, `@sec-intro`.
//!
//! The pass runs over rendered HTML. It numbers every heading, figure, and
//! table that carries a recognisable `id`, records what it found, and rewrites
//! `@id` in prose into a link carrying the assigned label.
//!
//! Diagnostics are returned, not raised. Which of them are fatal is the
//! caller's policy, and this crate does not panic on document content.

mod annotate;
mod html;
#[cfg(test)]
mod tests;
mod text;
mod types;

pub use types::{
    CrossReferenceDiagnostic, CrossReferenceEntry, CrossReferenceKind, CrossReferenceLabels,
    CrossReferenceOutput, CrossReferencesOptions, FailureMode,
};

use std::fmt::Write as _;

use annotate::Registry;
use html::{escape_attr, escape_html, expected_kind};
use text::{find_text_references, transform_text, transform_text_outside_citation_groups};

/// Numbers the document's targets and links every reference to them.
pub fn transform_cross_references(
    html: &str,
    options: &CrossReferencesOptions,
) -> CrossReferenceOutput {
    if !options.enabled {
        return CrossReferenceOutput {
            html: html.to_string(),
            references: Vec::new(),
            diagnostics: Vec::new(),
        };
    }

    let mut registry = Registry::new();
    let mut next = annotate::annotate_sections(html, options, &mut registry);
    next = annotate::annotate_figures_and_images(&next, options, &mut registry);
    next = annotate::apply_trailing_table_labels(&next);
    next = annotate::annotate_tables(&next, options, &mut registry);
    next = replace_references(&next, options, &mut registry);

    // Document order, not discovery order: figures are numbered after every
    // section, but a reader sees them interleaved.
    registry.targets.sort_by_key(|target| target.position);

    CrossReferenceOutput {
        html: next,
        references: registry.targets.into_iter().map(|target| target.entry).collect(),
        diagnostics: registry.diagnostics,
    }
}

fn replace_references(
    html: &str,
    options: &CrossReferencesOptions,
    registry: &mut Registry,
) -> String {
    let mut diagnostics = Vec::new();
    let out = transform_text(html, |text| {
        transform_text_outside_citation_groups(text, |segment| {
            replace_reference_segment(segment, options, registry, &mut diagnostics)
        })
    });
    registry.diagnostics.append(&mut diagnostics);
    out
}

fn replace_reference_segment(
    text: &str,
    options: &CrossReferencesOptions,
    registry: &Registry,
    diagnostics: &mut Vec<CrossReferenceDiagnostic>,
) -> String {
    let references = find_text_references(text);
    if references.is_empty() {
        return text.to_string();
    }

    let mut out = String::with_capacity(text.len());
    let mut cursor = 0usize;
    for reference in references {
        let id = &text[reference.id_start..reference.end];
        let Some(expected) = expected_kind(id) else {
            continue;
        };

        // Everything up to and including the prefix character is kept as-is.
        out.push_str(&text[cursor..reference.start + reference.prefix_len]);
        cursor = reference.start + reference.prefix_len;

        let Some(target) = registry.get(id) else {
            diagnostics.push(CrossReferenceDiagnostic {
                policy: options.missing,
                message: format!("missing cross-reference target \"{id}\""),
            });
            continue;
        };
        if target.kind != expected {
            diagnostics.push(CrossReferenceDiagnostic {
                policy: options.mismatches,
                message: format!(
                    "cross-reference \"{id}\" expects {} but found {}",
                    expected.as_str(),
                    target.kind.as_str()
                ),
            });
            continue;
        }

        let kind = target.kind.as_str();
        let _ = write!(
            out,
            "<a class=\"ox-xref ox-xref-{kind}\" href=\"{}\" data-ox-xref-id=\"{}\" data-ox-xref-kind=\"{kind}\">{}</a>",
            target.href,
            escape_attr(&target.id),
            escape_html(&target.text)
        );
        cursor = reference.end;
    }
    out.push_str(&text[cursor..]);
    out
}
