//! Sidebar scroll and collapse state must be restored before first paint.

use super::super::nav::SIDEBAR_BOOTSTRAP_JS;
use super::super::*;

fn page() -> PageData {
    PageData {
        title: "Beta".to_string(),
        description: None,
        content: "<p>Content</p>".to_string(),
        toc: vec![],
        last_updated: None,
        contributors: vec![],
        path: "beta".to_string(),
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

fn nav() -> Vec<NavGroup> {
    let item = |title: &str, path: &str| NavItem {
        title: title.to_string(),
        path: path.to_string(),
        href: format!("/docs/{path}.html"),
        children: vec![],
        collapsed: None,
        sticky_collapsed: None,
    };
    vec![NavGroup {
        title: "Guide".to_string(),
        items: vec![item("Alpha", "alpha"), item("Beta", "beta")],
        collapsed: Some(false),
        sticky_collapsed: Some(true),
    }]
}

fn config() -> SsgConfig {
    SsgConfig {
        site_name: "Docs".to_string(),
        base: "/docs/".to_string(),
        breadcrumb_root_href: None,
        og_image: None,
        theme: None,
        locale: None,
        available_locales: None,
        pagination: false,
        breadcrumbs: false,
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
fn restores_inline_between_the_sidebar_and_the_main_column() {
    let html = generate_html(&page(), &nav(), &config());
    let aside_end = html.find("</aside>").expect("sidebar");
    let restore = html.find("sessionStorage, \"sidebarScroll\"").expect("inline restore");
    let main = html.find("<main class=\"main").expect("main column");
    let deferred = html.find("<!-- ox-content:scripts:start -->").expect("page scripts");

    assert!(aside_end < restore && restore < main && main < deferred, "{html}");
    assert!(html.contains("\"ox-content:nav:/docs/:\" + key"), "{html}");
    assert!(!html.contains("ox-content:nav:{{base}}"), "{html}");
}

#[test]
fn production_asset_extraction_keeps_the_restore_inline() {
    let result = crate::externalize_shared_page_assets(
        vec![crate::GeneratedHtmlPage {
            input_path: "beta.md".to_string(),
            output_path: "/site/beta/index.html".to_string(),
            html: generate_html(&page(), &nav(), &config()),
        }],
        "/site",
        "/docs/",
    );

    let html = &result.pages[0].html;
    let aside_end = html.find("</aside>").expect("sidebar");
    let restore = html.find("sessionStorage, \"sidebarScroll\"").expect("inline restore");
    assert!(aside_end < restore && restore < html.find("<main").expect("main"), "{html}");
    assert!(html.contains("<script defer src=\"/docs/assets/ox-content-core-"), "{html}");
}

#[test]
fn deferred_core_script_records_but_never_restores_the_sidebar() {
    // Restoring from the deferred script paints the sidebar at the top first.
    assert!(SSG_JS.contains("sessionStorage.setItem(\"sidebarScroll\""));
    assert!(!SSG_JS.contains("sessionStorage.getItem(\"sidebarScroll\")"));
    assert!(!SSG_JS.contains("sidebar.scrollTop ="));
    assert!(!SSG_JS.contains("details.open ="));
}

#[test]
fn bootstrap_reveals_the_current_page_and_survives_blocked_storage() {
    assert!(SIDEBAR_BOOTSTRAP_JS.contains("sidebar.querySelector(\".nav-link.active\")"));
    assert!(SIDEBAR_BOOTSTRAP_JS.contains("details[data-ox-nav-state-key]"));
    assert!(SIDEBAR_BOOTSTRAP_JS.contains("try"));
    assert!(SIDEBAR_BOOTSTRAP_JS.contains("catch"));
    assert!(!SIDEBAR_BOOTSTRAP_JS.contains("</script"));
}
