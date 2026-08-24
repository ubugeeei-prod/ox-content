//! Opt-in header nav, announcement bar, and per-page chrome flags.

use serde::{Deserialize, Serialize};

use super::utils::escape_html;

pub(super) const HEADER_CHROME_CSS: &str = include_str!("header_chrome.css");
pub(super) const HEADER_CHROME_JS: &str = include_str!("header_chrome.js");

/// Header nav link or dropdown.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct HeaderNavItem {
    #[serde(default)]
    pub text: String,
    #[serde(default)]
    pub link: Option<String>,
    #[serde(default)]
    pub items: Vec<HeaderNavItem>,
}

/// Announcement bar. Text is always escaped.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ThemeAnnouncement {
    #[serde(default)]
    pub text: String,
    pub link: Option<String>,
    #[serde(default, rename = "dismissKey")]
    pub dismiss_key: Option<String>,
}

/// Per-page frontmatter chrome flags. `None` keeps the current default.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct PageChromeFlags {
    pub sidebar: Option<bool>,
    pub outline: Option<bool>,
    pub aside: Option<bool>,
    pub footer: Option<bool>,
    pub navbar: Option<bool>,
    #[serde(rename = "lastUpdated")]
    pub last_updated: Option<bool>,
    #[serde(rename = "editLink")]
    pub edit_link: Option<bool>,
}

/// Resolved visibility after `pageChrome` and frontmatter flags.
#[derive(Debug, Clone, Copy)]
pub(super) struct ResolvedPageChrome {
    pub show_sidebar: bool,
    pub show_outline: bool,
    pub show_footer: bool,
    pub show_navbar: bool,
    pub show_last_updated: bool,
    pub hide_edit_link: bool,
}

pub(super) fn resolve_page_chrome(
    enabled: bool,
    flags: PageChromeFlags,
    theme_aside: bool,
) -> ResolvedPageChrome {
    let hide = |flag: Option<bool>| enabled && flag == Some(false);
    ResolvedPageChrome {
        show_sidebar: !hide(flags.sidebar),
        show_outline: theme_aside && !hide(flags.outline) && !hide(flags.aside),
        show_footer: !hide(flags.footer),
        show_navbar: !hide(flags.navbar),
        show_last_updated: !hide(flags.last_updated),
        hide_edit_link: hide(flags.edit_link),
    }
}

pub(super) fn render_header_nav(items: &[HeaderNavItem]) -> String {
    let mut inner = String::new();
    for item in items {
        inner.push_str(&render_nav_item(item));
    }
    if inner.is_empty() {
        return String::new();
    }
    format!(
        "    <nav class=\"header-nav\" aria-label=\"Header\">\n      <ul class=\"header-nav-list\">\n{inner}      </ul>\n    </nav>\n"
    )
}

fn render_nav_item(item: &HeaderNavItem) -> String {
    let label = item.text.trim();
    if label.is_empty() {
        return String::new();
    }
    let escaped = escape_html(label);
    let children: String = item.items.iter().map(render_nav_item).collect();
    if !children.is_empty() {
        return format!(
            "        <li class=\"header-nav-item header-nav-dropdown\">\n          <button type=\"button\" aria-expanded=\"false\" aria-haspopup=\"true\">{escaped}</button>\n          <ul class=\"header-nav-menu\">\n{children}          </ul>\n        </li>\n"
        );
    }
    let Some(href) = item.link.as_deref().map(str::trim).filter(|href| !href.is_empty()) else {
        return String::new();
    };
    if !is_safe_nav_href(href) {
        return String::new();
    }
    format!(
        "        <li class=\"header-nav-item\"><a href=\"{}\">{escaped}</a></li>\n",
        escape_html(href)
    )
}

pub(super) fn render_announcement(announcement: &ThemeAnnouncement) -> String {
    let text = announcement.text.trim();
    if text.is_empty() {
        return String::new();
    }
    let escaped = escape_html(text);
    let body = match announcement.link.as_deref().map(str::trim).filter(|href| !href.is_empty()) {
        Some(href) if is_safe_announcement_href(href) => {
            format!("<a href=\"{}\">{escaped}</a>", escape_html(href))
        }
        _ => escaped,
    };
    let dismiss = sanitize_dismiss_key(announcement.dismiss_key.as_deref());
    let mut html =
        String::from("<div class=\"ox-announce\" role=\"region\" aria-label=\"Announcement\"");
    if let Some(key) = dismiss.as_deref() {
        html.push_str(" data-ox-announce=\"");
        html.push_str(&escape_html(key));
        html.push('"');
    }
    html.push_str(">\n  <p class=\"ox-announce-text\">");
    html.push_str(&body);
    html.push_str("</p>\n");
    if dismiss.is_some() {
        html.push_str(
            "  <button type=\"button\" class=\"ox-announce-dismiss\" aria-label=\"Dismiss announcement\">Dismiss</button>\n",
        );
        html.push_str(
            "<script>try{var n=document.currentScript.parentElement,k=n&&n.getAttribute(\"data-ox-announce\");if(k&&localStorage.getItem(\"ox-content:announce:\"+k)===\"1\"){n.hidden=true;document.body.classList.remove(\"ox-has-announce\")}}catch(e){}</script>\n",
        );
    }
    html.push_str("</div>\n");
    html
}

pub(super) fn header_chrome_needs_js(nav_html: &str, announcement_html: &str) -> bool {
    nav_html.contains("aria-expanded") || announcement_html.contains("data-ox-announce")
}

pub(super) fn header_chrome_needs_css(
    nav_html: &str,
    announcement_html: &str,
    chrome: ResolvedPageChrome,
) -> bool {
    !nav_html.is_empty()
        || !announcement_html.is_empty()
        || !chrome.show_navbar
        || chrome.hide_edit_link
}

pub(super) fn push_header_chrome_body_classes(
    body_classes: &mut Vec<String>,
    announcement_html: &str,
    chrome: ResolvedPageChrome,
) {
    if !announcement_html.is_empty() {
        body_classes.push("ox-has-announce".into());
    }
    if !chrome.show_navbar {
        body_classes.push("ox-no-navbar".into());
    }
    if chrome.hide_edit_link {
        body_classes.push("ox-hide-edit-link".into());
    }
}

fn is_dangerous_href(href: &str) -> bool {
    let trimmed = href.trim();
    if trimmed.is_empty() || trimmed.starts_with("//") {
        return true;
    }
    let compact: String = trimmed.chars().filter(|ch| !ch.is_ascii_whitespace()).collect();
    let lower = compact.to_ascii_lowercase();
    lower.starts_with("javascript:") || lower.starts_with("data:") || lower.starts_with("vbscript:")
}

fn is_safe_nav_href(href: &str) -> bool {
    !is_dangerous_href(href)
}

fn is_safe_announcement_href(href: &str) -> bool {
    if is_dangerous_href(href) {
        return false;
    }
    let compact: String = href.trim().chars().filter(|ch| !ch.is_ascii_whitespace()).collect();
    let lower = compact.to_ascii_lowercase();
    if lower.starts_with("https://") {
        return true;
    }
    if lower.contains(':') {
        return false;
    }
    lower.starts_with('/')
        || lower.starts_with("./")
        || lower.starts_with("../")
        || lower.starts_with('#')
        || lower.starts_with('?')
        || !lower.is_empty()
}

fn sanitize_dismiss_key(key: Option<&str>) -> Option<String> {
    let trimmed = key.map(str::trim).filter(|value| !value.is_empty())?;
    if trimmed.len() > 64 {
        return None;
    }
    trimmed
        .chars()
        .all(|ch| ch.is_ascii_alphanumeric() || matches!(ch, '-' | '_' | ':' | '.'))
        .then(|| trimmed.to_string())
}

#[cfg(test)]
mod tests;
