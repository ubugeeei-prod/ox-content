use std::borrow::Cow;

use super::format_count_label;
use crate::model::{ApiDocEntry, ApiDocTag, ApiThrowsDoc};
use crate::string_builder::StringBuilder;

#[derive(Debug, Clone)]
pub(super) struct EntryBadge {
    pub(in crate::markdown) label: String,
    pub(in crate::markdown) tone: Option<&'static str>,
}

fn entry_tag_value<'a>(entry: &'a ApiDocEntry, tag_name: &str) -> Option<&'a str> {
    entry.tags.iter().find(|tag| tag.tag == tag_name).map(|tag| tag.value.as_str())
}

/// JSDoc tags folded into a dedicated `Since` element (TypeDoc parity) instead of
/// the generic tag list. `@version` is normalized alongside `@since`. Shared by
/// both renderers (`super::SINCE_TAGS`).
pub(super) const SINCE_TAGS: [&str; 2] = ["since", "version"];

/// JSDoc lifecycle tags surfaced as structured callouts — GitHub alerts in the
/// markdown renderer, badges in the HTML renderer — rather than generic tags.
fn is_lifecycle_tag(tag: &str) -> bool {
    matches!(tag, "deprecated" | "experimental")
}

pub(super) fn is_throws_tag(tag: &str) -> bool {
    matches!(tag, "throws" | "exception")
}

pub(super) fn rendered_throws<'a>(
    throws: &'a [ApiThrowsDoc],
    tags: &[ApiDocTag],
) -> Cow<'a, [ApiThrowsDoc]> {
    if !throws.is_empty() {
        return Cow::Borrowed(throws);
    }

    Cow::Owned(tags.iter().filter_map(api_throws_from_tag).collect())
}

fn api_throws_from_tag(tag: &ApiDocTag) -> Option<ApiThrowsDoc> {
    if !is_throws_tag(&tag.tag) {
        return None;
    }
    let value = tag.value.trim();
    if value.is_empty() {
        return None;
    }
    if let Some((type_annotation, description)) = parse_throws_tag_value(value) {
        return Some(ApiThrowsDoc { type_annotation: Some(type_annotation), description });
    }
    Some(ApiThrowsDoc { type_annotation: None, description: value.to_string() })
}

fn parse_throws_tag_value(value: &str) -> Option<(String, String)> {
    let rest = value.strip_prefix('{')?;
    let end = rest.find('}')?;
    let type_annotation = rest[..end].trim();
    if type_annotation.is_empty() {
        return None;
    }
    let description = rest[end + 1..].trim().trim_start_matches('-').trim().to_string();
    Some((type_annotation.to_string(), description))
}

/// True when a tag is rendered as a structured element (lifecycle callout / Since
/// / Throws) and therefore must not also appear in the generic tag list. Shared
/// by both renderers so the generic-tag exclusion stays consistent.
pub(super) fn is_structured_tag(name: &str) -> bool {
    is_lifecycle_tag(name) || SINCE_TAGS.contains(&name) || is_throws_tag(name)
}

pub(super) fn get_entry_badges(entry: &ApiDocEntry) -> Vec<EntryBadge> {
    let mut badges = Vec::new();

    if entry_tag_value(entry, "deprecated").is_some() {
        badges.push(EntryBadge { label: "deprecated".to_string(), tone: Some("warning") });
    }
    if entry_tag_value(entry, "experimental").is_some() {
        badges.push(EntryBadge { label: "experimental".to_string(), tone: Some("warning") });
    }
    if !entry.params.is_empty() {
        badges.push(EntryBadge {
            label: format_count_label(entry.params.len(), "param", Some("params")),
            tone: None,
        });
    }
    if !entry.members.is_empty() {
        badges.push(EntryBadge {
            label: format_count_label(entry.members.len(), "member", Some("members")),
            tone: None,
        });
    }
    if let Some(returns) = &entry.returns {
        let mut label =
            StringBuilder::with_capacity("returns ".len() + returns.type_annotation.len());
        label.push_str("returns ");
        label.push_str(&returns.type_annotation);
        badges.push(EntryBadge { label: label.into_string(), tone: None });
    }
    if !entry.examples.is_empty() {
        badges.push(EntryBadge {
            label: format_count_label(entry.examples.len(), "example", Some("examples")),
            tone: None,
        });
    }
    if let Some(since) = entry_tag_value(entry, "since") {
        let mut label = StringBuilder::with_capacity("since ".len() + since.len());
        label.push_str("since ");
        label.push_str(since);
        badges.push(EntryBadge { label: label.into_string(), tone: None });
    }
    if let Some(version) = entry_tag_value(entry, "version") {
        let mut label = StringBuilder::with_capacity("version ".len() + version.len());
        label.push_str("version ");
        label.push_str(version);
        badges.push(EntryBadge { label: label.into_string(), tone: None });
    }
    if entry.private {
        badges.push(EntryBadge { label: "private".to_string(), tone: Some("warning") });
    }

    badges
}
