//! Opt-in JSON-LD structured data for TechArticle, WebSite, and BreadcrumbList.

use serde::{Deserialize, Serialize};
use serde_json::{Map, Value, json};

use super::breadcrumbs::BreadcrumbsView;
use super::page::{PageData, SsgConfig};

/// Optional publisher written only from site configuration.
#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct JsonLdPublisher {
    /// Organization name. Omitted when empty.
    #[serde(default)]
    pub name: Option<String>,
    /// Organization URL. Unsafe schemes are dropped.
    #[serde(default)]
    pub url: Option<String>,
}

/// Opt-in JSON-LD flags. Off unless [`Self::enabled`] is true.
#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct JsonLd {
    /// When true, emit TechArticle / WebSite (and optional BreadcrumbList).
    #[serde(default)]
    pub enabled: bool,
    /// When true and a visible trail exists, emit BreadcrumbList.
    #[serde(default)]
    pub breadcrumbs: bool,
    /// Optional publisher. Missing fields are not invented.
    #[serde(default)]
    pub publisher: Option<JsonLdPublisher>,
    /// Site origin used for `@id` / `url`. Absolute URLs are omitted when missing.
    #[serde(default, rename = "siteUrl")]
    pub site_url: Option<String>,
}

impl JsonLd {
    /// Omitted / `false` in JS maps here.
    pub fn disabled() -> Self {
        Self::default()
    }

    /// `true` or `{}` in JS: emit TechArticle / WebSite; BreadcrumbList follows the trail.
    pub fn enabled() -> Self {
        Self { enabled: true, breadcrumbs: true, publisher: None, site_url: None }
    }

    pub(super) fn is_enabled(&self) -> bool {
        self.enabled
    }
}

/// Builds the JSON-LD script body, or `None` when the feature is off.
pub(super) fn render_json_ld(
    page: &PageData,
    config: &SsgConfig,
    breadcrumbs: Option<&BreadcrumbsView>,
) -> Option<String> {
    if !config.json_ld.is_enabled() {
        return None;
    }

    let mut graph = vec![website_node(config), tech_article_node(page, config)];
    if config.json_ld.breadcrumbs
        && let Some(trail) = breadcrumbs
        && let Some(list) = breadcrumb_list_node(trail, config)
    {
        graph.push(list);
    }

    let document = json!({
        "@context": "https://schema.org",
        "@graph": graph,
    });
    Some(serialize_for_script(&document))
}

fn website_node(config: &SsgConfig) -> Value {
    let mut node = Map::new();
    node.insert("@type".into(), json!("WebSite"));
    let name = config.site_name.trim();
    if !name.is_empty() {
        node.insert("name".into(), json!(name));
    }
    if let Some(url) = config.json_ld.site_url.as_deref().and_then(safe_http_url) {
        let url = url.trim_end_matches('/');
        node.insert("url".into(), json!(url));
        node.insert("@id".into(), json!(format!("{url}#website")));
    }
    Value::Object(node)
}

fn tech_article_node(page: &PageData, config: &SsgConfig) -> Value {
    let mut node = Map::new();
    node.insert("@type".into(), json!("TechArticle"));
    node.insert("headline".into(), json!(page.title.as_str()));
    if let Some(description) = page.description.as_deref().map(str::trim).filter(|d| !d.is_empty())
    {
        node.insert("description".into(), json!(description));
    }
    if let Some(url) = page_absolute_url(config, &page.path) {
        node.insert("url".into(), json!(url));
        node.insert("@id".into(), json!(format!("{url}#article")));
        if let Some(website_id) = website_id(config) {
            node.insert("isPartOf".into(), json!({ "@id": website_id }));
        }
    }
    if let Some(publisher) = publisher_node(config.json_ld.publisher.as_ref()) {
        node.insert("publisher".into(), publisher);
    }
    Value::Object(node)
}

fn breadcrumb_list_node(trail: &BreadcrumbsView, config: &SsgConfig) -> Option<Value> {
    if trail.crumbs.is_empty() {
        return None;
    }
    let items: Vec<Value> = trail
        .crumbs
        .iter()
        .enumerate()
        .map(|(index, crumb)| {
            let mut item = Map::new();
            item.insert("@type".into(), json!("ListItem"));
            item.insert("position".into(), json!(index + 1));
            item.insert("name".into(), json!(crumb.title.as_str()));
            if let Some(href) = crumb.href.as_deref()
                && let Some(url) = absolute_href(config, href)
            {
                item.insert("item".into(), json!(url));
            }
            Value::Object(item)
        })
        .collect();
    Some(json!({
        "@type": "BreadcrumbList",
        "itemListElement": items,
    }))
}

fn publisher_node(publisher: Option<&JsonLdPublisher>) -> Option<Value> {
    let publisher = publisher?;
    let name = publisher.name.as_deref().map(str::trim).filter(|name| !name.is_empty());
    let url = publisher.url.as_deref().and_then(safe_http_url);
    if name.is_none() && url.is_none() {
        return None;
    }
    let mut node = Map::new();
    node.insert("@type".into(), json!("Organization"));
    if let Some(name) = name {
        node.insert("name".into(), json!(name));
    }
    if let Some(url) = url {
        node.insert("url".into(), json!(url));
    }
    Some(Value::Object(node))
}

fn website_id(config: &SsgConfig) -> Option<String> {
    config
        .json_ld
        .site_url
        .as_deref()
        .and_then(safe_http_url)
        .map(|url| format!("{}#website", url.trim_end_matches('/')))
}

fn page_absolute_url(config: &SsgConfig, path: &str) -> Option<String> {
    let site = config.json_ld.site_url.as_deref().and_then(safe_http_url)?.trim_end_matches('/');
    let path = path.trim().trim_matches('/');
    if path.is_empty() || path.eq_ignore_ascii_case("index") {
        Some(format!("{site}{}", config.base))
    } else {
        Some(format!("{site}{}{path}/", config.base))
    }
}

fn absolute_href(config: &SsgConfig, href: &str) -> Option<String> {
    let href = href.trim();
    if let Some(url) = safe_http_url(href) {
        return Some(url.to_string());
    }
    if !is_safe_relative_href(href) {
        return None;
    }
    let site = config.json_ld.site_url.as_deref().and_then(safe_http_url)?;
    if href.starts_with('/') {
        return Some(format!("{}{href}", site_origin(site)?));
    }
    let site = site.trim_end_matches('/');
    Some(format!("{site}/{href}"))
}

fn site_origin(site_url: &str) -> Option<String> {
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

fn safe_http_url(value: &str) -> Option<&str> {
    let trimmed = value.trim();
    if trimmed.is_empty() || !is_safe_absolute_http_url(trimmed) {
        return None;
    }
    Some(trimmed)
}

fn is_safe_absolute_http_url(value: &str) -> bool {
    let lower = value.to_ascii_lowercase();
    (lower.starts_with("https://") || lower.starts_with("http://"))
        && !lower.starts_with("https:///")
        && is_safe_href(value)
}

fn is_safe_relative_href(href: &str) -> bool {
    !href.is_empty() && is_safe_href(href)
}

fn is_safe_href(href: &str) -> bool {
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

fn serialize_for_script(value: &Value) -> String {
    let json = serde_json::to_string(value).unwrap_or_else(|_| "{}".to_string());
    escape_json_for_script(&json)
}

fn escape_json_for_script(json: &str) -> String {
    let mut out = String::with_capacity(json.len());
    for ch in json.chars() {
        match ch {
            '<' => out.push_str("\\u003c"),
            '>' => out.push_str("\\u003e"),
            '&' => out.push_str("\\u0026"),
            '\u{2028}' => out.push_str("\\u2028"),
            '\u{2029}' => out.push_str("\\u2029"),
            _ => out.push(ch),
        }
    }
    out
}

#[cfg(test)]
mod tests;
