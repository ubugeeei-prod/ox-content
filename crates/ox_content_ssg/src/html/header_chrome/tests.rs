use super::super::{
    HeaderNavItem, NavGroup, NavItem, PageChromeFlags, PageData, ReaderChrome, SsgConfig,
    ThemeConfig, TocEntry, generate_html,
};

mod announcement;
mod nav;
mod page_chrome;

fn nav_item(text: &str, link: Option<&str>) -> HeaderNavItem {
    HeaderNavItem { text: text.to_string(), link: link.map(ToOwned::to_owned), items: vec![] }
}

fn dropdown(text: &str, items: Vec<HeaderNavItem>) -> HeaderNavItem {
    HeaderNavItem { text: text.to_string(), link: None, items }
}

fn page() -> PageData {
    PageData {
        title: "Guide".to_string(),
        description: None,
        content: "<p class=\"ox-edit-this-page\"><a href=\"https://example.com/edit\">Edit this page</a></p>"
            .to_string(),
        toc: vec![TocEntry { depth: 1, text: "Hello".to_string(), slug: "hello".to_string() }],
        last_updated: Some(0),
        contributors: vec![],
        path: "guide".to_string(),
        entry_page: None,
        prev: None,
        next: None,
        breadcrumbs: None,
        chrome: PageChromeFlags::default(),
    }
}

fn sidebar() -> Vec<NavGroup> {
    vec![NavGroup {
        title: "Guide".to_string(),
        items: vec![NavItem {
            title: "Guide".to_string(),
            path: "guide".to_string(),
            href: "/docs/guide/index.html".to_string(),
            children: vec![],
            collapsed: None,
            sticky_collapsed: None,
        }],
        collapsed: None,
        sticky_collapsed: None,
    }]
}

fn config(theme: Option<ThemeConfig>, page_chrome: bool) -> SsgConfig {
    SsgConfig {
        site_name: "Docs".to_string(),
        base: "/docs/".to_string(),
        breadcrumb_root_href: None,
        og_image: None,
        theme,
        locale: None,
        available_locales: None,
        pagination: false,
        breadcrumbs: false,
        reader_chrome: ReaderChrome::default(),
        locale_switcher: false,
        locale_paths: vec![],
        a11y: crate::A11y::default(),
        page_chrome,
        json_ld: crate::JsonLd::default(),
    }
}

fn render(theme: Option<ThemeConfig>, page_chrome: bool, chrome: PageChromeFlags) -> String {
    let mut page_data = page();
    page_data.chrome = chrome;
    generate_html(&page_data, &sidebar(), &config(theme, page_chrome))
}

fn theme_nav(items: Vec<HeaderNavItem>) -> ThemeConfig {
    ThemeConfig { nav: Some(items), ..ThemeConfig::default() }
}

fn announce_html(html: &str) -> &str {
    let start = html.find(r#"<div class="ox-announce""#).expect("announcement bar");
    let rest = &html[start..];
    let end = rest.find("</div>").expect("announcement close");
    &rest[..end]
}

fn body_class(html: &str) -> &str {
    let start = html.find("<body").expect("body");
    let tag = &html[start..];
    let end = tag.find('>').expect("body close");
    &tag[..=end]
}
