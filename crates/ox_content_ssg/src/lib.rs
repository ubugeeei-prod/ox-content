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
//! };
//!
//! let html = generate_html(&page_data, &nav_groups, &config);
//! ```

mod assets;
mod html;
mod routes;

pub use assets::{
    externalize_shared_page_assets, ExternalizedAssets, GeneratedHtmlPage, SharedAsset,
};
pub use html::{
    generate_html, EntryPageConfig, FeatureConfig, HeroAction, HeroConfig, HeroImage,
    HeroNoticeConfig, LocaleInfo, NavGroup, NavItem, PageData, SocialLink, SocialLinks, SsgConfig,
    ThemeColors, ThemeConfig, ThemeEmbed, ThemeEntryPage, ThemeFonts, ThemeFooter, ThemeHeader,
    ThemeLayout, TocEntry,
};
pub use routes::{
    build_nav_items, build_theme_nav_items, collect_markdown_files, extract_title, format_title,
    get_href, get_og_image_path, get_og_image_url, get_output_path, get_page_locale, get_url_path,
    resolve_navigation_groups, resolve_route_paths, ManualNavigationGroup, ManualNavigationItem,
    RoutePaths, SidebarItem,
};
