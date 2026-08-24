use super::{
    NotFoundOptions, NotFoundOutput, NotFoundRequest, generate_not_found, not_found_sitemap_page,
};
use crate::html::{NavGroup, NavItem};
use crate::site_maps::{SiteMapPage, SiteMapsOptions, generate_site_maps};

fn enabled() -> NotFoundOptions {
    NotFoundOptions { enabled: true, source: "404.md".to_string() }
}

fn request<'a>(markdown: Option<&'a str>, nav_groups: &'a [NavGroup]) -> NotFoundRequest<'a> {
    NotFoundRequest {
        src_dir: "/repo/content",
        out_dir: "/repo/dist",
        extension: ".html",
        base: "/",
        site_name: "Docs",
        markdown,
        rendered_html: None,
        nav_groups,
    }
}

fn nav() -> Vec<NavGroup> {
    vec![NavGroup {
        title: "Guide".to_string(),
        items: vec![NavItem {
            title: "Home".to_string(),
            path: "/".to_string(),
            href: "/index.html".to_string(),
            children: vec![],
            collapsed: None,
            sticky_collapsed: None,
        }],
        collapsed: None,
        sticky_collapsed: None,
    }]
}

fn sitemap_options() -> SiteMapsOptions {
    SiteMapsOptions {
        enabled: true,
        site_url: Some("https://example.com".to_string()),
        sitemap_loc: "https://example.com/sitemap.xml".to_string(),
        site_name: "Docs".to_string(),
        site_description: None,
        robots: true,
        llms: true,
    }
}

#[test]
fn disabled_by_default() {
    let nav = nav();
    let output = generate_not_found(
        &NotFoundOptions::default(),
        &request(Some("---\ntitle: Missing\n---\n\nShould not render.\n"), &nav),
    );

    assert_eq!(output, NotFoundOutput::default());
}

#[test]
fn happy_path_writes_404_from_markdown() {
    let nav = nav();
    let markdown = "\
---
title: Page not found
---

This URL is not a published page. Use search to find another topic.

unique-404-body
";
    let output = generate_not_found(&enabled(), &request(Some(markdown), &nav));
    let html = output.html.expect("enabled 404.md should produce HTML");
    let output_path = output.output_path.expect("enabled 404.md should produce a path");
    let page = output.page.expect("enabled 404.md should produce page metadata");

    assert!(
        output_path.ends_with("404/index.html") || output_path.ends_with("404.html"),
        "404 output must match SSG URL style, got {output_path}"
    );
    assert!(
        output_path.contains("404/index.html"),
        "pretty-dir SSG output should be 404/index.html, got {output_path}"
    );
    assert!(html.contains("unique-404-body"), "{html}");
    assert!(html.contains("Page not found"), "{html}");
    assert!(html.contains("search-button"), "default theme search chrome: {html}");
    assert!(html.contains(r#"<meta name="robots" content="noindex">"#), "{html}");
    assert!(html.contains("Home"), "nav chrome should stay on the 404 page: {html}");
    assert!(page.noindex, "{page:?}");
    assert!(page.draft, "{page:?}");
    assert_eq!(output.warning, None);
}

#[test]
fn missing_source_warns() {
    let nav = nav();
    let output = generate_not_found(&enabled(), &request(None, &nav));

    assert_eq!(output.html, None);
    assert_eq!(output.output_path, None);
    assert_eq!(output.page, None);
    assert_eq!(
        output.warning.as_deref(),
        Some(
            "[ox-content] notFound is enabled but 404.md was not found; the 404 page was not written"
        )
    );
}

#[test]
fn not_included_in_sitemap_pages() {
    let nav = nav();
    let output =
        generate_not_found(&enabled(), &request(Some("---\ntitle: Lost\n---\n\nGone.\n"), &nav));
    let page = output.page.expect("404 page metadata");
    assert!(page.noindex, "{page:?}");
    assert!(page.draft, "{page:?}");

    let maps = generate_site_maps(
        &sitemap_options(),
        &[
            not_found_sitemap_page(&page, "https://example.com/404/"),
            SiteMapPage {
                loc: "https://example.com/guide/".to_string(),
                title: "Guide".to_string(),
                description: None,
                draft: false,
                noindex: false,
                unlisted: false,
            },
        ],
    );
    let sitemap = maps.sitemap_xml.expect("published pages should still emit a sitemap");
    let llms = maps.llms_txt.expect("published pages should still emit llms.txt");

    assert!(sitemap.contains("https://example.com/guide/"), "{sitemap}");
    assert!(!sitemap.contains("/404"), "{sitemap}");
    assert!(llms.contains("Guide"), "{llms}");
    assert!(!llms.contains("Lost"), "{llms}");
}

#[test]
fn hostile_title_escaped() {
    let nav = nav();
    let markdown = "\
---
title: \"</title><script>alert(1)</script>\"
---

Body
";
    let output = generate_not_found(&enabled(), &request(Some(markdown), &nav));
    let html = output.html.expect("hostile title must still render");

    assert!(!html.contains("</title><script>alert(1)</script>"), "{html}");
    assert!(!html.contains("<script>alert(1)</script>"), "{html}");
    assert!(html.contains("&lt;/title&gt;") || html.contains("&#60;/title&#62;"), "{html}");
    assert!(html.contains("&lt;script&gt;") || html.contains("&#60;script&#62;"), "{html}");
}
