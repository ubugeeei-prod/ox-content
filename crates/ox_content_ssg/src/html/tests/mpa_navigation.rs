use super::super::*;

fn page() -> PageData {
    PageData {
        title: "Stable navigation".to_string(),
        description: None,
        content: "<p>Content</p>".to_string(),
        toc: vec![],
        last_updated: None,
        contributors: vec![],
        path: "stable".to_string(),
        entry_page: None,
        prev: None,
        next: None,
        breadcrumbs: None,
        chrome: PageChromeFlags::default(),
        robots: None,
        canonical: None,
    }
}

fn config(theme: Option<ThemeConfig>) -> SsgConfig {
    SsgConfig {
        site_name: "Test Site".to_string(),
        base: "/".to_string(),
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
        a11y: A11y::default(),
        page_chrome: false,
        json_ld: JsonLd::default(),
        site_url: None,
        head_validation: HeadValidation::Off,
    }
}

#[test]
fn default_theme_bootstraps_before_styles_and_enables_mpa_transitions() {
    let html = generate_html(&page(), &[], &config(None));
    let color_scheme = html.find("<meta name=\"color-scheme\"").expect("color-scheme metadata");
    let bootstrap = html.find("localStorage.getItem(\"theme\")").expect("theme bootstrap");
    let styles = html.find("<!-- ox-content:styles:start -->").expect("style marker");

    assert!(color_scheme < bootstrap && bootstrap < styles);
    assert!(html.contains("/* ox-content:css:mpa-navigation:start */"));
    assert!(html.contains("@view-transition"));
}

#[test]
fn theme_can_disable_cross_document_transitions_without_disabling_prepaint_theme() {
    let theme = ThemeConfig { view_transitions: Some(false), ..ThemeConfig::default() };
    let html = generate_html(&page(), &[], &config(Some(theme)));

    assert!(!html.contains("@view-transition"));
    assert!(html.contains("localStorage.getItem(\"theme\")"));
}
