// @author kazuya kawaguchi (a.k.a. kazupon)
// @license MIT
//

//! Description text post-processing helpers (decoder side).
//!
//! Mirrors `ox_jsdoc::parser::text::parsed_preserving_whitespace` byte-for-byte
//! so the binary AST decoder can produce the same preserve-whitespace output
//! as the typed AST `description_text(true)` method without taking a
//! cross-crate dependency. See `design/008-oxlint-oxfmt-support/README.md`
//! §3 for the algorithm + §4.3 for the JS API contract this method backs.

/// Reflow a raw description slice into preserve-whitespace form. See
/// `crates/ox_jsdoc/src/parser/text.rs` for the canonical implementation
/// + per-line behavior (markdown emphasis exemption, indented code block
/// preservation, blank-line preservation).
#[must_use]
pub fn parsed_preserving_whitespace(raw: &str) -> String {
    if !raw.contains('\n') {
        return raw.trim().to_string();
    }
    let mut result = String::with_capacity(raw.len());
    for (i, line) in raw.lines().enumerate() {
        if i > 0 {
            result.push('\n');
        }
        let trimmed = line.trim();
        if let Some(rest) = trimmed.strip_prefix('*') {
            let is_emphasis = rest.starts_with(|ch: char| ch.is_alphanumeric() || ch == '_');
            if !is_emphasis {
                result.push_str(rest.strip_prefix(' ').unwrap_or(rest));
                continue;
            }
        }
        result.push_str(trimmed);
    }
    result
}
