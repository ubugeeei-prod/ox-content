//! Previous/next page links for SSG pages.

use super::page::{NavGroup, NavItem, PageData, PagerOverride};

/// One resolved pager link rendered after the article.
pub(super) struct PagerLinkView {
    pub title: String,
    pub href: String,
}

/// Previous/next links for the current page, when either side is visible.
pub(super) struct PagerView {
    pub prev: Option<PagerLinkView>,
    pub next: Option<PagerLinkView>,
}

struct NavPage {
    title: String,
    path: String,
    href: String,
}

/// Resolves previous/next links from sidebar order and frontmatter overrides.
pub(super) fn resolve_pager(
    page: &PageData,
    nav_groups: &[NavGroup],
    enabled: bool,
) -> Option<PagerView> {
    if !enabled || page.entry_page.is_some() {
        return None;
    }

    let pages = flatten_nav_pages(nav_groups);
    let index = pages.iter().position(|item| page_matches(&page.path, item));
    let auto_prev = index.and_then(|i| i.checked_sub(1)).and_then(|i| pages.get(i));
    let auto_next = index.and_then(|i| pages.get(i + 1));

    let prev = resolve_side(page.prev.as_ref(), auto_prev);
    let next = resolve_side(page.next.as_ref(), auto_next);

    if prev.is_none() && next.is_none() {
        return None;
    }

    Some(PagerView { prev, next })
}

fn flatten_nav_pages(nav_groups: &[NavGroup]) -> Vec<NavPage> {
    let mut pages = Vec::new();
    for group in nav_groups {
        flatten_nav_items(&group.items, &mut pages);
    }
    pages
}

fn flatten_nav_items(items: &[NavItem], pages: &mut Vec<NavPage>) {
    for item in items {
        if is_in_site_href(&item.href) {
            pages.push(NavPage {
                title: item.title.clone(),
                path: item.path.clone(),
                href: item.href.clone(),
            });
        }
        flatten_nav_items(&item.children, pages);
    }
}

fn page_matches(page_path: &str, item: &NavPage) -> bool {
    let current = normalize_path(page_path);
    current == normalize_path(&item.path) || current == normalize_path(&item.href)
}

fn normalize_path(value: &str) -> &str {
    value.trim().trim_matches('/')
}

fn is_in_site_href(href: &str) -> bool {
    let trimmed = href.trim();
    if trimmed.is_empty() {
        return false;
    }
    let lower = trimmed.to_ascii_lowercase();
    if lower.starts_with("http://") || lower.starts_with("https://") {
        return false;
    }
    is_safe_href(trimmed)
}

fn resolve_side(
    override_link: Option<&PagerOverride>,
    auto: Option<&NavPage>,
) -> Option<PagerLinkView> {
    match override_link {
        Some(over) if over.hidden => None,
        Some(over) => match over.href.as_deref().map(str::trim) {
            Some(href) if !href.is_empty() => {
                if is_safe_href(href) {
                    Some(PagerLinkView {
                        title: over.text.as_deref().filter(|text| !text.is_empty()).map_or_else(
                            || auto.map(|page| page.title.clone()).unwrap_or_default(),
                            ToOwned::to_owned,
                        ),
                        href: href.to_string(),
                    })
                } else {
                    None
                }
            }
            _ => auto.map(nav_page_to_link),
        },
        None => auto.map(nav_page_to_link),
    }
}

fn nav_page_to_link(page: &NavPage) -> PagerLinkView {
    PagerLinkView { title: page.title.clone(), href: page.href.clone() }
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
