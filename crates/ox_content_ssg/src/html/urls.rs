//! Shared URL safety for JSON-LD, page-head, and SEO metadata.

/// Absolute `http:` / `https:` URL, or `None` when empty or unsafe.
pub(super) fn safe_http_url(value: &str) -> Option<&str> {
    let trimmed = value.trim();
    if trimmed.is_empty() || !is_safe_absolute_http_url(trimmed) {
        return None;
    }
    Some(trimmed)
}

pub(super) fn is_safe_absolute_http_url(value: &str) -> bool {
    let lower = value.to_ascii_lowercase();
    (lower.starts_with("https://") || lower.starts_with("http://"))
        && !lower.starts_with("https:///")
        && is_safe_href(value)
}

pub(super) fn is_safe_relative_href(href: &str) -> bool {
    !href.is_empty() && is_safe_href(href)
}

/// Rejects `javascript:`, `data:`, `vbscript:`, and protocol-relative URLs.
pub(super) fn is_safe_href(href: &str) -> bool {
    let trimmed = href.trim();
    if trimmed.is_empty() || trimmed.starts_with("//") {
        return false;
    }
    let lower = trimmed.to_ascii_lowercase();
    if lower.starts_with("javascript:")
        || lower.starts_with("data:")
        || lower.starts_with("vbscript:")
    {
        return false;
    }
    if let Some(scheme_end) = trimmed.find(':') {
        let scheme = &lower[..scheme_end];
        return matches!(scheme, "http" | "https");
    }
    true
}

pub(super) fn site_origin(site_url: &str) -> Option<String> {
    let (scheme, rest) = if let Some(rest) = site_url.strip_prefix("https://") {
        ("https", rest)
    } else {
        let rest = site_url.strip_prefix("http://")?;
        ("http", rest)
    };
    let host = rest.split('/').next()?.trim();
    if host.is_empty() {
        return None;
    }
    Some(format!("{scheme}://{host}"))
}

/// Joins `site_url` + `base` + page path, applying trailing slashes.
pub(super) fn page_absolute_url(site_url: &str, base: &str, path: &str) -> Option<String> {
    let site = safe_http_url(site_url)?.trim_end_matches('/');
    let path = path.trim().trim_matches('/');
    if path.is_empty() || path.eq_ignore_ascii_case("index") {
        Some(format!("{site}{base}"))
    } else {
        Some(format!("{site}{base}{path}/"))
    }
}

pub(super) fn absolute_href(site_url: &str, href: &str) -> Option<String> {
    let href = href.trim();
    if let Some(url) = safe_http_url(href) {
        return Some(url.to_string());
    }
    if !is_safe_relative_href(href) {
        return None;
    }
    let site = safe_http_url(site_url)?;
    if href.starts_with('/') {
        return Some(format!("{}{href}", site_origin(site)?));
    }
    let site = site.trim_end_matches('/');
    Some(format!("{site}/{href}"))
}
