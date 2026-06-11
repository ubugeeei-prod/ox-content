//! HTML page generation for SSG.

use askama::Template;

mod entry;
mod footer;
mod nav;
mod page;
mod render;
mod social;
mod theme;
mod theme_css;
mod utils;

pub use page::{
    EntryPageConfig, FeatureConfig, HeroAction, HeroConfig, HeroImage, HeroNoticeConfig,
    LocaleInfo, NavGroup, NavItem, PageData, SsgConfig, TocEntry,
};
pub use render::{generate_bare_html, generate_html};
pub use theme::{
    SocialLink, SocialLinks, ThemeColors, ThemeConfig, ThemeEmbed, ThemeEntryPage, ThemeFonts,
    ThemeFooter, ThemeHeader, ThemeLayout,
};

// =============================================================================
// Askama Template Structures
// =============================================================================

/// Social links template (desktop header).
#[derive(Template)]
#[template(path = "social_links.html")]
struct SocialLinksTemplate<'a> {
    github: Option<&'a str>,
    twitter: Option<&'a str>,
    discord: Option<&'a str>,
}

/// Mobile social links template (mobile footer).
#[derive(Template)]
#[template(path = "mobile_social_links.html")]
struct MobileSocialLinksTemplate<'a> {
    github: Option<&'a str>,
    twitter: Option<&'a str>,
    discord: Option<&'a str>,
}

/// Footer template.
#[derive(Template)]
#[template(path = "footer.html")]
struct FooterTemplate<'a> {
    message: Option<&'a str>,
    copyright: Option<&'a str>,
}

/// Hero action for entry template.
pub struct HeroActionView {
    pub href: String,
    pub theme_class: String,
    pub text: String,
}

/// Feature card for entry template.
pub struct FeatureView {
    pub tag: &'static str,
    pub href_attr: String,
    pub icon_html: Option<String>,
    pub title: String,
    pub details: Option<String>,
    pub has_link: bool,
}

/// Hero view for entry template.
pub struct HeroView {
    pub name: Option<String>,
    pub text: Option<String>,
    pub tagline: Option<String>,
    pub notice: Option<HeroNoticeConfig>,
    pub image: Option<HeroImage>,
    pub actions: Option<Vec<HeroActionView>>,
}

/// Entry page template (hero + features).
#[derive(Template)]
#[template(path = "entry.html")]
struct EntryTemplate<'a> {
    hero: Option<&'a HeroView>,
    features: Option<&'a [FeatureView]>,
}

struct LastUpdatedView {
    text: String,
    datetime: String,
}

/// Main page template.
#[derive(Template)]
#[template(path = "page.html")]
struct PageTemplate<'a> {
    html_lang: &'a str,
    html_dir: &'a str,
    site_name: &'a str,
    document_title: &'a str,
    description: Option<&'a str>,
    og_image: Option<&'a str>,
    css: &'a str,
    embed_head: &'a str,
    body_class: &'a str,
    embed_header_before: &'a str,
    embed_header_after: &'a str,
    base: &'a str,
    logo_src: &'a str,
    logo_light_src: Option<&'a str>,
    logo_dark_src: Option<&'a str>,
    show_site_name_text: bool,
    logo_width: u32,
    logo_height: u32,
    social_links: &'a str,
    is_entry_page: bool,
    embed_sidebar_before: &'a str,
    navigation: &'a str,
    embed_sidebar_after: &'a str,
    embed_content_before: &'a str,
    main_content: &'a str,
    has_toc: bool,
    toc_html: &'a str,
    last_updated: Option<&'a LastUpdatedView>,
    embed_content_after: &'a str,
    embed_footer_before: &'a str,
    footer_html: &'a str,
    mobile_social_links: &'a str,
    js: &'a str,
}

/// Bare page template (no navigation, no styles).
#[derive(Template)]
#[template(path = "bare_page.html")]
struct BarePageTemplate<'a> {
    title: &'a str,
    content: &'a str,
}

/// CSS styles for SSG pages.
const SSG_CSS: &str = include_str!("ssg.css");

/// CSS styles for Entry pages (hero, features).
const ENTRY_CSS: &str = include_str!("entry.css");

/// CSS styles for Tabs plugin.
const TABS_CSS: &str = include_str!("plugins/tabs.css");

/// CSS styles for YouTube plugin.
const YOUTUBE_CSS: &str = include_str!("plugins/youtube.css");

/// CSS styles for GitHub plugin.
const GITHUB_CSS: &str = include_str!("plugins/github.css");

/// CSS styles for OGP plugin.
const OGP_CSS: &str = include_str!("plugins/ogp.css");

/// CSS styles for Mermaid plugin.
const MERMAID_CSS: &str = include_str!("plugins/mermaid.css");

/// CSS styles for Island plugin.
const ISLAND_CSS: &str = include_str!("plugins/island.css");

/// JavaScript for SSG pages.
const SSG_JS: &str = include_str!("ssg.js");

/// Client runtime for opt-in synced tab groups. Only acts on tab groups that
/// carry a `data-ox-tab-group` attribute (emitted when syncing is enabled), so
/// it is inert for the default no-JavaScript tab widget.
const TABS_JS: &str = include_str!("plugins/tabs.js");

#[cfg(test)]
mod tests;
