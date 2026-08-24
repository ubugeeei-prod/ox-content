use serde::{Deserialize, Serialize};

use super::ThemeConfig;
use super::a11y::A11y;
use super::header_chrome::PageChromeFlags;
use super::reader_chrome::ReaderChrome;

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

/// Frontmatter override for one side of the previous/next pager.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct PagerOverride {
    /// Hide this side when true (`prev: false` / `next: false`).
    #[serde(default)]
    pub hidden: bool,
    /// Replacement title (`text` or `title` in frontmatter).
    #[serde(default)]
    pub text: Option<String>,
    /// Replacement href (`link` or `href` in frontmatter).
    #[serde(default)]
    pub href: Option<String>,
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
    /// Frontmatter override for the previous-page link.
    #[serde(default)]
    pub prev: Option<PagerOverride>,
    /// Frontmatter override for the next-page link.
    #[serde(default)]
    pub next: Option<PagerOverride>,
    /// Frontmatter `breadcrumbs: false` hides the trail on this page.
    #[serde(default)]
    pub breadcrumbs: Option<bool>,
    /// Per-page chrome flags. Honored only when `SsgConfig::page_chrome` is on.
    #[serde(default)]
    pub chrome: PageChromeFlags,
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
    /// When true, render previous/next page links after the article.
    #[serde(default)]
    pub pagination: bool,
    /// When true, render a breadcrumb trail above the article.
    #[serde(default)]
    pub breadcrumbs: bool,
    /// Opt-in copy, external-link, and back-to-top chrome. Off by default.
    #[serde(default)]
    pub reader_chrome: ReaderChrome,
    /// Opt-in header locale switcher. Off unless explicitly enabled.
    #[serde(default)]
    pub locale_switcher: bool,
    /// Existing sibling hrefs and locale roots, keyed by locale code.
    #[serde(default)]
    pub locale_paths: Vec<LocalePath>,
    /// Opt-in skip link and print styles. Off by default.
    #[serde(default)]
    pub a11y: A11y,
    /// When true, honor per-page frontmatter chrome flags. Off by default.
    #[serde(default)]
    pub page_chrome: bool,
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

/// Sibling page or locale-root href for one locale in the switcher.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct LocalePath {
    /// BCP 47 locale tag matching [`LocaleInfo::code`].
    pub code: String,
    /// Href of the same page in this locale, when that translation exists.
    #[serde(default)]
    pub href: Option<String>,
    /// Locale home href used when `href` is missing.
    #[serde(default)]
    pub root: Option<String>,
}
