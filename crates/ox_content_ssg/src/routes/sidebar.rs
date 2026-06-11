use serde::{Deserialize, Serialize};

use crate::{NavGroup, NavItem};

use super::path::strip_markdown_extension;
use super::{DEFAULT_ROOT_GROUP_TITLE, DEFAULT_UNTITLED_TITLE};

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
    /// Whether this group's open state persists across page navigations.
    pub sticky_collapsed: Option<bool>,
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
            sticky_collapsed: item.sticky_collapsed,
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
                sticky_collapsed: item.sticky_collapsed,
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
            sticky_collapsed: None,
        });
    }
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

#[cfg(test)]
mod tests {
    use super::*;

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
}
