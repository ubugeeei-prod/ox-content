//! Opt-in header locale switcher for the default theme.

use super::page::{LocaleInfo, SsgConfig};
use super::utils::escape_html;

/// Renders the header locale switcher, or an empty string when it is off.
pub(super) fn render_locale_switcher(config: &SsgConfig) -> String {
    render_switcher(config, false)
}

/// Renders the phone language control in the persistent navigation bar.
pub(super) fn render_mobile_locale_switcher(config: &SsgConfig) -> String {
    render_switcher(config, true)
}

fn render_switcher(config: &SsgConfig, mobile: bool) -> String {
    if !config.locale_switcher {
        return String::new();
    }
    let Some(locales) = config.available_locales.as_ref().filter(|locales| !locales.is_empty())
    else {
        return String::new();
    };

    let current = config.locale.as_deref().unwrap_or("");
    let current_name = locales
        .iter()
        .find(|locale| locale.code == current)
        .or_else(|| locales.first())
        .map(|locale| escape_html(&locale.name))
        .unwrap_or_default();
    let mut html = if mobile {
        String::from(
            "<nav class=\"ox-header-select ox-locale-switcher ox-mobile-locale-switcher\" aria-label=\"Language\">",
        )
    } else {
        String::from("<nav class=\"ox-header-select ox-locale-switcher\" aria-label=\"Language\">")
    };
    if mobile {
        html.push_str("<button type=\"button\" class=\"mobile-footer-btn\" aria-label=\"Change language\" disabled aria-expanded=\"false\" aria-haspopup=\"true\"><svg viewBox=\"0 0 24 24\" fill=\"none\" stroke=\"currentColor\" stroke-width=\"2\"><circle cx=\"12\" cy=\"12\" r=\"9\"/><path d=\"M3 12h18M12 3a18 18 0 0 1 0 18 18 18 0 0 1 0-18\"/></svg><span class=\"mobile-footer-label\">");
        html.push_str(&current_name);
        html.push_str("</span></button>");
    } else {
        html.push_str(
            "<button type=\"button\" disabled aria-expanded=\"false\" aria-haspopup=\"true\">",
        );
        html.push_str(&current_name);
        html.push_str("</button>");
    }
    html.push_str("<ul class=\"ox-header-select-menu\">");
    for locale in locales {
        html.push_str("<li>");
        html.push_str(&render_locale_item(locale, current, config));
        html.push_str("</li>");
    }
    html.push_str("</ul></nav>");
    html
}

fn render_locale_item(locale: &LocaleInfo, current: &str, config: &SsgConfig) -> String {
    let code = escape_html(&locale.code);
    let name = escape_html(&locale.name);
    let dir = match locale.dir.as_str() {
        "rtl" => "rtl",
        _ => "ltr",
    };
    if locale.code == current {
        return format!("<span lang=\"{code}\" dir=\"{dir}\" aria-current=\"page\">{name}</span>");
    }
    match locale_href(locale, config) {
        Some(href) => format!(
            "<a href=\"{}\" hreflang=\"{code}\" lang=\"{code}\" dir=\"{dir}\">{name}</a>",
            escape_html(&href)
        ),
        None => format!("<span lang=\"{code}\" dir=\"{dir}\">{name}</span>"),
    }
}

fn locale_href(locale: &LocaleInfo, config: &SsgConfig) -> Option<String> {
    let path = config.locale_paths.iter().find(|path| path.code == locale.code);
    if let Some(href) = path.and_then(|path| path.href.as_deref()).filter(|href| !href.is_empty())
        && is_safe_href(href)
    {
        return Some(href.to_string());
    }
    if let Some(root) = path.and_then(|path| path.root.as_deref()).filter(|root| !root.is_empty()) {
        return is_safe_href(root).then(|| root.to_string());
    }
    let default_root = default_locale_root(&config.base, &locale.code);
    is_safe_href(&default_root).then_some(default_root)
}

fn default_locale_root(base: &str, code: &str) -> String {
    let base = if base.is_empty() { "/" } else { base };
    if base.ends_with('/') { format!("{base}{code}/") } else { format!("{base}/{code}/") }
}

fn is_safe_href(href: &str) -> bool {
    let trimmed = href.trim();
    if trimmed.is_empty() || trimmed.starts_with("//") {
        return false;
    }
    let compact: String = trimmed.chars().filter(|ch| !ch.is_ascii_whitespace()).collect();
    let lower = compact.to_ascii_lowercase();
    if lower.starts_with("javascript:")
        || lower.starts_with("data:")
        || lower.starts_with("vbscript:")
    {
        return false;
    }
    if trimmed.starts_with('/') || trimmed.starts_with("./") || trimmed.starts_with("../") {
        return true;
    }
    match lower.find(':') {
        Some(idx) => matches!(&lower[..idx], "http" | "https" | "mailto"),
        None => true,
    }
}

#[cfg(test)]
mod tests;
