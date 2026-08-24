use std::borrow::Cow;

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
    if target.contains('#') { LinkKind::FileAnchor } else { LinkKind::File }
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

pub fn percent_decode(input: &str) -> Cow<'_, str> {
    if !input.as_bytes().contains(&b'%') {
        return Cow::Borrowed(input);
    }

    let bytes = input.as_bytes();
    let mut decoded = Vec::with_capacity(bytes.len());
    let mut index = 0;
    while index < bytes.len() {
        if bytes[index] == b'%'
            && index + 2 < bytes.len()
            && let (Some(high), Some(low)) = (hex(bytes[index + 1]), hex(bytes[index + 2]))
        {
            decoded.push((high << 4) | low);
            index += 3;
            continue;
        }
        decoded.push(bytes[index]);
        index += 1;
    }

    match String::from_utf8(decoded) {
        Ok(value) => Cow::Owned(value),
        Err(_) => Cow::Borrowed(input),
    }
}

fn hex(byte: u8) -> Option<u8> {
    match byte {
        b'0'..=b'9' => Some(byte - b'0'),
        b'a'..=b'f' => Some(byte - b'a' + 10),
        b'A'..=b'F' => Some(byte - b'A' + 10),
        _ => None,
    }
}
