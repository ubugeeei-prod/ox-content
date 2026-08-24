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
//! use ox_content_ssg::{generate_html, PageData, NavGroup, NavItem, SsgConfig, TocEntry};
//!
//! let page_data = PageData {
//!     title: "Getting Started".to_string(),
//!     description: Some("Learn how to use ox-content".to_string()),
//!     content: "<h1>Getting Started</h1><p>Welcome!</p>".to_string(),
//!     toc: vec![TocEntry { depth: 1, text: "Getting Started".to_string(), slug: "getting-started".to_string() }],
//!     last_updated: None,
//!     path: "getting-started".to_string(),
//!     entry_page: None,
//!     prev: None,
//!     next: None,
//!     breadcrumbs: None,
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
//!     og_image: None,
//!     theme: None,
//!     pagination: false,
//!     breadcrumbs: false,
//!     reader_chrome: Default::default(),
//!     locale_switcher: false,
//!     locale_paths: vec![],
//!     a11y: Default::default(),
//! };
//!
//! let html = generate_html(&page_data, &nav_groups, &config);
//! ```

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
pub use feeds::{FeedFormat, FeedItem, FeedsOptions, FeedsOutput, generate_feeds};
pub use html::{
    A11y, BarePageData, EntryPageConfig, FeatureConfig, HeroAction, HeroConfig, HeroImage,
    HeroNoticeConfig, LocaleInfo, LocalePath, NavGroup, NavItem, PageData, PagerOverride,
    ReaderChrome, SocialLink, SocialLinks, SsgConfig, TeamLink, TeamMember, TeamOptions,
    ThemeColors, ThemeConfig, ThemeEmbed, ThemeEntryPage, ThemeFonts, ThemeFooter, ThemeHeader,
    ThemeLayout, TocEntry, generate_bare_html, generate_bare_page, generate_html, render_team_page,
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
    ManualNavigationGroup, ManualNavigationItem, RoutePaths, SidebarItem, build_nav_items,
    build_theme_nav_items, collect_markdown_files, extract_title, format_title, get_href,
    get_og_image_path, get_og_image_url, get_output_path, get_page_locale, get_url_path,
    resolve_navigation_groups, resolve_route_paths,
};
pub use site_maps::{SiteMapPage, SiteMapsOptions, SiteMapsOutput, generate_site_maps};
pub use vitepress::normalize_vitepress_frontmatter;
