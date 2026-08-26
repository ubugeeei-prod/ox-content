use super::super::*;

fn page(content: &str, path: &str) -> PageData {
    PageData {
        title: "Abbr".to_string(),
        description: None,
        content: content.to_string(),
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

fn config() -> SsgConfig {
    SsgConfig {
        site_name: "Test Site".to_string(),
        base: "/".to_string(),
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
fn abbr_css_is_included_only_when_ox_abbr_is_present() {
    let config = config();
    let with = generate_html(
        &page(
            r#"<p><abbr class="ox-abbr" title="Language Server Protocol">LSP</abbr></p>"#,
            "abbr",
        ),
        &[],
        &config,
    );
    assert!(with.contains("ox-content:css:plugin-abbr:start"), "{with}");
    assert!(with.contains(".ox-abbr"), "{with}");
    let without = generate_html(&page("<p>no terms</p>", "plain"), &[], &config);
    assert!(!without.contains("plugin-abbr"), "{without}");
}
