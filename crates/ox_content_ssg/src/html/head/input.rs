//! Unhead-compatible head descriptors. Build-time only; no client runtime.

use serde::{Deserialize, Serialize};

pub use crate::html::page::HeadValidation;

/// One `meta` descriptor.
#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct HeadMeta {
    /// Stable identity. Wins over name/property when set.
    #[serde(default)]
    pub key: Option<String>,
    #[serde(default)]
    pub name: Option<String>,
    #[serde(default)]
    pub property: Option<String>,
    #[serde(default, rename = "httpEquiv")]
    pub http_equiv: Option<String>,
    #[serde(default)]
    pub content: String,
}

/// One `link` descriptor.
#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct HeadLink {
    #[serde(default)]
    pub key: Option<String>,
    #[serde(default)]
    pub rel: String,
    #[serde(default)]
    pub href: String,
    #[serde(default)]
    pub hreflang: Option<String>,
    #[serde(default, rename = "type")]
    pub r#type: Option<String>,
    #[serde(default)]
    pub sizes: Option<String>,
}

/// Extra JSON-LD graph node. Serialized into a script element.
#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct HeadJsonLd {
    #[serde(default)]
    pub key: Option<String>,
    /// Serialized JSON object (not a full `@graph` document).
    pub json: String,
}

/// `hreflang` alternate.
#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct HeadAlternate {
    pub lang: String,
    pub href: String,
}

/// Site-level defaults that pages inherit.
#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct SiteHead {
    #[serde(default)]
    pub name: Option<String>,
    #[serde(default)]
    pub url: Option<String>,
    #[serde(default)]
    pub locale: Option<String>,
    /// `%s` is the page title. `%siteName` comes from [`Self::name`].
    #[serde(default, rename = "titleTemplate")]
    pub title_template: Option<String>,
}

/// Typed page-head input shared by themed, bare, custom, and `ssg: false` hosts.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct HeadInput {
    #[serde(default)]
    pub site: SiteHead,
    #[serde(default)]
    pub title: Option<String>,
    #[serde(default, rename = "titleTemplate")]
    pub title_template: Option<String>,
    /// When true, suffix `site.name` as `{title} - {site}` when they differ.
    #[serde(default = "default_true", rename = "titleSuffix")]
    pub title_suffix: bool,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default)]
    pub canonical: Option<String>,
    #[serde(default)]
    pub robots: Option<String>,
    #[serde(default, rename = "ogImage")]
    pub og_image: Option<String>,
    #[serde(default, rename = "ogType")]
    pub og_type: Option<String>,
    #[serde(default, rename = "twitterCard")]
    pub twitter_card: Option<String>,
    /// When true, emit the built-in OG/Twitter pair for title/description/image.
    #[serde(default = "default_true")]
    pub social: bool,
    /// When true, emit `og:site_name` from [`SiteHead::name`].
    #[serde(default, rename = "emitSiteName")]
    pub emit_site_name: bool,
    /// When true, emit built-in URL fields without scheme checks.
    #[serde(default)]
    pub trusted: bool,
    #[serde(default)]
    pub metas: Vec<HeadMeta>,
    #[serde(default)]
    pub links: Vec<HeadLink>,
    #[serde(default)]
    pub alternates: Vec<HeadAlternate>,
    #[serde(default, rename = "jsonLd")]
    pub json_ld: Vec<HeadJsonLd>,
    #[serde(default)]
    pub validation: HeadValidation,
}

fn default_true() -> bool {
    true
}

impl Default for HeadInput {
    fn default() -> Self {
        Self {
            site: SiteHead::default(),
            title: None,
            title_template: None,
            title_suffix: true,
            description: None,
            canonical: None,
            robots: None,
            og_image: None,
            og_type: None,
            twitter_card: None,
            social: true,
            emit_site_name: false,
            trusted: false,
            metas: Vec::new(),
            links: Vec::new(),
            alternates: Vec::new(),
            json_ld: Vec::new(),
            validation: HeadValidation::Off,
        }
    }
}

impl HeadInput {
    pub(super) fn document_title(&self) -> String {
        let page = self.title.as_deref().unwrap_or("").trim();
        let site = self.site.name.as_deref().unwrap_or("").trim();
        let template = self
            .title_template
            .as_deref()
            .or(self.site.title_template.as_deref())
            .map(str::trim)
            .filter(|template| !template.is_empty());
        if let Some(template) = template {
            return template.replace("%s", page).replace("%siteName", site);
        }
        if page.is_empty() {
            return site.to_string();
        }
        if !self.title_suffix || site.is_empty() || page == site {
            return page.to_string();
        }
        format!("{page} - {site}")
    }
}
