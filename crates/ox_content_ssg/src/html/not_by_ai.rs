//! Conditional CSS for the opt-in `<NotByAI />` authorship badge.

use super::utils::{page_content_contains_any, wrap_css_section};

pub(super) const NOT_BY_AI_CSS: &str = include_str!("../plugins/not-by-ai.css");

const MARKER: &str = "ox-not-by-ai";

pub(super) fn push_not_by_ai_css(css_sections: &mut Vec<String>, content: &str) {
    if page_content_contains_any(content, &[MARKER]) {
        css_sections.push(wrap_css_section("plugin-not-by-ai", NOT_BY_AI_CSS));
    }
}

#[cfg(test)]
mod tests {
    use crate::html::{
        A11y, HeadValidation, PageChromeFlags, PageData, ReaderChrome, SsgConfig, generate_html,
    };

    fn page(content: &str) -> PageData {
        PageData {
            title: "NotByAI".to_string(),
            description: None,
            content: content.to_string(),
            toc: vec![],
            last_updated: None,
            contributors: vec![],
            path: "not-by-ai".to_string(),
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
            base: "/".to_string(),
            breadcrumb_root_href: None,
            og_image: None,
            site_url: None,
            head_validation: HeadValidation::Off,
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
            json_ld: crate::JsonLd::default(),
        }
    }

    fn badge_html() -> &'static str {
        r#"<a class="ox-not-by-ai" href="https://notbyai.fyi" aria-label="Written by human, not by AI" target="_blank" rel="noopener noreferrer"><svg class="ox-not-by-ai__badge ox-not-by-ai__badge--light" aria-hidden="true"></svg><svg class="ox-not-by-ai__badge ox-not-by-ai__badge--dark" aria-hidden="true"></svg></a>"#
    }

    #[test]
    fn css_is_omitted_when_the_page_has_no_badge() {
        let html = generate_html(&page("<p>Hello</p>"), &[], &config());
        assert!(!html.contains("ox-content:css:plugin-not-by-ai"), "{html}");
        assert!(!html.contains(".ox-not-by-ai {"), "{html}");
    }

    #[test]
    fn css_is_included_only_when_the_marker_is_present() {
        let html = generate_html(&page(badge_html()), &[], &config());
        assert!(html.contains("ox-content:css:plugin-not-by-ai"), "{html}");
        assert!(html.contains(".ox-not-by-ai {"), "{html}");
        assert!(html.contains("prefers-color-scheme: dark"), "{html}");
        assert!(html.contains("[data-theme=\"dark\"]"), "{html}");
        assert!(html.contains(".dark .ox-not-by-ai__badge--dark"), "{html}");
        assert!(!html.contains("<script src="), "{html}");
    }
}
