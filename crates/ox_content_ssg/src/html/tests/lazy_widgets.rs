use super::super::*;
use crate::{GeneratedHtmlPage, externalize_shared_page_assets};

fn page(content: &str) -> PageData {
    PageData {
        title: "Guide".to_string(),
        description: None,
        content: content.to_string(),
        toc: vec![],
        last_updated: None,
        contributors: vec![],
        path: "guide".to_string(),
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

fn externalize(html: String) -> crate::ExternalizedAssets {
    externalize_shared_page_assets(
        vec![GeneratedHtmlPage {
            input_path: "guide.md".to_string(),
            output_path: "/tmp/site/guide/index.html".to_string(),
            html,
        }],
        "/tmp/site",
        "/docs/",
    )
}

fn script_srcs(html: &str) -> Vec<String> {
    html.split("<script")
        .skip(1)
        .filter_map(|part| {
            let after_tag = part.split('>').next()?;
            let src = after_tag.split("src=\"").nth(1)?.split('"').next()?;
            Some(src.to_string())
        })
        .collect()
}

fn assert_no_optional_widget_script_srcs(html: &str) {
    let srcs = script_srcs(html);
    for src in &srcs {
        assert!(
            !src.contains("search") && !src.contains("code-play") && !src.contains("island"),
            "optional widget script leaked onto a plain page: {src} in {html}"
        );
        assert!(!src.contains("tabs"), "static pages must not load tabs JS: {src}");
        assert!(!src.contains("mermaid"), "mermaid is static SVG: {src}");
    }
}

#[test]
fn generate_bare_html_has_no_optional_widget_scripts() {
    let html = generate_bare_html("<p>Hello</p>", "Bare");
    assert!(!html.contains("<script"), "{html}");
    assert_no_optional_widget_script_srcs(&html);
}

#[test]
fn themed_page_without_widgets_does_not_emit_optional_script_srcs() {
    let html = generate_html(&page("<h1>Hello</h1><p>Plain prose.</p>"), &[], &config());
    assert!(html.contains("// ox-content:js:core:start"), "{html}");
    assert!(!html.contains("// ox-content:js:tabs:"), "{html}");
    assert!(!html.contains("ox-code-play"), "{html}");
    assert!(!html.contains("data-ox-island"), "{html}");

    let result = externalize(html);
    let page_html = &result.pages[0].html;
    assert_no_optional_widget_script_srcs(page_html);
    assert!(
        result.assets.iter().any(|asset| asset.public_path.contains("ox-content-search-")),
        "search stays a lazy asset, not a script src: {result:?}"
    );
    assert!(
        !result.assets.iter().any(|asset| asset.public_path.contains("tabs")
            || asset.public_path.contains("code-play")
            || asset.public_path.contains("island")
            || asset.public_path.contains("mermaid")),
        "{result:?}"
    );
}

#[test]
fn synced_tabs_emit_a_tabs_script_src() {
    let html = generate_html(
        &page(
            r#"<div class="ox-tabs" data-ox-tab-group="pkg"><div class="ox-tab-panel">A</div></div>"#,
        ),
        &[],
        &config(),
    );
    assert!(html.contains("// ox-content:js:tabs:start"), "{html}");

    let result = externalize(html);
    let srcs = script_srcs(&result.pages[0].html);
    assert!(
        srcs.iter().any(|src| src.contains("ox-content-tabs-")),
        "synced tabs must emit a tabs chunk: {srcs:?}\n{}",
        result.pages[0].html
    );
    assert_no_optional_widget_script_srcs_except_tabs(&result.pages[0].html);
}

fn assert_no_optional_widget_script_srcs_except_tabs(html: &str) {
    for src in script_srcs(html) {
        assert!(
            !src.contains("search") && !src.contains("code-play") && !src.contains("island"),
            "{src}"
        );
    }
}
