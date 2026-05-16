//! Routing and navigation helpers for SSG builds.

use std::cmp::Ordering;
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::{NavGroup, NavItem};

const DEFAULT_NAV_GROUP_ORDER: &[&str] = &["", "examples", "packages", "api"];
const DEFAULT_ROOT_NAV_TITLE: &str = "Overview";
const DEFAULT_ROOT_GROUP_TITLE: &str = "Guide";
const DEFAULT_INDEX_TITLE: &str = "Home";
const DEFAULT_UNTITLED_TITLE: &str = "Untitled";

/// Resolved output and public paths for an SSG page.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct RoutePaths {
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

/// Sidebar item configuration supplied by the theme.
#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq, Eq)]
pub struct SidebarItem {
    /// Display text.
    pub text: Option<String>,
    /// Link URL or route path.
    pub link: Option<String>,
    /// Child sidebar items.
    #[serde(default)]
    pub items: Vec<SidebarItem>,
    /// Whether this group is collapsed by default.
    pub collapsed: Option<bool>,
}

/// Resolves all path variants needed to generate a page.
pub fn resolve_route_paths(
    input_path: &str,
    src_dir: &str,
    out_dir: &str,
    base: &str,
    extension: &str,
    site_url: Option<&str>,
) -> RoutePaths {
    RoutePaths {
        output_path: get_output_path(input_path, src_dir, out_dir, extension),
        url_path: get_url_path(input_path, src_dir),
        href: get_href(input_path, src_dir, base, extension),
        og_image_path: get_og_image_path(input_path, src_dir, out_dir),
        og_image_url: get_og_image_url(input_path, src_dir, base, site_url),
    }
}

/// Converts a markdown file path to its corresponding HTML output path.
pub fn get_output_path(input_path: &str, src_dir: &str, out_dir: &str, extension: &str) -> String {
    let relative_path = relative_path(input_path, src_dir);
    let base_name = replace_markdown_extension(&relative_path, extension);

    if base_name.ends_with(&format!("index{extension}")) {
        return join_path(out_dir, &base_name);
    }

    let dir_name = trim_suffix(&base_name, extension);
    join_path(out_dir, &join_path(&dir_name, &format!("index{extension}")))
}

/// Converts a markdown file path to a relative URL path.
pub fn get_url_path(input_path: &str, src_dir: &str) -> String {
    let relative_path = normalize_separators(&relative_path(input_path, src_dir));
    let base_name = strip_markdown_extension(&relative_path);

    if base_name == "index" || base_name.ends_with("/index") {
        let trimmed = trim_trailing_index(&base_name);
        if trimmed.is_empty() {
            "/".to_string()
        } else {
            trimmed
        }
    } else {
        base_name
    }
}

/// Converts a markdown file path to an HTML href.
pub fn get_href(input_path: &str, src_dir: &str, base: &str, extension: &str) -> String {
    let url_path = get_url_path(input_path, src_dir);
    if url_path == "/" || url_path.is_empty() {
        format!("{base}index{extension}")
    } else {
        format!("{base}{url_path}/index{extension}")
    }
}

/// Gets the OG image output path for a markdown file.
pub fn get_og_image_path(input_path: &str, src_dir: &str, out_dir: &str) -> String {
    let relative_path = normalize_separators(&relative_path(input_path, src_dir));
    let base_name = strip_markdown_extension(&relative_path);

    if base_name == "index" || base_name.ends_with("/index") {
        let dir_path = trim_trailing_index(&base_name);
        return join_path(out_dir, &join_path(&dir_path, "og-image.png"));
    }

    join_path(out_dir, &join_path(&base_name, "og-image.png"))
}

/// Gets the OG image URL for use in meta tags.
pub fn get_og_image_url(
    input_path: &str,
    src_dir: &str,
    base: &str,
    site_url: Option<&str>,
) -> String {
    let url_path = get_url_path(input_path, src_dir);
    let relative_path = if url_path == "/" || url_path.is_empty() {
        format!("{base}og-image.png")
    } else {
        format!("{base}{url_path}/og-image.png")
    };

    match site_url {
        Some(site_url) => format!("{}{}", site_url.trim_end_matches('/'), relative_path),
        None => relative_path,
    }
}

/// Returns the page locale for a localized route.
pub fn get_page_locale(
    url_path: &str,
    default_locale: &str,
    locale_codes: &[String],
) -> Option<String> {
    let first_segment = url_path.split('/').find(|segment| !segment.is_empty());
    match first_segment {
        Some(segment) if locale_codes.iter().any(|locale| locale == segment) => {
            Some(segment.to_string())
        }
        _ => Some(default_locale.to_string()),
    }
}

/// Extracts a display title from frontmatter or a rendered `<h1>`.
pub fn extract_title(content: &str, frontmatter_title: Option<&str>) -> String {
    if let Some(title) = frontmatter_title {
        if !title.is_empty() {
            return title.to_string();
        }
    }

    if let Some(title) = extract_h1_text(content) {
        return title;
    }

    DEFAULT_UNTITLED_TITLE.to_string()
}

/// Formats a file or directory name as a display title.
pub fn format_title(name: &str) -> String {
    let mut chars = name.chars().peekable();
    let mut result = String::new();

    while let Some(ch) = chars.next() {
        if matches!(ch, '-' | '_') {
            match chars.peek().copied() {
                Some(next) if next.is_ascii_lowercase() => {
                    result.push(' ');
                    result.push(next.to_ascii_uppercase());
                    chars.next();
                }
                _ => result.push(ch),
            }
        } else {
            result.push(ch);
        }
    }

    if let Some(first) = result.chars().next().filter(char::is_ascii_lowercase) {
        let uppercase = first.to_ascii_uppercase().to_string();
        result.replace_range(0..first.len_utf8(), &uppercase);
    }

    result
}

/// Builds navigation groups from markdown files.
pub fn build_nav_items(
    markdown_files: &[String],
    src_dir: &str,
    base: &str,
    extension: &str,
) -> Vec<NavGroup> {
    let mut groups: BTreeMap<String, Vec<NavItem>> = BTreeMap::new();

    for file in markdown_files {
        let relative_path = normalize_separators(&relative_path(file, src_dir));
        let group_key = relative_path.split('/').next().filter(|_| relative_path.contains('/'));
        let group_key = group_key.unwrap_or("").to_string();
        let url_path = get_url_path(file, src_dir);
        let title = if url_path == "/" || url_path.is_empty() {
            DEFAULT_ROOT_NAV_TITLE.to_string()
        } else {
            get_display_title(file)
        };

        groups.entry(group_key).or_default().push(NavItem {
            title,
            path: url_path,
            href: get_href(file, src_dir, base, extension),
            children: Vec::new(),
            collapsed: None,
        });
    }

    let mut result = Vec::new();
    for key in DEFAULT_NAV_GROUP_ORDER {
        if let Some(items) = groups.remove(*key) {
            if !items.is_empty() {
                result.push(NavGroup {
                    title: if key.is_empty() {
                        DEFAULT_ROOT_GROUP_TITLE.to_string()
                    } else {
                        format_title(key)
                    },
                    items: sort_nav_items(items),
                    collapsed: None,
                });
            }
        }
    }

    for (key, items) in groups {
        if !items.is_empty() {
            result.push(NavGroup {
                title: format_title(&key),
                items: sort_nav_items(items),
                collapsed: None,
            });
        }
    }

    result
}

/// Builds navigation groups from an explicit theme sidebar tree.
pub fn build_theme_nav_items(
    sidebar: &[SidebarItem],
    base: &str,
    extension: &str,
) -> Vec<NavGroup> {
    fn to_nav_item(item: &SidebarItem, base: &str, extension: &str) -> NavItem {
        NavItem {
            title: item
                .text
                .clone()
                .or_else(|| item.link.clone())
                .unwrap_or_else(|| DEFAULT_UNTITLED_TITLE.to_string()),
            path: sidebar_path(item.link.as_deref()),
            href: sidebar_href(item.link.as_deref(), base, extension),
            children: item.items.iter().map(|child| to_nav_item(child, base, extension)).collect(),
            collapsed: item.collapsed,
        }
    }

    let mut groups = Vec::new();
    let mut loose_items = Vec::new();

    for item in sidebar {
        if !item.items.is_empty() && item.link.is_none() {
            flush_loose_items(&mut groups, &mut loose_items);
            groups.push(NavGroup {
                title: item.text.clone().unwrap_or_else(|| DEFAULT_ROOT_GROUP_TITLE.to_string()),
                items: item.items.iter().map(|child| to_nav_item(child, base, extension)).collect(),
                collapsed: item.collapsed,
            });
        } else {
            loose_items.push(to_nav_item(item, base, extension));
        }
    }

    flush_loose_items(&mut groups, &mut loose_items);
    groups
}

fn flush_loose_items(groups: &mut Vec<NavGroup>, loose_items: &mut Vec<NavItem>) {
    if !loose_items.is_empty() {
        groups.push(NavGroup {
            title: DEFAULT_ROOT_GROUP_TITLE.to_string(),
            items: std::mem::take(loose_items),
            collapsed: None,
        });
    }
}

fn relative_path(input_path: &str, src_dir: &str) -> String {
    let input = Path::new(input_path);
    let src = Path::new(src_dir);
    input.strip_prefix(src).unwrap_or(input).to_string_lossy().into_owned()
}

fn normalize_separators(value: &str) -> String {
    value.replace('\\', "/")
}

fn join_path(left: &str, right: &str) -> String {
    if left.is_empty() {
        return right.to_string();
    }
    if right.is_empty() {
        return left.to_string();
    }
    PathBuf::from(left).join(right).to_string_lossy().into_owned()
}

fn strip_markdown_extension(path: &str) -> String {
    if path.len() >= 3 && path[path.len() - 3..].eq_ignore_ascii_case(".md") {
        return path[..path.len() - 3].to_string();
    }
    if path.len() >= 9 && path[path.len() - 9..].eq_ignore_ascii_case(".markdown") {
        return path[..path.len() - 9].to_string();
    }
    path.to_string()
}

fn replace_markdown_extension(path: &str, extension: &str) -> String {
    format!("{}{}", strip_markdown_extension(path), extension)
}

fn trim_suffix(value: &str, suffix: &str) -> String {
    value.strip_suffix(suffix).unwrap_or(value).to_string()
}

fn trim_trailing_index(value: &str) -> String {
    value
        .strip_suffix("/index")
        .or_else(|| value.strip_suffix("index"))
        .unwrap_or(value)
        .to_string()
}

fn get_display_title(file_path: &str) -> String {
    let path = Path::new(file_path);
    let file_name = path.file_stem().and_then(|name| name.to_str()).unwrap_or_default();

    if file_name == "index" {
        let dir_name = path.parent().and_then(Path::file_name).and_then(|name| name.to_str());
        return dir_name
            .filter(|name| !name.is_empty() && *name != ".")
            .map_or_else(|| DEFAULT_INDEX_TITLE.to_string(), format_title);
    }

    format_title(file_name)
}

fn sort_nav_items(mut items: Vec<NavItem>) -> Vec<NavItem> {
    items.sort_by(|a, b| {
        let a_is_root = a.path == "/" || a.path.is_empty();
        let b_is_root = b.path == "/" || b.path.is_empty();
        match (a_is_root, b_is_root) {
            (true, false) => Ordering::Less,
            (false, true) => Ordering::Greater,
            _ => a.title.cmp(&b.title),
        }
    });
    items
}

fn is_safe_sidebar_link(link: &str) -> bool {
    let trimmed = link.trim();
    if trimmed.starts_with("//") {
        return false;
    }
    !has_uri_scheme(trimmed) || is_allowed_sidebar_scheme(trimmed)
}

fn is_allowed_sidebar_scheme(link: &str) -> bool {
    let lower = link.to_ascii_lowercase();
    lower.starts_with("http:") || lower.starts_with("https:") || lower.starts_with("mailto:")
}

fn has_uri_scheme(link: &str) -> bool {
    let mut chars = link.chars();
    let Some(first) = chars.next() else {
        return false;
    };
    if !first.is_ascii_alphabetic() {
        return false;
    }

    for ch in chars {
        if ch == ':' {
            return true;
        }
        if !(ch.is_ascii_alphanumeric() || matches!(ch, '+' | '.' | '-')) {
            return false;
        }
    }
    false
}

fn is_external_or_anchor_sidebar_link(link: &str) -> bool {
    link.starts_with('#') || is_allowed_sidebar_scheme(link)
}

fn sidebar_path(link: Option<&str>) -> String {
    let Some(link) = link else {
        return String::new();
    };
    if !is_safe_sidebar_link(link) {
        return String::new();
    }
    let trimmed = link.trim();
    if is_external_or_anchor_sidebar_link(trimmed) {
        return String::new();
    }

    let without_hash = trimmed.split('#').next().unwrap_or_default();
    let without_query = without_hash.split('?').next().unwrap_or_default();
    let bare =
        strip_markdown_extension(without_query.trim_start_matches('/').trim_end_matches('/'));

    if bare.is_empty() || bare == "index" {
        return "/".to_string();
    }
    bare.strip_suffix("/index").unwrap_or(&bare).to_string()
}

fn sidebar_href(link: Option<&str>, base: &str, extension: &str) -> String {
    let Some(link) = link else {
        return "#".to_string();
    };
    let trimmed = link.trim();
    if !is_safe_sidebar_link(trimmed) {
        return "#".to_string();
    }
    if is_external_or_anchor_sidebar_link(trimmed) {
        return trimmed.to_string();
    }

    let hash =
        trimmed.split_once('#').map(|(_, fragment)| format!("#{fragment}")).unwrap_or_default();
    let without_hash =
        trimmed.split('#').next().unwrap_or_default().trim_start_matches('/').trim_end_matches('/');
    let without_ext = strip_markdown_extension(without_hash);
    let route = if without_ext.is_empty() || without_ext == "index" {
        "index".to_string()
    } else {
        format!("{}/index", without_ext.strip_suffix("/index").unwrap_or(&without_ext))
    };

    format!("{base}{route}{extension}{hash}")
}

fn extract_h1_text(content: &str) -> Option<String> {
    let lower = content.to_ascii_lowercase();
    let h1_start = lower.find("<h1")?;
    let tag_end = lower[h1_start..].find('>')? + h1_start;
    let text_start = tag_end + 1;
    let close = lower[text_start..].find("</h1>")? + text_start;
    let text = content[text_start..close].trim();

    if text.is_empty() || text.contains('<') {
        None
    } else {
        Some(text.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn resolves_index_and_nested_routes() {
        let root = "/repo/docs";

        assert_eq!(get_url_path("/repo/docs/index.md", root), "/");
        assert_eq!(get_href("/repo/docs/index.md", root, "/base/", ".html"), "/base/index.html");
        assert_eq!(
            get_output_path("/repo/docs/guide/intro.md", root, "/repo/dist", ".html"),
            join_path("/repo/dist", "guide/intro/index.html")
        );
        assert_eq!(
            get_og_image_url(
                "/repo/docs/guide/index.markdown",
                root,
                "/base/",
                Some("https://example.com/")
            ),
            "https://example.com/base/guide/og-image.png"
        );
    }

    #[test]
    fn builds_default_nav_groups() {
        let root = "/repo/docs";
        let files = vec![
            "/repo/docs/api/reference.md".to_string(),
            "/repo/docs/index.md".to_string(),
            "/repo/docs/examples/basic.md".to_string(),
            "/repo/docs/guide.md".to_string(),
        ];

        let groups = build_nav_items(&files, root, "/docs/", ".html");

        assert_eq!(groups[0].title, "Guide");
        assert_eq!(groups[0].items[0].title, "Overview");
        assert_eq!(groups[1].title, "Examples");
        assert_eq!(groups[2].title, "Api");
    }

    #[test]
    fn builds_theme_sidebar_groups() {
        let groups = build_theme_nav_items(
            &[
                SidebarItem {
                    text: Some("Intro".to_string()),
                    link: Some("/index.md".to_string()),
                    ..SidebarItem::default()
                },
                SidebarItem {
                    text: Some("Guide".to_string()),
                    items: vec![SidebarItem {
                        text: Some("Install".to_string()),
                        link: Some("guide/install.md#cli".to_string()),
                        ..SidebarItem::default()
                    }],
                    collapsed: Some(true),
                    ..SidebarItem::default()
                },
                SidebarItem {
                    text: Some("Unsafe".to_string()),
                    link: Some("javascript:alert(1)".to_string()),
                    ..SidebarItem::default()
                },
            ],
            "/docs/",
            ".html",
        );

        assert_eq!(groups[0].items[0].href, "/docs/index.html");
        assert_eq!(groups[1].title, "Guide");
        assert_eq!(groups[1].items[0].href, "/docs/guide/install/index.html#cli");
        assert_eq!(groups[2].items[0].href, "#");
    }

    #[test]
    fn extracts_titles_like_the_ts_helper() {
        assert_eq!(extract_title("<h1>Rendered Title</h1>", None), "Rendered Title");
        assert_eq!(extract_title("<h1><span>Nested</span></h1>", None), "Untitled");
        assert_eq!(extract_title("<h1>Ignored</h1>", Some("Frontmatter")), "Frontmatter");
        assert_eq!(format_title("getting_started-now"), "Getting Started Now");
    }
}
