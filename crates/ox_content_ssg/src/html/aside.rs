use super::ThemeConfig;

/// `theme.aside` is opt-in. Omitted and `false` keep the outline off.
pub(super) fn aside_enabled(theme: Option<&ThemeConfig>) -> bool {
    theme.and_then(|theme| theme.aside) == Some(true)
}

/// Show the outline only when it is enabled and the page has TOC HTML.
pub(super) fn has_toc(aside_enabled: bool, toc_html: &str) -> bool {
    aside_enabled && !toc_html.is_empty()
}
