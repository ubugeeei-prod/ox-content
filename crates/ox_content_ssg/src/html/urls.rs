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

/// Puts `base` in front of a path that resolves inside the site.
///
/// A site deployed under a sub-path needs every in-site URL prefixed. The
/// renderer already does this for Markdown links and images, and the
/// navigation this crate generates is built on `base` to begin with. A path
/// written in theme config or entry-page frontmatter used to go out
/// verbatim, so one page could carry both shapes and the hand-written half
/// 404'd.
///
/// A root-absolute path is prefixed as written, the way an author would
/// write it against the site root — the value is not inspected for a `base`
/// that is already there, which matches how the renderer rebases Markdown.
///
/// Left alone: another origin, a protocol-relative `//` URL, a bare
/// `#fragment`, and anything carrying a scheme (`data:`, `mailto:`,
/// `javascript:`).
pub(super) fn with_base(base: &str, url: &str) -> String {
    let trimmed = url.trim();
    if trimmed.is_empty()
        || trimmed.starts_with('#')
        || trimmed.starts_with("//")
        || has_uri_scheme(trimmed)
    {
        return trimmed.to_string();
    }

    let base = base.trim_end_matches('/');
    format!("{base}/{}", trimmed.trim_start_matches('/'))
}

/// True when `url` starts with `scheme:`, as opposed to a path with a colon.
fn has_uri_scheme(url: &str) -> bool {
    let mut chars = url.chars();
    if !chars.next().is_some_and(|first| first.is_ascii_alphabetic()) {
        return false;
    }
    for ch in chars {
        if ch == ':' {
            return true;
        }
        if !(ch.is_ascii_alphanumeric() || matches!(ch, '+' | '.' | '-')) {
            return false;
        }
    }
    false
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
