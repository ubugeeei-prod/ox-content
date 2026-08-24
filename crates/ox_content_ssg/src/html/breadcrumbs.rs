//! Opt-in breadcrumb trail from the site root through sidebar ancestors.

use super::page::{NavGroup, NavItem, PageData, SsgConfig};

/// One crumb in the visible trail.
pub(super) struct BreadcrumbCrumb {
    pub title: String,
    pub href: Option<String>,
    pub current: bool,
}

/// Resolved breadcrumb trail for the current page.
pub(super) struct BreadcrumbsView {
    pub crumbs: Vec<BreadcrumbCrumb>,
}

/// Builds the visible trail when breadcrumbs are enabled for this page.
pub(super) fn resolve_breadcrumbs(
    page: &PageData,
    nav_groups: &[NavGroup],
    config: &SsgConfig,
) -> Option<BreadcrumbsView> {
    if !breadcrumbs_enabled(page, config) {
        return None;
    }

    let mut crumbs = vec![site_root_crumb(config)];
    if let Some(sidebar) = find_sidebar_trail(nav_groups, &page.path) {
        crumbs.extend(sidebar);
    } else {
        crumbs.push(current_crumb(&page.title));
    }

    Some(BreadcrumbsView { crumbs })
}

fn breadcrumbs_enabled(page: &PageData, config: &SsgConfig) -> bool {
    if page.breadcrumbs == Some(false) {
        return false;
    }
    config.breadcrumbs || config.theme.as_ref().and_then(|theme| theme.breadcrumbs) == Some(true)
}

fn site_root_crumb(config: &SsgConfig) -> BreadcrumbCrumb {
    let href = format!("{}index.html", config.base);
    BreadcrumbCrumb {
        title: config.site_name.clone(),
        href: is_safe_href(&href).then_some(href),
        current: false,
    }
}

fn current_crumb(title: &str) -> BreadcrumbCrumb {
    BreadcrumbCrumb { title: title.to_string(), href: None, current: true }
}

fn find_sidebar_trail(nav_groups: &[NavGroup], page_path: &str) -> Option<Vec<BreadcrumbCrumb>> {
    for group in nav_groups {
        let mut trail = Vec::new();
        if !group.title.trim().is_empty() {
            trail.push(BreadcrumbCrumb { title: group.title.clone(), href: None, current: false });
        }
        if walk_items(&group.items, page_path, &mut trail) {
            return Some(trail);
        }
    }
    None
}

fn walk_items(items: &[NavItem], page_path: &str, trail: &mut Vec<BreadcrumbCrumb>) -> bool {
    for item in items {
        let is_current = page_matches(page_path, item);
        trail.push(crumb_from_item(item, is_current));
        if is_current || walk_items(&item.children, page_path, trail) {
            return true;
        }
        trail.pop();
    }
    false
}

fn crumb_from_item(item: &NavItem, current: bool) -> BreadcrumbCrumb {
    let href = (!current)
        .then_some(item.href.as_str())
        .filter(|href| is_safe_href(href))
        .map(str::to_string);
    BreadcrumbCrumb { title: item.title.clone(), href, current }
}

fn page_matches(page_path: &str, item: &NavItem) -> bool {
    let current = normalize_path(page_path);
    current == normalize_path(&item.path) || current == normalize_path(&item.href)
}

fn normalize_path(value: &str) -> &str {
    value.trim().trim_matches('/')
}

fn is_safe_href(href: &str) -> bool {
    let trimmed = href.trim();
    if trimmed.is_empty() || trimmed.starts_with("//") {
        return false;
    }
    let lower = trimmed.to_ascii_lowercase();
    if lower.starts_with("javascript:")
        || lower.starts_with("data:")
        || lower.starts_with("vbscript:")
    {
        return false;
    }
    if let Some(scheme_end) = trimmed.find(':') {
        let scheme = &lower[..scheme_end];
        return matches!(scheme, "http" | "https" | "mailto");
    }
    true
}

#[cfg(test)]
mod tests;
