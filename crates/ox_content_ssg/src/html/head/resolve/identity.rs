use super::super::input::{HeadLink, HeadMeta};
use super::ResolvedTag;

pub(super) fn meta_identity(meta: &HeadMeta) -> String {
    if let Some(key) = meta.key.as_deref().map(str::trim).filter(|key| !key.is_empty()) {
        let key_lower = key.to_ascii_lowercase();
        for identity in [
            keyed_identity("name", meta.name.as_deref()),
            keyed_identity("property", meta.property.as_deref()),
            keyed_identity("http-equiv", meta.http_equiv.as_deref()),
        ]
        .into_iter()
        .flatten()
        {
            if key_lower == identity {
                return identity;
            }
        }
        return format!("key:{key}");
    }
    if let Some(name) = meta.name.as_deref() {
        return format!("name:{}", normalize_identity_attr(name));
    }
    if let Some(property) = meta.property.as_deref() {
        return format!("property:{}", normalize_identity_attr(property));
    }
    format!(
        "http-equiv:{}",
        meta.http_equiv.as_deref().map(normalize_identity_attr).unwrap_or_default()
    )
}

pub(super) fn link_identity(link: &HeadLink) -> String {
    let rel = normalize_identity_attr(&link.rel);
    let hreflang = link.hreflang.as_deref().map(normalize_identity_attr).unwrap_or_default();
    if let Some(key) = link.key.as_deref().map(str::trim).filter(|key| !key.is_empty()) {
        if rel == "canonical" && key.eq_ignore_ascii_case("canonical") {
            return "canonical".into();
        }
        if rel == "alternate" && !hreflang.is_empty() {
            let expected = format!("alternate:{hreflang}");
            if key.eq_ignore_ascii_case(&expected) {
                return expected;
            }
        }
        return format!("key:{key}");
    }
    if rel == "alternate" {
        return format!("alternate:{hreflang}");
    }
    rel
}

pub(super) fn json_ld_identity(tag: &ResolvedTag) -> Option<String> {
    match tag {
        ResolvedTag::JsonLd { key, .. } => key.clone(),
        _ => None,
    }
}

fn normalize_identity_attr(value: &str) -> String {
    value.trim().to_ascii_lowercase()
}

fn keyed_identity(prefix: &str, value: Option<&str>) -> Option<String> {
    Some(format!("{prefix}:{}", normalize_identity_attr(value?)))
}
