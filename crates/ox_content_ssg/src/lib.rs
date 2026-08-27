//! Static Site Generation for Ox Content.
//!
//! This crate provides HTML page generation for documentation sites,
//! including navigation, table of contents, search functionality,
//! and theming support.
//!
//! # Features
//!
//! - Full HTML page generation with responsive layout
//! - Navigation sidebar with grouping
//! - Table of contents generation
//! - Client-side search integration
//! - Dark/light theme support
//! - Mobile-friendly responsive design
//! - Customizable theme configuration
//!
//! # Example
//!
//! ```ignore
//! use ox_content_ssg::{generate_html, PageChromeFlags, PageData, NavGroup, NavItem, SsgConfig, TocEntry};
//!
//! let page_data = PageData {
//!     title: "Getting Started".to_string(),
//!     description: Some("Learn how to use ox-content".to_string()),
//!     content: "<h1>Getting Started</h1><p>Welcome!</p>".to_string(),
//!     toc: vec![TocEntry { depth: 1, text: "Getting Started".to_string(), slug: "getting-started".to_string() }],
//!     last_updated: None,
//!     contributors: vec![],
//!     path: "getting-started".to_string(),
//!     entry_page: None,
//!     prev: None,
//!     next: None,
//!     breadcrumbs: None,
//!     chrome: PageChromeFlags::default(),
//!     robots: None,
//!     canonical: None,
//!     markdown_source: None,
//! };
//!
//! let nav_groups = vec![NavGroup {
//!     title: "Guide".to_string(),
//!     items: vec![NavItem {
//!         title: "Getting Started".to_string(),
//!         path: "getting-started".to_string(),
//!         href: "/docs/getting-started/index.html".to_string(),
//!         children: vec![],
//!         collapsed: None,
//!     }],
//!     collapsed: None,
//! }];
//!
//! let config = SsgConfig {
//!     site_name: "My Docs".to_string(),
//!     base: "/docs/".to_string(),
//!     breadcrumb_root_href: None,
//!     og_image: None,
//!     theme: None,
//!     pagination: false,
//!     breadcrumbs: false,
//!     reader_chrome: Default::default(),
//!     locale_switcher: false,
//!     locale_paths: vec![],
//!     a11y: Default::default(),
//!     page_chrome: false,
//!     json_ld: Default::default(),
//! };
//!
//! let html = generate_html(&page_data, &nav_groups, &config);
//! ```

#![cfg_attr(
    not(test),
    deny(
        clippy::unwrap_used,
        clippy::expect_used,
        clippy::panic,
        clippy::todo,
        clippy::unimplemented
    )
)]

mod assets;
mod feeds;
mod html;
mod permalinks;
mod redirects;
mod routes;
mod site_maps;
mod vitepress;

pub use assets::{
    ExternalizedAssets, GeneratedHtmlPage, SharedAsset, externalize_shared_page_assets,
};
pub use feeds::{
    FeedAttachment, FeedAuthor, FeedFormat, FeedItem, FeedsOptions, FeedsOutput, ParsedDate,
    generate_feeds, parse_date,
};
pub use html::{
    A11y, BarePageData, Contributor, EntryPageConfig, FeatureConfig, GeneratedHtml, HeadAlternate,
    HeadDiagnostic, HeadInput, HeadJsonLd, HeadLink, HeadMeta, HeadValidation, HeaderNavItem,
    HeroAction, HeroConfig, HeroImage, HeroNoticeConfig, JsonLd, JsonLdPublisher, LocaleInfo,
    LocalePath, NavGroup, NavItem, PageChromeFlags, PageData, PagerOverride, ReaderChrome,
    RenderedHead, ResolvedHead, ResolvedTag, SectionIndexItem, SectionIndexStyle, SiteHead,
    SocialLink, SocialLinks, SsgConfig, TeamLink, TeamMember, TeamOptions, ThemeAnnouncement,
    ThemeColors, ThemeConfig, ThemeEmbed, ThemeEntryPage, ThemeFonts, ThemeFooter, ThemeHeader,
    ThemeLayout, TocEntry, generate_bare_html, generate_bare_page, generate_html,
    generate_html_result, is_safe_section_href, render_head, render_section_index,
    render_team_page, resolve_head, serialize_head,
};
pub use permalinks::{
    CascadeOptions, PermalinksOptions, ResolvedRoutePage, RoutePage, RouteResolveOutput,
    apply_cascade, escape_attribute, is_safe_permalink, resolve_page_routes,
};
pub use redirects::{
    RedirectEntry, RedirectPage, RedirectsOptions, RedirectsOutput, generate_redirect_html,
    generate_redirects, is_safe_dest, normalize_path,
};
pub use routes::{
    ManualNavigationGroup, ManualNavigationItem, RoutePaths, SidebarItem, apply_route_prefix,
    build_nav_items, build_theme_nav_items, collect_markdown_files, extract_title, format_title,
    get_href, get_og_image_path, get_og_image_url, get_output_path, get_page_locale, get_url_path,
    normalize_route_prefix, resolve_navigation_groups, resolve_route_paths,
};
pub use site_maps::{SiteMapPage, SiteMapsOptions, SiteMapsOutput, generate_site_maps};
pub use vitepress::normalize_vitepress_frontmatter;
