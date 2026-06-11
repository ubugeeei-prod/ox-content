use super::super::{MarkdownLinkContext, SINCE_TAGS};
use super::format::inline;
use crate::model::ApiDocTag;

/// JSDoc lifecycle tags rendered as GitHub alerts rather than generic `## Tags`
/// entries: `@experimental` → `> [!WARNING]`, `@deprecated` → `> [!CAUTION]`.
/// Appends GitHub alert blocks for lifecycle tags (`@experimental`,
/// `@deprecated`) present in `tags`, in source order. Uses the tag's own text as
/// the alert body (with `{@link}` resolved), falling back to a default message.
pub(in crate::markdown) fn push_lifecycle_alerts(
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
pub(super) fn render_since_section(
    tags: &[ApiDocTag],
    context: Option<&MarkdownLinkContext<'_>>,
    heading: &str,
) -> String {
    let values = tags
        .iter()
        .filter(|tag| SINCE_TAGS.contains(&tag.tag.as_str()))
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
