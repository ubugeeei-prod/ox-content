//! Opt-in `{kbd:...}` inline keyboard keys.
//!
//! Disabled by default. When enabled, `{kbd:Ctrl+K}` becomes nested
//! `<kbd class="ox-kbd">` markup with escaped key labels. Aliases are
//! resolved from build config, not the runtime user agent.

use rustc_hash::FxHashMap;
use std::borrow::Cow;

use crate::KeyboardKeysOptions;

mod aliases;
mod protect;

#[cfg(test)]
mod tests;

use aliases::{KeyboardKeyStyle, normalize_key};
use protect::next_protected;

#[derive(Clone)]
pub(super) struct ResolvedKeyboardKeys {
    aliases: FxHashMap<String, String>,
    style: KeyboardKeyStyle,
}

const OPEN: &str = "{kbd:";

pub(super) fn resolve(options: Option<&KeyboardKeysOptions>) -> Option<ResolvedKeyboardKeys> {
    let options = options?;
    if options.enabled == Some(false) {
        return None;
    }
    Some(ResolvedKeyboardKeys {
        aliases: options
            .aliases
            .as_ref()
            .map(|aliases| {
                aliases
                    .iter()
                    .map(|(key, value)| (key.to_ascii_lowercase(), value.clone()))
                    .collect()
            })
            .unwrap_or_default(),
        style: KeyboardKeyStyle::from_option(options.style.as_deref()),
    })
}

pub(super) fn apply(current: &mut Cow<'_, str>, options: Option<&ResolvedKeyboardKeys>) {
    let Some(options) = options else {
        return;
    };
    if current.contains(OPEN)
        && let Some(replaced) =
            super::segments::transform_markdown_text_segments(current, |segment, out| {
                replace(segment, options, out);
            })
    {
        *current = Cow::Owned(replaced);
    }
}

pub(super) fn replace(segment: &str, options: &ResolvedKeyboardKeys, out: &mut String) {
    let mut cursor = 0usize;
    while cursor < segment.len() {
        let kbd = segment[cursor..].find(OPEN).map(|rel| cursor + rel);
        let protected = next_protected(segment, cursor);
        match (kbd, protected) {
            (None, None) => {
                out.push_str(&segment[cursor..]);
                return;
            }
            (Some(start), Some(span)) if span.start <= start => {
                out.push_str(&segment[cursor..span.end]);
                cursor = span.end;
            }
            (None, Some(span)) => {
                out.push_str(&segment[cursor..span.end]);
                cursor = span.end;
            }
            (Some(start), _) => {
                out.push_str(&segment[cursor..start]);
                cursor = rewrite_or_copy(segment, start, options, out);
            }
        }
    }
}

fn rewrite_or_copy(
    segment: &str,
    start: usize,
    options: &ResolvedKeyboardKeys,
    out: &mut String,
) -> usize {
    if start > 0 && segment.as_bytes()[start - 1] == b'\\' {
        out.pop();
        return copy_literal_notation(segment, start, out);
    }
    let body_start = start + OPEN.len();
    let Some(close) = segment[body_start..].find('}').map(|rel| body_start + rel) else {
        out.push_str(&segment[start..]);
        return segment.len();
    };
    let body = &segment[body_start..close];
    if !emit_keys(body, options, out) {
        out.push_str(&segment[start..close + 1]);
    }
    close + 1
}

fn copy_literal_notation(segment: &str, start: usize, out: &mut String) -> usize {
    let body_start = start + OPEN.len();
    if let Some(close) = segment[body_start..].find('}').map(|rel| body_start + rel) {
        out.push_str(&segment[start..close + 1]);
        close + 1
    } else {
        out.push_str(&segment[start..]);
        segment.len()
    }
}

fn emit_keys(body: &str, options: &ResolvedKeyboardKeys, out: &mut String) -> bool {
    if body.bytes().any(|byte| byte == b'\n' || byte == b'\r') {
        return false;
    }
    let trimmed = body.trim();
    if trimmed.is_empty() {
        return false;
    }
    if !has_key(trimmed) {
        return false;
    }
    out.push_str("<kbd class=\"ox-kbd");
    if is_combo(trimmed) {
        out.push_str(" ox-kbd--combo");
    }
    out.push_str("\">");
    if trimmed == "+" {
        emit_key("+", options, out);
    } else {
        for part in trimmed.split(|ch: char| ch == '+' || ch.is_whitespace()) {
            let key = part.trim();
            if key.is_empty() {
                continue;
            }
            emit_key(key, options, out);
        }
    }
    out.push_str("</kbd>");
    true
}

fn emit_key(key: &str, options: &ResolvedKeyboardKeys, out: &mut String) {
    out.push_str("<kbd class=\"ox-kbd__key\">");
    super::escape_html_text(&normalize_key(key, &options.aliases, options.style), out);
    out.push_str("</kbd>");
}

fn has_key(body: &str) -> bool {
    body == "+"
        || body
            .split(|ch: char| ch == '+' || ch.is_whitespace())
            .any(|part| !part.trim().is_empty())
}

fn is_combo(body: &str) -> bool {
    if body == "+" {
        return false;
    }
    let mut seen = false;
    for part in body.split(|ch: char| ch == '+' || ch.is_whitespace()) {
        if part.trim().is_empty() {
            continue;
        }
        if seen {
            return true;
        }
        seen = true;
    }
    false
}
