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
        path: "themed".to_string(),
        entry_page: None,
    };

    let nav_groups = vec![];

    let config = SsgConfig {
        site_name: "Themed Site".to_string(),
        base: "/".to_string(),
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
    };

    let html = generate_html(&page_data, &nav_groups, &config);

    // Check theme CSS is applied
    assert!(html.contains("--octc-color-primary: #3498db;"));
    // Check footer is present
    assert!(html.contains("Built with ox-content"));
    assert!(html.contains("2025 Test"));
}

#[test]
fn test_generate_html_with_custom_social_link() {
    let page_data = PageData {
        title: "Social Page".to_string(),
        description: None,
        content: "<p>Content</p>".to_string(),
        toc: vec![],
        last_updated: None,
        path: "social".to_string(),
        entry_page: None,
    };
    let config = SsgConfig {
        site_name: "Social Site".to_string(),
        base: "/".to_string(),
        og_image: None,
        locale: None,
        available_locales: None,
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

    assert!(html.contains("aria-label=\"Example\""));
    assert!(html.contains("href=\"https://example.com\""));
    assert!(html.contains("<svg viewBox=\"0 0 24 24\"></svg>"));
    assert!(html.contains("<span class=\"mobile-footer-label\">Example</span>"));
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

    assert!(css.contains("--octc-color-primary: #ff0000;"));
    assert!(css.contains("--octc-color-bg: #ffffff;"));
    assert!(css.contains("[data-theme=\"dark\"]"));
    assert!(css.contains("--octc-sidebar-width: 300px;"));
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

    assert!(html.contains("site-footer"));
    assert!(html.contains("Footer message"));
    assert!(html.contains("Copyright info"));
}
