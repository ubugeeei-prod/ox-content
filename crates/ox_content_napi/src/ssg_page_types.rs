use napi_derive::napi;

use crate::TocEntry;

/// Navigation item for SSG.
#[napi(object)]
#[derive(Clone)]
pub struct JsSsgNavItem {
    /// Display title.
    pub title: String,
    /// URL path.
    pub path: String,
    /// Full href.
    pub href: String,
    pub children: Option<Vec<JsSsgNavItem>>,
    pub collapsed: Option<bool>,
    pub sticky_collapsed: Option<bool>,
}

/// Navigation group for SSG.
#[napi(object)]
#[derive(Clone)]
pub struct JsSsgNavGroup {
    /// Group title.
    pub title: String,
    /// Navigation items.
    pub items: Vec<JsSsgNavItem>,
    pub collapsed: Option<bool>,
    pub sticky_collapsed: Option<bool>,
}

/// Head metadata and injected markup for a bare SSG page.
///
/// Every field is optional. A value with none of them set renders the same
/// document bare mode emitted before any of this existed.
#[napi(object)]
pub struct JsSsgBarePage {
    /// Page title.
    pub title: String,
    /// Rendered page body.
    pub content: String,
    /// `lang` attribute. Defaults to `en`.
    pub lang: Option<String>,
    /// `dir` attribute. Omitted when absent.
    pub dir: Option<String>,
    /// Page description for `description` and the OG/Twitter variants.
    pub description: Option<String>,
    /// Absolute page URL for `<link rel="canonical">` and `og:url`.
    pub canonical_url: Option<String>,
    /// Site name for `og:site_name`.
    pub site_name: Option<String>,
    /// Image URL for `og:image` and `twitter:image`.
    pub og_image: Option<String>,
    /// Raw markup appended to `<head>`.
    pub head: Option<String>,
    /// Raw markup inserted directly after `<body>`.
    pub body_start: Option<String>,
    /// Raw markup inserted directly before `</body>`.
    pub body_end: Option<String>,
}

/// Resolved SSG output and public route paths.
#[napi(object)]
pub struct JsSsgRoutePaths {
    /// HTML output file path.
    pub output_path: String,
    /// Route path without extension.
    pub url_path: String,
    /// Public HTML href.
    pub href: String,
    /// OG image output file path.
    pub og_image_path: String,
    /// OG image public URL.
    pub og_image_url: String,
}

/// Theme sidebar item for SSG navigation generation.
#[napi(object)]
#[derive(Clone, Default)]
pub struct JsSsgSidebarItem {
    /// Display text.
    pub text: Option<String>,
    /// Link URL or route path.
    pub link: Option<String>,
    /// Child sidebar items.
    pub items: Option<Vec<JsSsgSidebarItem>>,
    /// Whether this group is collapsed by default.
    pub collapsed: Option<bool>,
    /// Whether this group's open state persists across page navigations.
    pub sticky_collapsed: Option<bool>,
}

/// Manual SSG navigation item supplied by user configuration.
#[napi(object)]
#[derive(Clone)]
pub struct JsSsgNavigationItem {
    pub title: String,
    pub path: Option<String>,
    pub href: Option<String>,
}

/// Manual SSG navigation group supplied by user configuration.
#[napi(object)]
#[derive(Clone)]
pub struct JsSsgNavigationGroup {
    pub title: String,
    pub items: Vec<JsSsgNavigationItem>,
}

/// Generated SSG HTML page for shared asset extraction.
#[napi(object)]
#[derive(Clone)]
pub struct JsSsgGeneratedHtmlPage {
    /// Source Markdown path.
    pub input_path: String,
    /// Output HTML path.
    pub output_path: String,
    /// HTML content.
    pub html: String,
}

/// Shared SSG asset extracted from generated pages.
#[napi(object)]
#[derive(Clone)]
pub struct JsSsgSharedAsset {
    /// Output file path.
    pub output_path: String,
    /// Public URL path used from HTML.
    pub public_path: String,
    /// Asset content.
    pub content: String,
}

/// Result of SSG shared asset extraction.
#[napi(object)]
pub struct JsSsgExternalizedAssets {
    /// HTML pages with inline assets replaced.
    pub pages: Vec<JsSsgGeneratedHtmlPage>,
    /// Extracted shared assets.
    pub assets: Vec<JsSsgSharedAsset>,
}

/// Hero action for entry page.
#[napi(object)]
#[derive(Clone, Default)]
pub struct JsHeroAction {
    /// Button theme: "brand" or "alt".
    pub theme: Option<String>,
    /// Button text.
    pub text: String,
    /// Link URL.
    pub link: String,
}

/// Hero image for entry page.
#[napi(object)]
#[derive(Clone, Default)]
pub struct JsHeroImage {
    /// Image source URL.
    pub src: String,
    /// Light mode image source URL.
    pub light_src: Option<String>,
    /// Dark mode image source URL.
    pub dark_src: Option<String>,
    /// Alt text.
    pub alt: Option<String>,
    /// Image width.
    pub width: Option<u32>,
    /// Image height.
    pub height: Option<u32>,
}

/// Hero notice for entry page.
#[napi(object)]
#[derive(Clone, Default)]
pub struct JsHeroNotice {
    /// Notice title.
    pub title: Option<String>,
    /// Notice paragraphs.
    pub body: Option<Vec<String>>,
}

/// Hero section configuration for entry page.
#[napi(object)]
#[derive(Clone, Default)]
pub struct JsHeroConfig {
    /// Main title (large, gradient text).
    pub name: Option<String>,
    /// Secondary text.
    pub text: Option<String>,
    /// Tagline.
    pub tagline: Option<String>,
    /// Optional notice shown in the hero.
    pub notice: Option<JsHeroNotice>,
    /// Hero image.
    pub image: Option<JsHeroImage>,
    /// Action buttons.
    pub actions: Option<Vec<JsHeroAction>>,
}

/// Feature card for entry page.
#[napi(object)]
#[derive(Clone, Default)]
pub struct JsFeatureConfig {
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

/// Entry page configuration.
#[napi(object)]
#[derive(Clone, Default)]
pub struct JsEntryPageConfig {
    /// Hero section.
    pub hero: Option<JsHeroConfig>,
    /// Feature cards.
    pub features: Option<Vec<JsFeatureConfig>>,
}

/// Frontmatter override for one previous/next pager side.
#[napi(object)]
#[derive(Clone, Default)]
pub struct JsPagerOverride {
    /// Hide this side when true.
    pub hidden: Option<bool>,
    /// Replacement title.
    pub text: Option<String>,
    /// Replacement href.
    pub href: Option<String>,
}

/// One unique git author returned by `getGitContributors`.
#[napi(object)]
#[derive(Clone, Debug, Default)]
pub struct JsGitContributor {
    /// Author name from `%an`.
    pub name: String,
    /// Author email from `%ae`. Never rendered into `href`.
    pub email: Option<String>,
    /// Number of commits attributed to this author on the file.
    pub commits: Option<u32>,
}

/// One rendered git author on an SSG page.
#[napi(object)]
#[derive(Clone, Default)]
pub struct JsSsgContributor {
    /// Display name. Escaped in HTML.
    pub name: String,
    /// Optional `https:` avatar URL. Emails are never accepted here.
    pub avatar: Option<String>,
}

/// Page data for SSG.
#[napi(object)]
pub struct JsSsgPageData {
    /// Page title.
    pub title: String,
    /// Page description.
    pub description: Option<String>,
    /// Page content HTML.
    pub content: String,
    /// Table of contents entries.
    pub toc: Vec<TocEntry>,
    /// Last updated timestamp in milliseconds since the Unix epoch.
    pub last_updated: Option<f64>,
    /// Unique git authors for this page. Empty unless the site opted in.
    pub contributors: Option<Vec<JsSsgContributor>>,
    /// URL path.
    pub path: String,
    /// Entry page configuration (if layout: entry).
    pub entry_page: Option<JsEntryPageConfig>,
    /// Frontmatter override for the previous-page link.
    pub prev: Option<JsPagerOverride>,
    /// Frontmatter override for the next-page link.
    pub next: Option<JsPagerOverride>,
    /// Frontmatter `breadcrumbs: false` hides the trail on this page.
    pub breadcrumbs: Option<bool>,
    /// Frontmatter `layout`. `team` renders member cards when the option is on.
    pub layout: Option<String>,
    /// Per-page chrome flags. Honored only when page chrome is enabled.
    pub chrome: Option<JsPageChromeFlags>,
}

/// Per-page frontmatter chrome flags.
#[napi(object)]
#[derive(Clone, Default)]
pub struct JsPageChromeFlags {
    pub sidebar: Option<bool>,
    pub outline: Option<bool>,
    pub aside: Option<bool>,
    pub footer: Option<bool>,
    pub navbar: Option<bool>,
    pub last_updated: Option<bool>,
    pub edit_link: Option<bool>,
}
