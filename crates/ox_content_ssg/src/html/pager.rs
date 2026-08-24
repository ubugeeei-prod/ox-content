//! Opt-in previous/next page links derived from sidebar order.

use std::fmt::Write as _;

use super::utils::escape_html;
use super::{NavGroup, NavItem};

pub(super) const PAGER_CSS: &str = include_str!("pager.css");

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct PagerLink {
    pub href: String,
    pub title: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct Pager {
    pub prev: Option<PagerLink>,
    pub next: Option<PagerLink>,
}

pub(super) fn resolve_pager(nav_groups: &[NavGroup], current_path: &str) -> Option<Pager> {
    let pages = flatten_pager_pages(nav_groups);
    let index = pages.iter().position(|page| page.path == current_path)?;
    let prev = index.checked_sub(1).map(|prev_index| pager_link(pages[prev_index]));
    let next = pages.get(index + 1).map(|page| pager_link(page));
    (prev.is_some() || next.is_some()).then_some(Pager { prev, next })
}

pub(super) fn generate_pager_html(nav_groups: &[NavGroup], current_path: &str) -> String {
    let Some(pager) = resolve_pager(nav_groups, current_path) else {
        return String::new();
    };

    let mut html = String::from("<nav class=\"pager\" aria-label=\"Page navigation\">\n");
    match pager.prev {
        Some(prev) => push_link(&mut html, "prev", "Previous", &prev),
        None => html.push_str("  <div class=\"pager-slot\"></div>\n"),
    }
    match pager.next {
        Some(next) => push_link(&mut html, "next", "Next", &next),
        None => html.push_str("  <div class=\"pager-slot\"></div>\n"),
    }
    html.push_str("</nav>\n");
    html
}

fn flatten_pager_pages(nav_groups: &[NavGroup]) -> Vec<&NavItem> {
    let mut pages = Vec::new();
    for group in nav_groups {
        collect_pager_pages(&group.items, &mut pages);
    }
    pages
}

fn collect_pager_pages<'a>(items: &'a [NavItem], pages: &mut Vec<&'a NavItem>) {
    for item in items {
        if !item.path.is_empty() && is_pager_href(&item.href) {
            pages.push(item);
        }
        collect_pager_pages(&item.children, pages);
    }
}

fn pager_link(item: &NavItem) -> PagerLink {
    PagerLink { href: item.href.clone(), title: item.title.clone() }
}

fn is_pager_href(href: &str) -> bool {
    let trimmed = href.trim();
    if trimmed.is_empty() || trimmed == "#" || trimmed.starts_with("//") {
        return false;
    }
    let lower = trimmed.to_ascii_lowercase();
    let safe_scheme = lower.starts_with("http://")
        || lower.starts_with("https://")
        || lower.starts_with("mailto:");
    !trimmed.contains(':') || safe_scheme
}

fn push_link(html: &mut String, kind: &str, label: &str, link: &PagerLink) {
    let href = escape_html(link.href.trim());
    let title = escape_html(&link.title);
    if html.write_fmt(format_args!(
        "  <a class=\"pager-link pager-link--{kind}\" href=\"{href}\">\n    <span class=\"pager-label\">{label}</span>\n    <span class=\"pager-title\">{title}</span>\n  </a>\n"
    ))
    .is_err()
    {
        html.push_str("[formatting failed]");
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::html::{NavGroup, NavItem};

    fn item(title: &str, path: &str, href: &str, children: Vec<NavItem>) -> NavItem {
        NavItem {
            title: title.to_string(),
            path: path.to_string(),
            href: href.to_string(),
            children,
            collapsed: None,
            sticky_collapsed: None,
        }
    }

    fn group(items: Vec<NavItem>) -> NavGroup {
        NavGroup { title: "Guide".to_string(), items, collapsed: None, sticky_collapsed: None }
    }

    #[test]
    fn neighbors_follow_sidebar_order_including_nested_items() {
        let groups = vec![
            group(vec![item(
                "Intro",
                "intro",
                "/intro.html",
                vec![item("Setup", "setup", "/setup.html", vec![])],
            )]),
            group(vec![item("API", "api", "/api.html", vec![])]),
        ];

        assert_eq!(
            resolve_pager(&groups, "setup"),
            Some(Pager {
                prev: Some(PagerLink { href: "/intro.html".into(), title: "Intro".into() }),
                next: Some(PagerLink { href: "/api.html".into(), title: "API".into() }),
            })
        );
    }

    #[test]
    fn first_page_has_only_next_and_last_page_has_only_prev() {
        let groups = vec![group(vec![
            item("One", "one", "/one.html", vec![]),
            item("Two", "two", "/two.html", vec![]),
        ])];

        let first = resolve_pager(&groups, "one").expect("first page");
        assert!(first.prev.is_none());
        assert_eq!(first.next.as_ref().map(|link| link.title.as_str()), Some("Two"));

        let last = resolve_pager(&groups, "two").expect("last page");
        assert_eq!(last.prev.as_ref().map(|link| link.title.as_str()), Some("One"));
        assert!(last.next.is_none());
    }

    #[test]
    fn unknown_or_unsafe_entries_are_not_pager_targets() {
        let groups = vec![group(vec![
            item("Safe", "safe", "/safe.html", vec![]),
            item("Script", "script", "javascript:alert(1)", vec![]),
            item("Blank", "", "/blank.html", vec![]),
        ])];

        assert!(resolve_pager(&groups, "missing").is_none());
        assert!(resolve_pager(&groups, "script").is_none());
        assert_eq!(
            resolve_pager(&groups, "safe"),
            None,
            "a lone safe page has no neighbor after unsafe/blank entries are dropped"
        );
    }

    #[test]
    fn generated_markup_escapes_titles_and_hrefs() {
        let groups = vec![group(vec![
            item("A <script>", "a", "/a.html", vec![]),
            item("B & C", "b", "/b\".html", vec![]),
        ])];
        let html = generate_pager_html(&groups, "a");
        assert!(html.contains("class=\"pager\""), "{html}");
        assert!(html.contains("pager-link--next"), "{html}");
        assert!(html.contains("B &amp; C"), "{html}");
        assert!(html.contains("href=\"/b&quot;.html\""), "{html}");
        assert!(!html.contains("<script>"), "{html}");
        assert!(!html.contains("pager-link--prev"), "{html}");
    }
}
