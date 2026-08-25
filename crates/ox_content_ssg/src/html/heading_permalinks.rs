//! CSS-only visibility for renderer heading permalinks.
//!
//! The HTML contract (`<a class="header-anchor" href="#id">`) is owned by the
//! renderer. This module only injects styles and an optional body class.

use super::theme::ThemeConfig;
use super::utils::{page_content_contains_any, wrap_css_section};

pub(super) const HEADING_PERMALINK_CSS: &str = include_str!("heading_permalinks.css");

const MARKER: &str = "header-anchor";

pub(super) fn page_has_heading_permalinks(content: &str) -> bool {
    page_content_contains_any(content, &[MARKER])
}

pub(super) fn heading_permalink_always(theme: Option<&ThemeConfig>) -> bool {
    theme.and_then(|theme| theme.heading_permalink.as_deref()) == Some("always")
}

pub(super) fn push_heading_permalink_css(css_sections: &mut Vec<String>, content: &str) {
    if page_has_heading_permalinks(content) {
        css_sections.push(wrap_css_section("heading-permalinks", HEADING_PERMALINK_CSS));
    }
}

pub(super) fn push_heading_permalink_body_class(
    body_classes: &mut Vec<String>,
    theme: Option<&ThemeConfig>,
    content: &str,
) {
    if page_has_heading_permalinks(content) && heading_permalink_always(theme) {
        body_classes.push("ox-heading-permalinks--always".into());
    }
}

#[cfg(test)]
mod tests;
