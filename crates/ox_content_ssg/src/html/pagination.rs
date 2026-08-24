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
mod tests {
    use super::super::{
        NavGroup, NavItem, PageData, PagerOverride, SsgConfig, generate_bare_html, generate_html,
    };

    fn nav_item(title: &str, path: &str, href: &str) -> NavItem {
        NavItem {
            title: title.to_string(),
            path: path.to_string(),
            href: href.to_string(),
            children: vec![],
            collapsed: None,
            sticky_collapsed: None,
        }
    }

    fn nav_item_with_children(
        title: &str,
        path: &str,
        href: &str,
        children: Vec<NavItem>,
    ) -> NavItem {
        NavItem {
            title: title.to_string(),
            path: path.to_string(),
            href: href.to_string(),
            children,
            collapsed: None,
            sticky_collapsed: None,
        }
    }

    fn guide(items: Vec<NavItem>) -> Vec<NavGroup> {
        vec![NavGroup {
            title: "Guide".to_string(),
            items,
            collapsed: None,
            sticky_collapsed: None,
        }]
    }

    fn linear_nav() -> Vec<NavGroup> {
        guide(vec![
            nav_item("Intro", "intro", "/docs/intro/index.html"),
            nav_item("Guide", "guide", "/docs/guide/index.html"),
            nav_item("API", "api", "/docs/api/index.html"),
        ])
    }

    fn page(path: &str) -> PageData {
        PageData {
            title: "Current".to_string(),
            description: None,
            content: "<p>Body</p>".to_string(),
            toc: vec![],
            last_updated: None,
            path: path.to_string(),
            entry_page: None,
            prev: None,
            next: None,
        }
    }

    fn config(pagination: bool) -> SsgConfig {
        SsgConfig {
            site_name: "Docs".to_string(),
            base: "/docs/".to_string(),
            og_image: None,
            theme: None,
            locale: None,
            available_locales: None,
            pagination,
        }
    }

    fn pager_html(html: &str) -> Option<&str> {
        let start = html.find(r#"<nav class="pager""#)?;
        let rest = &html[start..];
        let end = rest.find("</nav>")?;
        Some(&rest[..end + "</nav>".len()])
    }

    #[test]
    fn pagination_is_disabled_by_default() {
        let html = generate_html(&page("guide"), &linear_nav(), &config(false));

        assert!(
            pager_html(&html).is_none(),
            "disabled pagination must not emit pager chrome: {html}"
        );
        assert!(
            !html.contains(r#"rel="prev""#),
            "disabled pagination must not emit rel=prev: {html}"
        );
        assert!(
            !html.contains("Previous"),
            "disabled pagination must not emit a Previous label: {html}"
        );
    }

    #[test]
    fn enabled_middle_page_emits_prev_and_next() {
        let html = generate_html(&page("guide"), &linear_nav(), &config(true));
        let pager = pager_html(&html).expect("middle page should emit a pager");

        assert!(pager.contains(r#"rel="prev""#), "{pager}");
        assert!(pager.contains(r#"rel="next""#), "{pager}");
        assert!(pager.contains(r#"href="/docs/intro/index.html""#), "{pager}");
        assert!(pager.contains(r#"href="/docs/api/index.html""#), "{pager}");
        assert!(pager.contains("Intro"), "{pager}");
        assert!(pager.contains("API"), "{pager}");
        assert!(pager.contains("Previous"), "{pager}");
        assert!(pager.contains("Next"), "{pager}");
    }

    #[test]
    fn first_page_emits_next_only() {
        let html = generate_html(&page("intro"), &linear_nav(), &config(true));
        let pager = pager_html(&html).expect("first page should emit a pager");

        assert!(!pager.contains(r#"rel="prev""#), "{pager}");
        assert!(!pager.contains("Previous"), "{pager}");
        assert!(pager.contains(r#"rel="next""#), "{pager}");
        assert!(pager.contains(r#"href="/docs/guide/index.html""#), "{pager}");
        assert!(pager.contains("Guide"), "{pager}");
    }

    #[test]
    fn last_page_emits_prev_only() {
        let html = generate_html(&page("api"), &linear_nav(), &config(true));
        let pager = pager_html(&html).expect("last page should emit a pager");

        assert!(pager.contains(r#"rel="prev""#), "{pager}");
        assert!(pager.contains(r#"href="/docs/guide/index.html""#), "{pager}");
        assert!(pager.contains("Guide"), "{pager}");
        assert!(!pager.contains(r#"rel="next""#), "{pager}");
        assert!(!pager.contains("Next"), "{pager}");
    }

    #[test]
    fn nested_sidebar_children_flatten_depth_first() {
        let nav = guide(vec![
            nav_item_with_children(
                "Parent",
                "parent",
                "/docs/parent/index.html",
                vec![
                    nav_item("Child A", "parent/a", "/docs/parent/a/index.html"),
                    nav_item("Child B", "parent/b", "/docs/parent/b/index.html"),
                ],
            ),
            nav_item("Sibling", "sibling", "/docs/sibling/index.html"),
        ]);

        let html = generate_html(&page("parent/a"), &nav, &config(true));
        let pager = pager_html(&html).expect("nested child should emit a pager");

        assert!(pager.contains(r#"href="/docs/parent/index.html""#), "{pager}");
        assert!(pager.contains("Parent"), "{pager}");
        assert!(pager.contains(r#"href="/docs/parent/b/index.html""#), "{pager}");
        assert!(pager.contains("Child B"), "{pager}");
        assert!(!pager.contains(r#"href="/docs/sibling/index.html""#), "{pager}");
    }

    #[test]
    fn flatten_skips_items_with_empty_href() {
        let nav = guide(vec![
            nav_item("Skipped", "skipped", ""),
            nav_item("First", "first", "/docs/first/index.html"),
            nav_item("Second", "second", "/docs/second/index.html"),
        ]);

        let html = generate_html(&page("first"), &nav, &config(true));
        let pager = pager_html(&html).expect("real neighbors should still emit a pager");

        assert!(!pager.contains(r#"rel="prev""#), "{pager}");
        assert!(!pager.contains("Skipped"), "{pager}");
        assert!(pager.contains(r#"href="/docs/second/index.html""#), "{pager}");
        assert!(pager.contains("Second"), "{pager}");
    }

    #[test]
    fn flatten_skips_external_https_hrefs() {
        let nav = guide(vec![
            nav_item("Intro", "intro", "/docs/intro/index.html"),
            nav_item("External", "external", "https://example.com/docs"),
            nav_item("API", "api", "/docs/api/index.html"),
        ]);

        let html = generate_html(&page("intro"), &nav, &config(true));
        let pager = pager_html(&html).expect("in-site neighbor should still emit a pager");

        assert!(pager.contains(r#"href="/docs/api/index.html""#), "{pager}");
        assert!(pager.contains("API"), "{pager}");
        assert!(!pager.contains("https://example.com/docs"), "{pager}");
        assert!(!pager.contains("External"), "{pager}");
    }

    #[test]
    fn frontmatter_prev_false_hides_prev() {
        let mut page_data = page("guide");
        page_data.prev = Some(PagerOverride { hidden: true, ..PagerOverride::default() });

        let html = generate_html(&page_data, &linear_nav(), &config(true));
        let pager = pager_html(&html).expect("next should still render");

        assert!(!pager.contains(r#"rel="prev""#), "{pager}");
        assert!(!pager.contains("Previous"), "{pager}");
        assert!(!pager.contains("Intro"), "{pager}");
        assert!(pager.contains(r#"rel="next""#), "{pager}");
        assert!(pager.contains(r#"href="/docs/api/index.html""#), "{pager}");
    }

    #[test]
    fn custom_prev_next_title_and_href_are_used() {
        let mut page_data = page("guide");
        page_data.prev = Some(PagerOverride {
            text: Some("Back to Intro".to_string()),
            href: Some("/custom/intro/".to_string()),
            ..PagerOverride::default()
        });
        page_data.next = Some(PagerOverride {
            text: Some("See API".to_string()),
            href: Some("/custom/api/".to_string()),
            ..PagerOverride::default()
        });

        let html = generate_html(&page_data, &linear_nav(), &config(true));
        let pager = pager_html(&html).expect("custom overrides should emit a pager");

        assert!(pager.contains("Back to Intro"), "{pager}");
        assert!(pager.contains(r#"href="/custom/intro/""#), "{pager}");
        assert!(pager.contains("See API"), "{pager}");
        assert!(pager.contains(r#"href="/custom/api/""#), "{pager}");
        assert!(!pager.contains(r#"href="/docs/intro/index.html""#), "{pager}");
        assert!(!pager.contains(r#"href="/docs/api/index.html""#), "{pager}");
    }

    #[test]
    fn hostile_title_is_escaped() {
        let mut page_data = page("guide");
        page_data.prev = Some(PagerOverride {
            text: Some("<script>alert(1)</script>".to_string()),
            href: Some("/safe/".to_string()),
            ..PagerOverride::default()
        });

        let html = generate_html(&page_data, &linear_nav(), &config(true));
        let pager = pager_html(&html).expect("escaped title should still render a pager");

        assert!(
            pager.contains("&#60;script&#62;alert(1)&#60;/script&#62;")
                || pager.contains("&lt;script&gt;alert(1)&lt;/script&gt;"),
            "{pager}"
        );
        assert!(!pager.contains("<script>alert(1)</script>"), "{pager}");
        assert!(pager.contains(r#"href="/safe/""#), "{pager}");
    }

    #[test]
    fn javascript_href_override_is_dropped() {
        let mut page_data = page("guide");
        page_data.prev = Some(PagerOverride {
            text: Some("Evil".to_string()),
            href: Some("javascript:alert(1)".to_string()),
            ..PagerOverride::default()
        });

        let html = generate_html(&page_data, &linear_nav(), &config(true));
        let pager = pager_html(&html).expect("next should still render after a dropped prev");

        assert!(!html.contains("javascript:"), "{html}");
        assert!(!pager.contains(r#"rel="prev""#), "{pager}");
        assert!(!pager.contains("Evil"), "{pager}");
        assert!(pager.contains(r#"rel="next""#), "{pager}");
        assert!(pager.contains(r#"href="/docs/api/index.html""#), "{pager}");
    }

    #[test]
    fn pagination_is_independent_of_empty_toc() {
        let mut page_data = page("guide");
        page_data.toc = Vec::new();

        let html = generate_html(&page_data, &linear_nav(), &config(true));
        let pager = pager_html(&html).expect("empty TOC must not suppress the pager");

        assert!(pager.contains(r#"rel="prev""#), "{pager}");
        assert!(pager.contains(r#"rel="next""#), "{pager}");
        assert!(
            !html.contains(r#"<aside class="toc""#),
            "empty TOC must not emit an outline aside: {html}"
        );
        assert!(
            !html.contains(r#"class="main main--with-toc""#),
            "empty TOC must not add the TOC layout class: {html}"
        );
    }

    #[test]
    fn entry_page_skips_pager_even_when_enabled() {
        let mut page_data = page("guide");
        page_data.entry_page = Some(super::super::EntryPageConfig::default());

        let html = generate_html(&page_data, &linear_nav(), &config(true));

        assert!(pager_html(&html).is_none(), "entry pages must not emit pager chrome: {html}");
        assert!(!html.contains(r#"rel="prev""#), "{html}");
        assert!(!html.contains("Previous"), "{html}");
    }

    #[test]
    fn single_page_sidebar_emits_nothing() {
        let nav = guide(vec![nav_item("Only", "only", "/docs/only/index.html")]);
        let html = generate_html(&page("only"), &nav, &config(true));

        assert!(pager_html(&html).is_none(), "a single-page sidebar must emit no pager: {html}");
        assert!(!html.contains(r#"rel="prev""#), "{html}");
        assert!(!html.contains(r#"rel="next""#), "{html}");
    }

    #[test]
    fn generate_bare_html_is_unchanged() {
        let html = generate_bare_html("<h1>Hello</h1>", "Test Page");

        assert_eq!(
            html,
            "<!DOCTYPE html>\n<html lang=\"en\">\n<head>\n  <meta charset=\"UTF-8\">\n  <meta name=\"viewport\" content=\"width=device-width, initial-scale=1.0\">\n  <title>Test Page</title>\n</head>\n<body>\n<h1>Hello</h1>\n</body>\n</html>"
        );
        assert!(!html.contains("pager"));
        assert!(!html.contains("Previous"));
    }
}
