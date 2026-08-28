use super::super::footer::generate_footer_html;
use super::super::theme_css::generate_theme_css;
use super::super::*;

#[test]
fn test_generate_html_with_theme() {
    let page_data = PageData {
        title: "Themed Page".to_string(),
        description: None,
        content: "<p>Content</p>".to_string(),
        toc: vec![],
        last_updated: None,
        contributors: vec![],
        path: "themed".to_string(),
        entry_page: None,
        prev: None,
        next: None,
        breadcrumbs: None,
        chrome: PageChromeFlags::default(),
        robots: None,
        canonical: None,
        markdown_source: None,
    };

    let nav_groups = vec![];

    let config = SsgConfig {
        site_name: "Themed Site".to_string(),
        base: "/".to_string(),
        breadcrumb_root_href: None,
        og_image: None,
        locale: None,
        available_locales: None,
        theme: Some(ThemeConfig {
            colors: Some(ThemeColors {
                primary: Some("#3498db".to_string()),
                ..Default::default()
            }),
            footer: Some(ThemeFooter {
                message: Some("Built with ox-content".to_string()),
                copyright: Some("2025 Test".to_string()),
            }),
            ..Default::default()
        }),
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
    };

    let html = generate_html(&page_data, &nav_groups, &config);

    insta::assert_snapshot!(super::snapshot_text(&html));
}

#[test]
fn test_generate_html_with_custom_social_link() {
    let page_data = PageData {
        title: "Social Page".to_string(),
        description: None,
        content: "<p>Content</p>".to_string(),
        toc: vec![],
        last_updated: None,
        contributors: vec![],
        path: "social".to_string(),
        entry_page: None,
        prev: None,
        next: None,
        breadcrumbs: None,
        chrome: PageChromeFlags::default(),
        robots: None,
        canonical: None,
        markdown_source: None,
    };
    let config = SsgConfig {
        site_name: "Social Site".to_string(),
        base: "/".to_string(),
        breadcrumb_root_href: None,
        og_image: None,
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
        theme: Some(ThemeConfig {
            social_links: Some(SocialLinks {
                links: Some(vec![SocialLink {
                    icon: None,
                    icon_svg: Some("<svg viewBox=\"0 0 24 24\"></svg>".to_string()),
                    link: "https://example.com".to_string(),
                    aria_label: Some("Example".to_string()),
                }]),
                ..Default::default()
            }),
            ..Default::default()
        }),
    };

    let html = generate_html(&page_data, &[], &config);

    insta::assert_snapshot!(super::snapshot_text(&html));
}

#[test]
fn test_generate_theme_css() {
    let theme = ThemeConfig {
        colors: Some(ThemeColors {
            primary: Some("#ff0000".to_string()),
            background: Some("#ffffff".to_string()),
            ..Default::default()
        }),
        dark_colors: Some(ThemeColors {
            primary: Some("#ff6666".to_string()),
            ..Default::default()
        }),
        layout: Some(ThemeLayout {
            sidebar_width: Some("300px".to_string()),
            ..Default::default()
        }),
        ..Default::default()
    };

    let css = generate_theme_css(&theme);

    insta::assert_snapshot!(super::snapshot_text(&css));
}

#[test]
fn test_code_background_also_updates_gradient_top() {
    let css = generate_theme_css(&ThemeConfig {
        colors: Some(ThemeColors {
            code_background: Some("#f6f8fa".to_string()),
            ..Default::default()
        }),
        ..Default::default()
    });

    assert!(css.contains("--octc-color-code-bg: #f6f8fa;"));
    assert!(css.contains("--octc-color-code-bg-top: #f6f8fa;"));
}

#[test]
fn test_generate_footer_html() {
    let theme = ThemeConfig {
        footer: Some(ThemeFooter {
            message: Some("Footer message".to_string()),
            copyright: Some("Copyright info".to_string()),
        }),
        ..Default::default()
    };

    let html = generate_footer_html(&theme);

    insta::assert_snapshot!(super::snapshot_text(&html));
}

#[test]
fn theme_asset_paths_are_prefixed_with_the_deployment_base() {
    let page_data = PageData {
        title: "Based".to_string(),
        description: None,
        content: "<p>Content</p>".to_string(),
        toc: vec![],
        last_updated: None,
        contributors: vec![],
        path: "based".to_string(),
        entry_page: None,
        prev: None,
        next: None,
        breadcrumbs: None,
        chrome: PageChromeFlags::default(),
        robots: None,
        canonical: None,
        markdown_source: None,
    };

    let config = |base: &str, logo: &str| SsgConfig {
        site_name: "Based Site".to_string(),
        base: base.to_string(),
        breadcrumb_root_href: None,
        og_image: None,
        locale: None,
        available_locales: None,
        theme: Some(ThemeConfig {
            header: Some(ThemeHeader { logo: Some(logo.to_string()), ..Default::default() }),
            ..Default::default()
        }),
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
    };

    // A root-absolute path used to ship as written, so it 404'd on a site
    // deployed under a sub-path while the generated nav resolved correctly.
    let html = generate_html(&page_data, &[], &config("/team/docs/", "/img/logo.svg"));
    assert!(html.contains(r#"src="/team/docs/img/logo.svg""#), "{html}");

    // A relative path keeps resolving against the base, as before.
    let html = generate_html(&page_data, &[], &config("/team/docs/", "img/logo.svg"));
    assert!(html.contains(r#"src="/team/docs/img/logo.svg""#), "{html}");

    // The default base leaves every shape exactly as authored.
    let html = generate_html(&page_data, &[], &config("/", "/img/logo.svg"));
    assert!(html.contains(r#"src="/img/logo.svg""#), "{html}");

    // Another origin is not this site's to rebase.
    let html =
        generate_html(&page_data, &[], &config("/team/docs/", "https://cdn.example.com/logo.svg"));
    assert!(html.contains(r#"src="https://cdn.example.com/logo.svg""#), "{html}");
}
