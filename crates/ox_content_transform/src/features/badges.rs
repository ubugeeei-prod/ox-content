//! Opt-in `{badge:variant}` inline badges.
//!
//! Disabled by default. When enabled, `{badge:tip}Beta{/badge}` becomes a
//! `<span class="ox-badge ox-badge--tip">` with escaped text.

use crate::BadgeOptions;

#[cfg(test)]
mod tests;

const OPEN: &str = "{badge:";
const CLOSE: &str = "{/badge}";

pub(super) fn resolve(options: Option<&BadgeOptions>) -> bool {
    options.is_some_and(|options| options.enabled != Some(false))
}

pub(super) fn replace(segment: &str, out: &mut String) {
    let mut cursor = 0usize;
    while let Some(relative) = segment[cursor..].find(OPEN) {
        let start = cursor + relative;
        out.push_str(&segment[cursor..start]);
        let variant_start = start + OPEN.len();
        let Some(variant_end) = segment[variant_start..].find('}').map(|rel| variant_start + rel)
        else {
            out.push_str(&segment[start..]);
            return;
        };
        let variant = &segment[variant_start..variant_end];
        let text_start = variant_end + 1;
        let Some(close) = segment[text_start..].find(CLOSE).map(|rel| text_start + rel) else {
            out.push_str(&segment[start..]);
            return;
        };
        if is_allowed_variant(variant) {
            emit_badge(variant, &segment[text_start..close], out);
        } else {
            out.push_str(&segment[start..close + CLOSE.len()]);
        }
        cursor = close + CLOSE.len();
    }
    out.push_str(&segment[cursor..]);
}

fn is_allowed_variant(variant: &str) -> bool {
    matches!(
        variant,
        "tip" | "note" | "info" | "warning" | "danger" | "success" | "deprecated" | "required"
    )
}

fn emit_badge(variant: &str, text: &str, out: &mut String) {
    out.push_str("<span class=\"ox-badge ox-badge--");
    out.push_str(variant);
    out.push_str("\">");
    super::escape_html_text(text, out);
    out.push_str("</span>");
}
