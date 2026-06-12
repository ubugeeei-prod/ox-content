use crate::LinkKind;

pub fn classify(target: &str) -> LinkKind {
    if target.starts_with('#') {
        return LinkKind::Anchor;
    }
    if let Some(scheme_end) = target.find(':') {
        let scheme = &target[..scheme_end];
        if is_url_scheme(scheme) {
            return if matches!(scheme, "http" | "https") {
                LinkKind::External
            } else {
                LinkKind::Scheme
            };
        }
    }
    if target.contains('#') {
        LinkKind::FileAnchor
    } else {
        LinkKind::File
    }
}

pub fn split_anchor(target: &str) -> (&str, Option<&str>) {
    target.split_once('#').map_or((target, None), |(file, anchor)| (file, Some(anchor)))
}

pub fn anchor_of(target: &str) -> Option<&str> {
    target.strip_prefix('#')
}

fn is_url_scheme(scheme: &str) -> bool {
    if scheme.is_empty() {
        return false;
    }
    let mut chars = scheme.chars();
    let Some(first) = chars.next() else { return false };
    if !first.is_ascii_alphabetic() {
        return false;
    }
    chars.all(|ch| ch.is_ascii_alphanumeric() || matches!(ch, '+' | '-' | '.'))
}
