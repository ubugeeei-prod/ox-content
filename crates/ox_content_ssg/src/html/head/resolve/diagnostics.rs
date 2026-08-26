use super::super::input::{HeadLink, HeadMeta};
use super::identity::{link_identity, meta_identity};
use super::{HeadDiagnostic, ResolvedTag};

pub(super) fn diagnose_meta_replacement(
    previous: Option<&HeadMeta>,
    next: &HeadMeta,
    diagnostics: &mut Vec<HeadDiagnostic>,
) {
    let Some(previous) = previous else {
        return;
    };
    let Some(label) = seo_meta_label(next).or_else(|| seo_meta_label(previous)) else {
        return;
    };
    diagnostics.push(HeadDiagnostic {
        strict: false,
        message: format!("{label} meta overrides earlier SEO metadata; last descriptor wins"),
    });
}

pub(super) fn diagnose_link_replacement(
    previous: Option<&HeadLink>,
    next: &HeadLink,
    diagnostics: &mut Vec<HeadDiagnostic>,
) {
    let Some(previous) = previous else {
        return;
    };
    let Some(label) = seo_link_label(next).or_else(|| seo_link_label(previous)) else {
        return;
    };
    diagnostics.push(HeadDiagnostic {
        strict: false,
        message: format!("{label} overrides earlier SEO metadata; last descriptor wins"),
    });
}

fn seo_meta_label(meta: &HeadMeta) -> Option<&'static str> {
    match meta_identity(meta).as_str() {
        "name:description" => Some("description"),
        "name:robots" => Some("robots"),
        "name:twitter:card" => Some("twitter:card"),
        "name:twitter:description" => Some("twitter:description"),
        "name:twitter:image" => Some("twitter:image"),
        "name:twitter:title" => Some("twitter:title"),
        "property:og:description" => Some("og:description"),
        "property:og:image" => Some("og:image"),
        "property:og:site_name" => Some("og:site_name"),
        "property:og:title" => Some("og:title"),
        "property:og:type" => Some("og:type"),
        "property:og:url" => Some("og:url"),
        _ => None,
    }
}

fn seo_link_label(link: &HeadLink) -> Option<String> {
    match link_identity(link).as_str() {
        "canonical" => Some("canonical link".into()),
        identity if identity.starts_with("alternate:") => Some(format!(
            "hreflang \"{}\" alternate",
            link.hreflang.as_deref().map(str::trim).filter(|value| !value.is_empty()).unwrap_or("")
        )),
        _ => None,
    }
}

pub(super) fn diagnose_canonical_og_url_conflict(
    tags: &[ResolvedTag],
    diagnostics: &mut Vec<HeadDiagnostic>,
) {
    let canonical = tags.iter().find_map(|tag| match tag {
        ResolvedTag::Link(link) if link_identity(link) == "canonical" => Some(link.href.trim()),
        _ => None,
    });
    let og_url = tags.iter().find_map(|tag| match tag {
        ResolvedTag::Meta(meta) if meta_identity(meta) == "property:og:url" => {
            Some(meta.content.trim())
        }
        _ => None,
    });
    let Some(canonical) = canonical else {
        return;
    };
    let Some(og_url) = og_url else {
        return;
    };
    if !canonical.is_empty() && !og_url.is_empty() && canonical != og_url {
        diagnostics.push(HeadDiagnostic {
            strict: false,
            message: "canonical link and og:url disagree; emitted descriptors were kept".into(),
        });
    }
}
