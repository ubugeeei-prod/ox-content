//! Routing and navigation helpers for SSG builds.

mod files;
mod manual;
mod navigation;
mod path;
mod sidebar;

pub use files::{collect_markdown_files, extract_title, format_title};
pub use manual::{ManualNavigationGroup, ManualNavigationItem, resolve_navigation_groups};
pub use navigation::build_nav_items;
pub use path::{
    RoutePaths, get_href, get_og_image_path, get_og_image_url, get_output_path, get_page_locale,
    get_url_path, resolve_route_paths,
};
pub use sidebar::{SidebarItem, build_theme_nav_items};

const DEFAULT_NAV_GROUP_ORDER: &[&str] = &["", "examples", "packages", "api"];
const DEFAULT_ROOT_NAV_TITLE: &str = "Overview";
const DEFAULT_ROOT_GROUP_TITLE: &str = "Guide";
const DEFAULT_INDEX_TITLE: &str = "Home";
const DEFAULT_UNTITLED_TITLE: &str = "Untitled";
