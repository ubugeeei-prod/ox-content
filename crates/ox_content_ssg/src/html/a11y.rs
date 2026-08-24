//! Opt-in skip link and print styles.

use serde::{Deserialize, Serialize};

use super::utils::escape_html;

/// Skip-link and print-style flags. Off unless enabled.
///
/// `None` for [`Self::skip_link_label`] means the feature is off. `Some`
/// enables it; an empty label falls back to `Skip to content`.
#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct A11y {
    /// Override for the skip-link label. `Some` enables the feature.
    #[serde(default, rename = "skipLinkLabel")]
    pub skip_link_label: Option<String>,
}

impl A11y {
    /// Omitted / `false` in JS maps here.
    pub fn disabled() -> Self {
        Self::default()
    }

    /// `true` or `{}` in JS: emit the skip link and print CSS.
    pub fn enabled() -> Self {
        Self { skip_link_label: Some(String::new()) }
    }

    pub(super) fn is_enabled(&self) -> bool {
        self.skip_link_label.is_some()
    }

    pub(super) fn label(&self) -> &str {
        self.skip_link_label
            .as_deref()
            .map(str::trim)
            .filter(|label| !label.is_empty())
            .unwrap_or("Skip to content")
    }

    pub(super) fn skip_link_html(&self) -> Option<String> {
        self.is_enabled().then(|| {
            format!("<a class=\"ox-skip-link\" href=\"#ox-main\">{}</a>", escape_html(self.label()))
        })
    }
}

pub(super) const A11Y_CSS: &str = include_str!("a11y.css");

#[cfg(test)]
mod tests;
