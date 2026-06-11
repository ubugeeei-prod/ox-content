use serde::{Deserialize, Serialize};

use crate::{NavGroup, NavItem};

/// Manual navigation group supplied by user configuration.
#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq, Eq)]
pub struct ManualNavigationGroup {
    /// Group title.
    pub title: String,
    /// Items in this group.
    #[serde(default)]
    pub items: Vec<ManualNavigationItem>,
}

/// Manual navigation item supplied by user configuration.
#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq, Eq)]
pub struct ManualNavigationItem {
    /// Display title.
    pub title: String,
    /// Route path used for active matching.
    pub path: Option<String>,
    /// Final href, or source for href normalization.
    pub href: Option<String>,
}

/// Resolves manual navigation config to the format used by the SSG renderer.
pub fn resolve_navigation_groups(
    navigation: &[ManualNavigationGroup],
    base: &str,
    extension: &str,
) -> Vec<NavGroup> {
    navigation
        .iter()
        .map(|group| NavGroup {
            title: group.title.clone(),
            items: group
                .items
                .iter()
                .filter_map(|item| resolve_manual_nav_item(item, base, extension))
                .collect(),
            collapsed: None,
            sticky_collapsed: None,
        })
        .collect()
}

fn resolve_manual_nav_item(
    item: &ManualNavigationItem,
    base: &str,
    extension: &str,
) -> Option<NavItem> {
    let raw_href = item.href.as_deref().or(item.path.as_deref())?;

    if is_external_href(raw_href) || raw_href.starts_with('#') {
        return Some(NavItem {
            title: item.title.clone(),
            path: item.path.clone().unwrap_or_else(|| raw_href.to_string()),
            href: raw_href.to_string(),
            children: Vec::new(),
            collapsed: None,
            sticky_collapsed: None,
        });
    }

    let path_source = item.path.as_deref().unwrap_or(raw_href);
    let normalized_path = normalize_navigation_path(path_source);
    let href = if let Some(href) = &item.href {
        let normalized_href = normalize_navigation_path(href);
        format!(
            "{}{}",
            build_href_from_navigation_path(&normalized_href.path, base, extension),
            normalized_href.suffix
        )
    } else {
        build_href_from_navigation_path(&normalized_path.path, base, extension)
    };

    Some(NavItem {
        title: item.title.clone(),
        path: normalized_path.path,
        href,
        children: Vec::new(),
        collapsed: None,
        sticky_collapsed: None,
    })
}

fn is_external_href(value: &str) -> bool {
    if value.starts_with("//") {
        return true;
    }

    let Some((scheme, _)) = value.split_once(':') else {
        return false;
    };
    let mut chars = scheme.chars();
    matches!(chars.next(), Some(first) if first.is_ascii_alphabetic())
        && chars.all(|ch| ch.is_ascii_alphanumeric() || matches!(ch, '+' | '.' | '-'))
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct NormalizedNavigationPath {
    path: String,
    suffix: String,
}

fn split_href_suffix(value: &str) -> (&str, &str) {
    let split_at = value.find(['?', '#']).unwrap_or(value.len());
    value.split_at(split_at)
}

fn normalize_navigation_path(value: &str) -> NormalizedNavigationPath {
    let (pathname, suffix) = split_href_suffix(value.trim());
    let mut normalized = if pathname.is_empty() { "/" } else { pathname }.to_string();

    if !normalized.starts_with('/') {
        normalized.insert(0, '/');
    }

    normalized = strip_navigation_index(&normalized);
    normalized = strip_navigation_extension(&normalized);

    if normalized != "/" {
        normalized = normalized.trim_end_matches('/').to_string();
    }

    NormalizedNavigationPath {
        path: if normalized.is_empty() { "/".to_string() } else { normalized },
        suffix: suffix.to_string(),
    }
}

fn strip_navigation_index(pathname: &str) -> String {
    for suffix in ["/index.html", "/index.htm", "/index.md", "/index.markdown", "/index"] {
        if pathname.to_ascii_lowercase().ends_with(suffix) {
            let keep = pathname.len().saturating_sub(suffix.len());
            let mut stripped = pathname[..keep].to_string();
            stripped.push('/');
            return stripped;
        }
    }
    pathname.to_string()
}

fn strip_navigation_extension(pathname: &str) -> String {
    for extension in [".html", ".htm", ".md", ".markdown"] {
        if pathname.to_ascii_lowercase().ends_with(extension) {
            return pathname[..pathname.len().saturating_sub(extension.len())].to_string();
        }
    }
    pathname.to_string()
}

fn build_href_from_navigation_path(pathname: &str, base: &str, extension: &str) -> String {
    if pathname == "/" || pathname.is_empty() {
        return format!("{base}index{extension}");
    }

    format!("{}{}/index{extension}", base, pathname.trim_start_matches('/'))
}
