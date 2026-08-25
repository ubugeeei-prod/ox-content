//! Resolve, deduplicate, and validate head descriptors.

use super::input::{HeadAlternate, HeadInput, HeadJsonLd, HeadLink, HeadMeta, HeadValidation};
use crate::html::urls::{is_safe_href, safe_http_url};
use crate::html::utils::escape_json_for_script;

/// One resolved tag ready to serialize.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ResolvedTag {
    Title(String),
    Meta(HeadMeta),
    Link(HeadLink),
    JsonLd { key: Option<String>, json: String },
}

/// A validation finding. `strict` findings are errors.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HeadDiagnostic {
    pub strict: bool,
    pub message: String,
}

/// Resolved tags plus diagnostics.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ResolvedHead {
    pub tags: Vec<ResolvedTag>,
    pub diagnostics: Vec<HeadDiagnostic>,
}

pub fn resolve_head(input: &HeadInput) -> ResolvedHead {
    let mut diagnostics = Vec::new();
    let title = input.document_title();
    let mut tags = Vec::new();
    if !title.is_empty() {
        tags.push(ResolvedTag::Title(title.clone()));
    }

    push_meta(&mut tags, name_meta("description", input.description.as_deref()));
    if input.social {
        push_meta(&mut tags, property_meta("og:description", input.description.as_deref()));
        push_meta(&mut tags, name_meta("twitter:description", input.description.as_deref()));
    }
    push_meta(&mut tags, name_meta("robots", input.robots.as_deref()));

    if let Some(canonical) =
        resolve_url(input.canonical.as_deref(), "canonical", input.trusted, &mut diagnostics)
    {
        upsert_link(
            &mut tags,
            HeadLink {
                key: Some("canonical".into()),
                rel: "canonical".into(),
                href: canonical.clone(),
                ..HeadLink::default()
            },
        );
        if input.social {
            push_meta(&mut tags, property_meta("og:url", Some(canonical.as_str())));
        }
    }
    if input.social {
        if input.emit_site_name {
            push_meta(&mut tags, property_meta("og:site_name", input.site.name.as_deref()));
        }
        push_meta(
            &mut tags,
            property_meta("og:type", input.og_type.as_deref().or(Some("website"))),
        );
        push_meta(&mut tags, property_meta("og:title", Some(title.as_str())));
        let image =
            resolve_url(input.og_image.as_deref(), "og:image", input.trusted, &mut diagnostics);
        push_meta(&mut tags, property_meta("og:image", image.as_deref()));
        push_meta(&mut tags, name_meta("twitter:image", image.as_deref()));
        push_meta(
            &mut tags,
            name_meta(
                "twitter:card",
                input.twitter_card.as_deref().or(Some("summary_large_image")),
            ),
        );
        push_meta(&mut tags, name_meta("twitter:title", Some(title.as_str())));
    }

    for meta in &input.metas {
        if let Some(meta) = sanitize_meta(meta, &mut diagnostics) {
            upsert_meta(&mut tags, meta);
        }
    }
    for alternate in &input.alternates {
        if let Some(link) = sanitize_alternate(alternate, &mut diagnostics) {
            upsert_link(&mut tags, link);
        }
    }
    for link in &input.links {
        if let Some(link) = sanitize_link(link, &mut diagnostics) {
            upsert_link(&mut tags, link);
        }
    }
    for node in &input.json_ld {
        if let Some(tag) = sanitize_json_ld(node, &mut diagnostics) {
            upsert_json_ld(&mut tags, tag);
        }
    }

    if input.validation == HeadValidation::Off {
        diagnostics.clear();
    }
    ResolvedHead { tags, diagnostics }
}

fn name_meta(name: &str, content: Option<&str>) -> Option<HeadMeta> {
    let content = content.map(str::trim).filter(|value| !value.is_empty())?;
    Some(HeadMeta {
        key: Some(format!("name:{name}")),
        name: Some(name.into()),
        content: content.to_string(),
        ..HeadMeta::default()
    })
}

fn property_meta(property: &str, content: Option<&str>) -> Option<HeadMeta> {
    let content = content.map(str::trim).filter(|value| !value.is_empty())?;
    Some(HeadMeta {
        key: Some(format!("property:{property}")),
        property: Some(property.into()),
        content: content.to_string(),
        ..HeadMeta::default()
    })
}

fn push_meta(tags: &mut Vec<ResolvedTag>, meta: Option<HeadMeta>) {
    if let Some(meta) = meta {
        upsert_meta(tags, meta);
    }
}

fn upsert_meta(tags: &mut Vec<ResolvedTag>, meta: HeadMeta) {
    let identity = meta_identity(&meta);
    if let Some(existing) = tags.iter_mut().find(|tag| match tag {
        ResolvedTag::Meta(current) => meta_identity(current) == identity,
        _ => false,
    }) {
        *existing = ResolvedTag::Meta(meta);
        return;
    }
    tags.push(ResolvedTag::Meta(meta));
}

fn upsert_link(tags: &mut Vec<ResolvedTag>, link: HeadLink) {
    let identity = link_identity(&link);
    if let Some(existing) = tags.iter_mut().find(|tag| match tag {
        ResolvedTag::Link(current) => link_identity(current) == identity,
        _ => false,
    }) {
        *existing = ResolvedTag::Link(link);
        return;
    }
    tags.push(ResolvedTag::Link(link));
}

fn upsert_json_ld(tags: &mut Vec<ResolvedTag>, tag: ResolvedTag) {
    let identity = json_ld_identity(&tag);
    if let Some(existing) =
        tags.iter_mut().find(|current| json_ld_identity(current) == identity && identity.is_some())
    {
        *existing = tag;
        return;
    }
    tags.push(tag);
}

fn meta_identity(meta: &HeadMeta) -> String {
    if let Some(key) = meta.key.as_deref().map(str::trim).filter(|key| !key.is_empty()) {
        return format!("key:{key}");
    }
    if let Some(name) = meta.name.as_deref() {
        return format!("name:{name}");
    }
    if let Some(property) = meta.property.as_deref() {
        return format!("property:{property}");
    }
    format!("http-equiv:{}", meta.http_equiv.as_deref().unwrap_or(""))
}

fn link_identity(link: &HeadLink) -> String {
    if let Some(key) = link.key.as_deref().map(str::trim).filter(|key| !key.is_empty()) {
        return format!("key:{key}");
    }
    if link.rel == "alternate" {
        return format!("alternate:{}", link.hreflang.as_deref().unwrap_or(""));
    }
    link.rel.clone()
}

fn json_ld_identity(tag: &ResolvedTag) -> Option<String> {
    match tag {
        ResolvedTag::JsonLd { key, .. } => key.clone(),
        _ => None,
    }
}

fn resolve_url(
    value: Option<&str>,
    label: &str,
    trusted: bool,
    diagnostics: &mut Vec<HeadDiagnostic>,
) -> Option<String> {
    let raw = value.map(str::trim).filter(|value| !value.is_empty())?;
    if trusted {
        return Some(raw.to_string());
    }
    if let Some(url) = safe_http_url(raw) {
        return Some(url.to_string());
    }
    diagnostics.push(HeadDiagnostic {
        strict: true,
        message: format!("{label} is not a safe http(s) URL"),
    });
    None
}

fn sanitize_meta(meta: &HeadMeta, diagnostics: &mut Vec<HeadDiagnostic>) -> Option<HeadMeta> {
    let has_attr = meta.name.is_some() || meta.property.is_some() || meta.http_equiv.is_some();
    if !has_attr || meta.content.trim().is_empty() {
        diagnostics.push(HeadDiagnostic {
            strict: false,
            message: "meta descriptor needs a name, property, or http-equiv and non-empty content"
                .into(),
        });
        return None;
    }
    Some(meta.clone())
}

fn sanitize_link(link: &HeadLink, diagnostics: &mut Vec<HeadDiagnostic>) -> Option<HeadLink> {
    if link.rel.trim().is_empty() || !is_safe_href(&link.href) {
        diagnostics.push(HeadDiagnostic {
            strict: true,
            message: format!("link rel=\"{}\" has an empty rel or unsafe href", link.rel),
        });
        return None;
    }
    Some(link.clone())
}

fn sanitize_alternate(
    alternate: &HeadAlternate,
    diagnostics: &mut Vec<HeadDiagnostic>,
) -> Option<HeadLink> {
    let lang = alternate.lang.trim();
    if !is_valid_hreflang(lang) {
        diagnostics.push(HeadDiagnostic {
            strict: true,
            message: format!("invalid hreflang \"{}\"", alternate.lang),
        });
        return None;
    }
    if !is_safe_href(&alternate.href) {
        diagnostics.push(HeadDiagnostic {
            strict: true,
            message: format!("alternate hreflang=\"{lang}\" has an unsafe href"),
        });
        return None;
    }
    Some(HeadLink {
        key: Some(format!("alternate:{lang}")),
        rel: "alternate".into(),
        href: alternate.href.trim().to_string(),
        hreflang: Some(lang.to_string()),
        ..HeadLink::default()
    })
}

fn is_valid_hreflang(lang: &str) -> bool {
    if lang.eq_ignore_ascii_case("x-default") {
        return true;
    }
    let mut parts = lang.split('-');
    let primary = parts.next().unwrap_or("");
    if primary.len() < 2 || primary.len() > 8 || !primary.bytes().all(|b| b.is_ascii_alphabetic()) {
        return false;
    }
    parts.all(|part| {
        (1..=8).contains(&part.len()) && part.bytes().all(|b| b.is_ascii_alphanumeric())
    })
}

fn sanitize_json_ld(
    node: &HeadJsonLd,
    diagnostics: &mut Vec<HeadDiagnostic>,
) -> Option<ResolvedTag> {
    let json = node.json.trim();
    if json.is_empty() {
        diagnostics
            .push(HeadDiagnostic { strict: false, message: "empty jsonLd node dropped".into() });
        return None;
    }
    if serde_json::from_str::<serde_json::Value>(json).is_err() {
        diagnostics
            .push(HeadDiagnostic { strict: true, message: "jsonLd node is not valid JSON".into() });
        return None;
    }
    Some(ResolvedTag::JsonLd { key: node.key.clone(), json: escape_json_for_script(json) })
}
