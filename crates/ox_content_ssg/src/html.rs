//! HTML page generation for SSG.

use askama::Template;

mod a11y;
mod aside;
mod bare;
mod breadcrumbs;
mod content_css;
mod entry;
mod footer;
mod head;
mod header_chrome;
mod heading_permalinks;
mod json_ld;
mod locale_switcher;
mod mpa_navigation;
mod nav;
mod page;
mod pagination;
mod reader_chrome;
mod render;
mod render_inner;
mod section_index;
mod social;
mod team;
mod theme;
mod theme_css;
mod urls;
mod utils;

pub use a11y::A11y;
use breadcrumbs::BreadcrumbsView;
pub use head::{
    HeadAlternate, HeadDiagnostic, HeadInput, HeadJsonLd, HeadLink, HeadMeta, RenderedHead,
    ResolvedHead, ResolvedTag, SiteHead, render_head, resolve_head, serialize_head,
};
pub use header_chrome::{HeaderNavItem, PageChromeFlags, ThemeAnnouncement};
pub use json_ld::{JsonLd, JsonLdPublisher};
use pagination::PagerView;
pub use reader_chrome::ReaderChrome;
pub use section_index::{
    SectionIndexItem, SectionIndexStyle, is_safe_section_href, render_section_index,
};
pub use team::{TeamLink, TeamMember, TeamOptions, render_team_page};

pub use bare::{BarePageData, generate_bare_html, generate_bare_page};
pub use page::{
    Contributor, EntryPageConfig, FeatureConfig, HeadValidation, HeroAction, HeroConfig, HeroImage,
    HeroNoticeConfig, LocaleInfo, LocalePath, NavGroup, NavItem, PageData, PagerOverride,
    SsgConfig, TocEntry,
};
pub use render::{GeneratedHtml, generate_html, generate_html_result};
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

struct ContributorView {
    name: String,
    avatar: Option<String>,
}

/// Main page template.
#[derive(Template)]
#[template(path = "page.html")]
struct PageTemplate<'a> {
    html_lang: &'a str,
    html_dir: &'a str,
    site_name: &'a str,
    page_head: &'a str,
    theme_bootstrap_js: &'a str,
    css: &'a str,
    embed_head: &'a str,
    body_class: &'a str,
    skip_link: Option<&'a str>,
    embed_header_before: &'a str,
    announcement_html: &'a str,
    show_navbar: bool,
    header_nav_html: &'a str,
    embed_header_after: &'a str,
    base: &'a str,
    logo_src: &'a str,
    logo_light_src: Option<&'a str>,
    logo_dark_src: Option<&'a str>,
    show_site_name_text: bool,
    logo_width: u32,
    logo_height: u32,
    social_links: &'a str,
    locale_switcher: &'a str,
    is_entry_page: bool,
    embed_sidebar_before: &'a str,
    navigation: &'a str,
    embed_sidebar_after: &'a str,
    embed_content_before: &'a str,
    breadcrumbs: Option<&'a BreadcrumbsView>,
    main_content: &'a str,
    has_toc: bool,
    toc_html: &'a str,
    pager: Option<&'a PagerView>,
    reader_chrome: Option<&'a ReaderChrome>,
    last_updated: Option<&'a LastUpdatedView>,
    contributors: &'a [ContributorView],
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
    lang: &'a str,
    dir: &'a str,
    page_head: &'a str,
    content: &'a str,
    head: &'a str,
    body_start: &'a str,
    body_end: &'a str,
}

/// Marker expanded from `ssg.css` so Magic Link rules live in one file.
const MAGIC_LINKS_INCLUDE_MARK: &str = "/* @include magic-links.css */\n";

/// Published Magic Link stylesheet. Inlined into `SSG_CSS` at the same
/// position the rules previously occupied in `ssg.css`.
const MAGIC_LINKS_CSS: &str = include_str!("plugins/magic-links.css");

/// CSS styles for SSG pages, with Magic Link rules spliced in.
static SSG_CSS: std::sync::LazyLock<String> = std::sync::LazyLock::new(|| {
    let template = include_str!("ssg.css");
    assert!(
        template.contains(MAGIC_LINKS_INCLUDE_MARK),
        "ssg.css must include the magic-links marker so SSG and published CSS stay aligned"
    );
    template.replacen(MAGIC_LINKS_INCLUDE_MARK, MAGIC_LINKS_CSS, 1)
});

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

/// CSS styles for social embeds (Twitter/X, Bluesky, WebContainer, media iframes).
const SOCIAL_CSS: &str =
    concat!(include_str!("plugins/social.css"), include_str!("plugins/social-twitter-rich.css"));

/// CSS for opt-in full-fidelity Tweet cards (`.ox-tweet--full`).
const SOCIAL_TWEET_FULL_CSS: &str = concat!(
    include_str!("plugins/social-tweet-full.css"),
    include_str!("plugins/social-tweet-full-media.css"),
);

/// CSS styles for Mermaid plugin.
const MERMAID_CSS: &str = include_str!("plugins/mermaid.css");

/// CSS styles for Island plugin.
const ISLAND_CSS: &str = include_str!("plugins/island.css");

/// CSS styles for the opt-in git contributor list.
const CONTRIBUTORS_CSS: &str = include_str!("html/contributors.css");

/// CSS styles for opt-in `file-tree` fences.
const FILE_TREE_CSS: &str = include_str!("html/file_tree.css");

/// CSS styles for opt-in `{kbd:...}` keyboard keys.
const KBD_CSS: &str = include_str!("plugins/kbd.css");

/// JavaScript for SSG pages.
const SSG_JS: &str = include_str!("ssg.js");

/// Client runtime for opt-in synced tab groups. Only acts on tab groups that
/// carry a `data-ox-tab-group` attribute (emitted when syncing is enabled), so
/// it is inert for the default no-JavaScript tab widget.
const TABS_JS: &str = include_str!("plugins/tabs.js");

#[cfg(test)]
mod tests;
