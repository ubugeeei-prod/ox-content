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

/// Resolves previous/next links from sidebar order and frontmatter overrides.
pub(super) fn resolve_pager(
    page: &PageData,
    nav_groups: &[NavGroup],
    enabled: bool,
) -> Option<PagerView> {
    if !enabled || page.entry_page.is_some() {
        return None;
    }

    let neighbours = find_neighbours(nav_groups, &page.path);
    let prev = resolve_side(page.prev.as_ref(), neighbours.prev);
    let next = resolve_side(page.next.as_ref(), neighbours.next);

    if prev.is_none() && next.is_none() {
        return None;
    }

    Some(PagerView { prev, next })
}

/// The sidebar entries on either side of the current page.
#[derive(Default)]
struct Neighbours<'a> {
    prev: Option<&'a NavItem>,
    next: Option<&'a NavItem>,
    /// Set once the current page's own entry has been passed.
    matched: bool,
    /// Set once `next` is known, so the walk stops early.
    done: bool,
}

/// Walks the sidebar in the order it renders and returns the steppable
/// entries either side of `page_path`.
///
/// This used to flatten the whole sidebar into an owned `Vec` first, which
/// cloned three `String`s per entry — on every page. A site pays that for
/// each page it builds, so it was quadratic in sidebar size with an
/// allocator-bound constant. Nothing here needs ownership, and the walk
/// stops one entry past the match.
fn find_neighbours<'a>(nav_groups: &'a [NavGroup], page_path: &str) -> Neighbours<'a> {
    let mut state = Neighbours::default();
    for group in nav_groups {
        visit_nav_items(&group.items, page_path, &mut state);
        if state.done {
            break;
        }
    }
    if !state.matched {
        // The page is not in the sidebar, so neither side is meaningful.
        return Neighbours::default();
    }
    state
}

fn visit_nav_items<'a>(items: &'a [NavItem], page_path: &str, state: &mut Neighbours<'a>) {
    for item in items {
        if is_in_site_href(&item.href) {
            if state.matched {
                state.next = Some(item);
                state.done = true;
                return;
            }
            if page_matches(page_path, item) {
                state.matched = true;
            } else {
                state.prev = Some(item);
            }
        }
        visit_nav_items(&item.children, page_path, state);
        if state.done {
            return;
        }
    }
}

fn page_matches(page_path: &str, item: &NavItem) -> bool {
    let current = normalize_path(page_path);
    current == normalize_path(&item.path) || current == normalize_path(&item.href)
}

fn normalize_path(value: &str) -> &str {
    value.trim().trim_matches('/')
}

/// True when an entry points at a page of this site the pager can step to.
///
/// A sidebar group with no `link` of its own is still a nav item, and the
/// theme gives it `href="#"` so the header can toggle the group. It is a
/// heading, not a destination: stepping onto it left "Next" pointing at
/// `#`, which goes nowhere. Its children are flattened either way.
fn is_in_site_href(href: &str) -> bool {
    let trimmed = href.trim();
    if trimmed.is_empty() || trimmed.starts_with('#') {
        return false;
    }
    if starts_with_ignore_case(trimmed, "http://") || starts_with_ignore_case(trimmed, "https://") {
        return false;
    }
    is_safe_href(trimmed)
}

/// Case-insensitive prefix test that does not build a lowercased copy.
///
/// This runs once per sidebar entry per page, so the copy it replaces was
/// an allocation per entry per page across the whole site.
fn starts_with_ignore_case(value: &str, prefix: &str) -> bool {
    value.len() >= prefix.len()
        && value.as_bytes()[..prefix.len()].eq_ignore_ascii_case(prefix.as_bytes())
}

fn resolve_side(
    override_link: Option<&PagerOverride>,
    auto: Option<&NavItem>,
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

fn nav_page_to_link(item: &NavItem) -> PagerLinkView {
    PagerLinkView { title: item.title.clone(), href: item.href.clone() }
}

fn is_safe_href(href: &str) -> bool {
    let trimmed = href.trim();
    if trimmed.is_empty() || trimmed.starts_with("//") {
        return false;
    }
    if starts_with_ignore_case(trimmed, "javascript:")
        || starts_with_ignore_case(trimmed, "data:")
        || starts_with_ignore_case(trimmed, "vbscript:")
    {
        return false;
    }
    if let Some(scheme_end) = trimmed.find(':') {
        let scheme = &trimmed[..scheme_end];
        return scheme.eq_ignore_ascii_case("http")
            || scheme.eq_ignore_ascii_case("https")
            || scheme.eq_ignore_ascii_case("mailto");
    }
    true
}

#[cfg(test)]
mod tests;
