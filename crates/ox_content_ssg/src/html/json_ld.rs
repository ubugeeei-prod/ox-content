//! Opt-in JSON-LD structured data for TechArticle, WebSite, and BreadcrumbList.

use serde::{Deserialize, Serialize};
use serde_json::{Map, Value, json};

use super::breadcrumbs::BreadcrumbsView;
use super::page::{PageData, SsgConfig};
use super::urls::{absolute_href, page_absolute_url, safe_http_url};
use super::utils::escape_json_for_script;

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
    /// Page `@type`. Defaults to `TechArticle`.
    #[serde(default, rename = "type")]
    pub page_type: Option<String>,
    /// Extra `@graph` nodes. Invalid JSON objects are dropped.
    #[serde(default)]
    pub graph: Vec<Value>,
}

impl JsonLd {
    /// Omitted / `false` in JS maps here.
    pub fn disabled() -> Self {
        Self::default()
    }

    /// `true` or `{}` in JS: emit TechArticle / WebSite; BreadcrumbList follows the trail.
    pub fn enabled() -> Self {
        Self {
            enabled: true,
            breadcrumbs: true,
            publisher: None,
            site_url: None,
            page_type: None,
            graph: Vec::new(),
        }
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

    let mut graph = vec![website_node(config), page_node(page, config)];
    if config.json_ld.breadcrumbs
        && let Some(trail) = breadcrumbs
        && let Some(list) = breadcrumb_list_node(trail, config)
    {
        graph.push(list);
    }
    graph.extend(config.json_ld.graph.iter().filter(|node| node.is_object()).cloned());

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
    if let Some(url) = json_ld_site_url(config) {
        let url = url.trim_end_matches('/');
        node.insert("url".into(), json!(url));
        node.insert("@id".into(), json!(format!("{url}#website")));
    }
    Value::Object(node)
}

fn page_node(page: &PageData, config: &SsgConfig) -> Value {
    let mut node = Map::new();
    node.insert("@type".into(), json!(page_schema_type(config.json_ld.page_type.as_deref())));
    node.insert("headline".into(), json!(page.title.as_str()));
    if let Some(description) = page.description.as_deref().map(str::trim).filter(|d| !d.is_empty())
    {
        node.insert("description".into(), json!(description));
    }
    if let Some(url) = json_ld_page_url(config, &page.path) {
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
                && let Some(url) = json_ld_absolute_href(config, href)
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
    json_ld_site_url(config).map(|url| format!("{}#website", url.trim_end_matches('/')))
}

fn json_ld_site_url(config: &SsgConfig) -> Option<&str> {
    config.json_ld.site_url.as_deref().or(config.site_url.as_deref()).and_then(safe_http_url)
}

fn json_ld_page_url(config: &SsgConfig, path: &str) -> Option<String> {
    page_absolute_url(json_ld_site_url(config)?, &config.base, path)
}

fn json_ld_absolute_href(config: &SsgConfig, href: &str) -> Option<String> {
    absolute_href(json_ld_site_url(config)?, href)
}

fn page_schema_type(page_type: Option<&str>) -> &str {
    match page_type.map(str::trim) {
        Some("BlogPosting") => "BlogPosting",
        Some("WebPage") => "WebPage",
        _ => "TechArticle",
    }
}

fn serialize_for_script(value: &Value) -> String {
    let json = serde_json::to_string(value).unwrap_or_else(|_| "{}".to_string());
    escape_json_for_script(&json)
}

#[cfg(test)]
mod tests;
