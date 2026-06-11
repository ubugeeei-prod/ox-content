use std::cmp::Ordering;
// BTreeMap keeps nav groups sorted deterministically before rendering.
use std::collections::BTreeMap;

use crate::{NavGroup, NavItem};

use super::files::{format_title, get_display_title};
use super::path::{get_href, get_url_path, normalize_separators, relative_path};
use super::{DEFAULT_NAV_GROUP_ORDER, DEFAULT_ROOT_GROUP_TITLE, DEFAULT_ROOT_NAV_TITLE};

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
            sticky_collapsed: None,
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
                    sticky_collapsed: None,
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
                sticky_collapsed: None,
            });
        }
    }

    result
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

#[cfg(test)]
mod tests {
    use super::*;

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
}
