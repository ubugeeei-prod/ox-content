use super::super::{
    A11y, EntryPageConfig, HeadValidation, JsonLd, NavGroup, NavItem, PageChromeFlags, PageData,
    ReaderChrome, SSG_CSS, SsgConfig, ThemeConfig, generate_bare_html, generate_html,
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

fn nav_item_with_children(title: &str, path: &str, href: &str, children: Vec<NavItem>) -> NavItem {
    NavItem {
        title: title.to_string(),
        path: path.to_string(),
        href: href.to_string(),
        children,
        collapsed: None,
        sticky_collapsed: None,
    }
}

fn nested_nav() -> Vec<NavGroup> {
    vec![NavGroup {
        title: "Guide".to_string(),
        items: vec![nav_item_with_children(
            "Features",
            "features",
            "/docs/features/index.html",
            vec![nav_item(
                "Breadcrumbs",
                "features/breadcrumbs",
                "/docs/features/breadcrumbs/index.html",
            )],
        )],
        collapsed: None,
        sticky_collapsed: None,
    }]
}

fn page(path: &str) -> PageData {
    PageData {
        title: "Breadcrumbs".to_string(),
        description: None,
        content: "<p>Body</p>".to_string(),
        toc: vec![],
        last_updated: None,
        contributors: vec![],
        path: path.to_string(),
        entry_page: None,
        prev: None,
        next: None,
        breadcrumbs: None,
        chrome: PageChromeFlags::default(),
        robots: None,
        canonical: None,
        markdown_source: None,
    }
}

fn config(breadcrumbs: bool) -> SsgConfig {
    SsgConfig {
        site_name: "Docs".to_string(),
        base: "/docs/".to_string(),
        breadcrumb_root_href: None,
        og_image: None,
        theme: None,
        locale: None,
        available_locales: None,
        pagination: false,
        breadcrumbs,
        reader_chrome: ReaderChrome::default(),
        locale_switcher: false,
        locale_paths: vec![],
        a11y: A11y::default(),
        page_chrome: false,
        json_ld: JsonLd::default(),
        site_url: None,
        head_validation: HeadValidation::Off,
    }
}

#[test]
fn breadcrumb_root_can_stay_inside_a_documentation_version() {
    let mut config = config(true);
    config.breadcrumb_root_href = Some("/docs/2.90/".to_string());
    let html = generate_html(&page("features/breadcrumbs"), &nested_nav(), &config);
    let trail = breadcrumbs_html(&html).expect("breadcrumbs");
    assert!(trail.contains(r#"href="/docs/2.90/""#), "{trail}");
    assert!(!trail.contains(r#"href="/docs/index.html""#), "{trail}");
}

fn breadcrumbs_html(html: &str) -> Option<&str> {
    let start = html.find(r#"<nav class="ox-breadcrumbs""#)?;
    let rest = &html[start..];
    let end = rest.find("</nav>")?;
    Some(&rest[..end + "</nav>".len()])
}

fn assert_no_json_ld(html: &str) {
    assert!(!html.contains("application/ld+json"), "breadcrumbs must not emit JSON-LD: {html}");
    assert!(!html.contains("BreadcrumbList"), "breadcrumbs must not emit BreadcrumbList: {html}");
}

#[test]
fn mobile_separator_spacing_is_compact() {
    assert!(
        SSG_CSS.contains(
            ".ox-breadcrumbs-item:not(:last-child)::after {\n    margin-inline: 0.25rem;"
        ),
        "mobile separators must not inherit the wide desktop spacing"
    );
}

#[test]
fn disabled_by_default() {
    let html = generate_html(&page("features/breadcrumbs"), &nested_nav(), &config(false));

    assert!(
        breadcrumbs_html(&html).is_none(),
        "disabled breadcrumbs must not emit a trail: {html}"
    );
    assert!(
        !html.contains(r#"aria-label="Breadcrumb""#),
        "disabled breadcrumbs must not emit breadcrumb chrome: {html}"
    );
    assert_no_json_ld(&html);
}

#[test]
fn happy_path_nested_sidebar_trail() {
    let html = generate_html(&page("features/breadcrumbs"), &nested_nav(), &config(true));
    let trail = breadcrumbs_html(&html).expect("enabled nested page should emit a trail");

    assert!(trail.contains(r#"aria-label="Breadcrumb""#), "{trail}");
    assert!(trail.contains(r#"href="/docs/index.html""#), "{trail}");
    assert!(trail.contains("Docs"), "{trail}");
    assert!(trail.contains("Guide"), "{trail}");
    assert!(trail.contains(r#"href="/docs/features/index.html""#), "{trail}");
    assert!(trail.contains("Features"), "{trail}");
    assert!(trail.contains("Breadcrumbs"), "{trail}");
    assert!(trail.contains(r#"aria-current="page""#), "{trail}");
    assert!(
        !trail.contains(r#"href="/docs/features/breadcrumbs/index.html""#),
        "current page must not be a link: {trail}"
    );
    let root = trail.find(r#"href="/docs/index.html""#).expect("root crumb");
    let parent = trail.find(r#"href="/docs/features/index.html""#).expect("parent crumb");
    let current = trail.find(r#"aria-current="page""#).expect("current crumb");
    assert!(root < parent && parent < current, "trail order is wrong: {trail}");
    assert_no_json_ld(&html);
}

#[test]
fn frontmatter_hides() {
    let mut page_data = page("features/breadcrumbs");
    page_data.breadcrumbs = Some(false);

    let html = generate_html(&page_data, &nested_nav(), &config(true));

    assert!(
        breadcrumbs_html(&html).is_none(),
        "frontmatter breadcrumbs: false must hide the trail: {html}"
    );
    assert!(!html.contains(r#"aria-label="Breadcrumb""#), "{html}");
}

#[test]
fn hostile_title_escaped() {
    let nav = vec![NavGroup {
        title: "Guide".to_string(),
        items: vec![nav_item_with_children(
            "<script>alert(1)</script>",
            "features",
            "/docs/features/index.html",
            vec![nav_item(
                "Breadcrumbs",
                "features/breadcrumbs",
                "/docs/features/breadcrumbs/index.html",
            )],
        )],
        collapsed: None,
        sticky_collapsed: None,
    }];

    let html = generate_html(&page("features/breadcrumbs"), &nav, &config(true));
    let trail = breadcrumbs_html(&html).expect("escaped title should still render a trail");

    assert!(
        trail.contains("&#60;script&#62;alert(1)&#60;/script&#62;")
            || trail.contains("&lt;script&gt;alert(1)&lt;/script&gt;"),
        "{trail}"
    );
    assert!(!trail.contains("<script>alert(1)</script>"), "{trail}");
    assert!(trail.contains(r#"href="/docs/features/index.html""#), "{trail}");
}

#[test]
fn javascript_href_rejected() {
    let nav = vec![NavGroup {
        title: "Guide".to_string(),
        items: vec![nav_item_with_children(
            "Evil",
            "features",
            "javascript:alert(1)",
            vec![nav_item(
                "Breadcrumbs",
                "features/breadcrumbs",
                "/docs/features/breadcrumbs/index.html",
            )],
        )],
        collapsed: None,
        sticky_collapsed: None,
    }];

    let html = generate_html(&page("features/breadcrumbs"), &nav, &config(true));
    let trail = breadcrumbs_html(&html).expect("safe crumbs should still render");

    assert!(!html.contains("javascript:"), "{html}");
    assert!(!trail.contains(r#"href="javascript:alert(1)""#), "{trail}");
    assert!(trail.contains("Evil"), "{trail}");
    assert!(trail.contains(r#"aria-current="page""#), "{trail}");
}

#[test]
fn data_and_protocol_relative_hrefs_are_rejected() {
    let nav = vec![NavGroup {
        title: "Guide".to_string(),
        items: vec![
            nav_item("Data", "data", "data:text/html,hi"),
            nav_item_with_children(
                "Proto",
                "features",
                "//evil.example/docs",
                vec![nav_item(
                    "Breadcrumbs",
                    "features/breadcrumbs",
                    "/docs/features/breadcrumbs/index.html",
                )],
            ),
            nav_item("Vbscript", "vb", "vbscript:msgbox(1)"),
        ],
        collapsed: None,
        sticky_collapsed: None,
    }];

    let html = generate_html(&page("features/breadcrumbs"), &nav, &config(true));
    let trail = breadcrumbs_html(&html).expect("safe crumbs should still render");

    assert!(!trail.contains("data:"), "{trail}");
    assert!(!trail.contains("vbscript:"), "{trail}");
    assert!(!trail.contains("//evil.example/docs"), "{trail}");
    assert!(!trail.contains(r#"href="data:"#), "{trail}");
    assert!(trail.contains("Proto"), "{trail}");
}

#[test]
fn theme_breadcrumbs_enables_without_ssg_flag() {
    let mut cfg = config(false);
    cfg.theme = Some(ThemeConfig { breadcrumbs: Some(true), ..ThemeConfig::default() });

    let html = generate_html(&page("features/breadcrumbs"), &nested_nav(), &cfg);
    let trail = breadcrumbs_html(&html).expect("theme.breadcrumbs should enable the trail");

    assert!(trail.contains(r#"aria-label="Breadcrumb""#), "{trail}");
    assert!(trail.contains(r#"href="/docs/features/index.html""#), "{trail}");
}

#[test]
fn generate_bare_html_is_unchanged() {
    let html = generate_bare_html("<h1>Hello</h1>", "Test Page");

    assert_eq!(
        html,
        "<!DOCTYPE html>\n<html lang=\"en\">\n<head>\n  <meta charset=\"UTF-8\">\n  <meta name=\"viewport\" content=\"width=device-width, initial-scale=1.0\">\n  <title>Test Page</title>\n</head>\n<body>\n<h1>Hello</h1>\n</body>\n</html>"
    );
    assert!(breadcrumbs_html(&html).is_none());
}

#[test]
fn entry_page_skips_trail_when_enabled() {
    let mut page_data = page("features/breadcrumbs");
    page_data.entry_page = Some(EntryPageConfig::default());
    page_data.title = "Docs".to_string();

    let html = generate_html(&page_data, &nested_nav(), &config(true));
    assert!(breadcrumbs_html(&html).is_none(), "entry pages must not emit a trail: {html}");
}
