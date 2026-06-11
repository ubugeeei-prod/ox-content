use serde::{Deserialize, Serialize};

use super::ThemeConfig;

/// Hero action button.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct HeroAction {
    /// Button theme: "brand" or "alt".
    pub theme: Option<String>,
    /// Button text.
    pub text: String,
    /// Link URL.
    pub link: String,
}

/// Hero image configuration.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct HeroImage {
    /// Image source URL.
    pub src: String,
    /// Light mode image source URL.
    #[serde(rename = "lightSrc")]
    pub light_src: Option<String>,
    /// Dark mode image source URL.
    #[serde(rename = "darkSrc")]
    pub dark_src: Option<String>,
    /// Alt text.
    pub alt: Option<String>,
    /// Image width.
    pub width: Option<u32>,
    /// Image height.
    pub height: Option<u32>,
}

/// Hero section configuration.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct HeroConfig {
    /// Main title (large, gradient text).
    pub name: Option<String>,
    /// Secondary text.
    pub text: Option<String>,
    /// Tagline.
    pub tagline: Option<String>,
    /// Optional notice shown in the hero.
    pub notice: Option<HeroNoticeConfig>,
    /// Hero image.
    pub image: Option<HeroImage>,
    /// Action buttons.
    pub actions: Option<Vec<HeroAction>>,
}

/// Hero notice configuration.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct HeroNoticeConfig {
    /// Notice title.
    pub title: Option<String>,
    /// Notice paragraphs.
    pub body: Option<Vec<String>>,
}

/// Feature card configuration.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct FeatureConfig {
    /// Icon - supports: "mdi:icon-name" (Iconify), image URL, or emoji.
    pub icon: Option<String>,
    /// Feature title.
    pub title: String,
    /// Feature description.
    pub details: Option<String>,
    /// Optional link.
    pub link: Option<String>,
    /// Link text.
    pub link_text: Option<String>,
}

/// Entry page configuration (for landing pages with hero and features).
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct EntryPageConfig {
    /// Hero section.
    pub hero: Option<HeroConfig>,
    /// Feature cards.
    pub features: Option<Vec<FeatureConfig>>,
}

/// Navigation item for SSG.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NavItem {
    /// Display title.
    pub title: String,
    /// URL path.
    pub path: String,
    /// Full href.
    pub href: String,
    #[serde(default)]
    pub children: Vec<NavItem>,
    #[serde(default)]
    pub collapsed: Option<bool>,
    #[serde(default)]
    pub sticky_collapsed: Option<bool>,
}

/// Navigation group for SSG.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NavGroup {
    /// Group title.
    pub title: String,
    /// Navigation items.
    pub items: Vec<NavItem>,
    #[serde(default)]
    pub collapsed: Option<bool>,
    #[serde(default)]
    pub sticky_collapsed: Option<bool>,
}

/// Table of contents entry.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TocEntry {
    /// Heading depth (1-6).
    pub depth: u8,
    /// Heading text.
    pub text: String,
    /// URL-friendly slug.
    pub slug: String,
}

/// Page data for SSG.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PageData {
    /// Page title.
    pub title: String,
    /// Page description.
    pub description: Option<String>,
    /// Page content HTML.
    pub content: String,
    /// Table of contents entries.
    pub toc: Vec<TocEntry>,
    /// Last updated timestamp in milliseconds since the Unix epoch.
    pub last_updated: Option<i64>,
    /// URL path.
    pub path: String,
    /// Entry page configuration (if layout: entry).
    pub entry_page: Option<EntryPageConfig>,
}

/// SSG configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SsgConfig {
    /// Site name.
    pub site_name: String,
    /// Base URL path.
    pub base: String,
    /// OG image URL.
    pub og_image: Option<String>,
    /// Theme configuration.
    pub theme: Option<ThemeConfig>,
    /// Current locale (BCP 47 tag) for this page, if i18n is enabled.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub locale: Option<String>,
    /// All available locales (for generating locale switcher and hreflang tags).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub available_locales: Option<Vec<LocaleInfo>>,
}

/// Locale information for the locale switcher.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LocaleInfo {
    /// BCP 47 locale tag.
    pub code: String,
    /// Display name.
    pub name: String,
    /// Text direction.
    pub dir: String,
}
