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
