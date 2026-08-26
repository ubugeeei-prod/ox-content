use super::header_chrome::{HEADER_CHROME_JS, header_chrome_needs_js};
use super::reader_chrome::READER_CHROME_JS;
use super::utils::{page_content_contains_any, wrap_js_section};
use super::{SSG_JS, TABS_JS};

pub(super) struct PageJsInput<'a> {
    pub content: &'a str,
    pub base: &'a str,
    pub custom_js: &'a str,
    pub header_nav_html: &'a str,
    pub announcement_html: &'a str,
    pub locale_switcher_html: &'a str,
    pub markdown_source_chrome: bool,
    pub reader_needs_js: bool,
}

/// Assembles feature-level JS the same way page CSS is sectioned.
///
/// Core chrome (menu, theme, search *loader*) is always present on themed
/// pages. Search *implementation* stays inside the core blob with lazy-load
/// markers. Optional widgets are omitted unless the page content needs them:
/// synced tabs, reader chrome, header chrome, and custom theme JS. Mermaid is
/// static SVG. Islands and Code Play are injected by their own hosts.
pub(super) fn assemble_page_js(input: &PageJsInput<'_>) -> String {
    let mut sections = Vec::new();
    sections.push(wrap_js_section("core", &SSG_JS.replace("{{base}}", input.base)));
    if page_content_contains_any(input.content, &["data-ox-tab-group"]) {
        sections.push(wrap_js_section("tabs", TABS_JS));
    }
    if input.reader_needs_js {
        sections.push(wrap_js_section("reader-chrome", READER_CHROME_JS));
    }
    if header_chrome_needs_js(
        input.header_nav_html,
        input.announcement_html,
        input.locale_switcher_html,
        input.markdown_source_chrome,
    ) {
        sections.push(wrap_js_section("header-chrome", HEADER_CHROME_JS));
    }
    if !input.custom_js.trim().is_empty() {
        sections.push(wrap_js_section("theme", input.custom_js));
    }
    sections.concat()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn input<'a>(content: &'a str, custom_js: &'a str) -> PageJsInput<'a> {
        PageJsInput {
            content,
            base: "/docs/",
            custom_js,
            header_nav_html: "",
            announcement_html: "",
            locale_switcher_html: "",
            markdown_source_chrome: false,
            reader_needs_js: false,
        }
    }

    #[test]
    fn core_is_always_present_and_optional_widgets_are_omitted() {
        let js = assemble_page_js(&input("<p>Hello</p>", ""));
        assert!(js.contains("// ox-content:js:core:start"), "{js}");
        assert!(js.contains("__OX_CONTENT_SEARCH_CHUNK__"), "{js}");
        assert!(js.contains("// ox-content:search:start"), "{js}");
        assert!(!js.contains("// ox-content:js:tabs:"), "{js}");
        assert!(!js.contains("data-ox-tab-group"), "{js}");
        assert!(!js.contains("ox-code-play"), "{js}");
        assert!(!js.contains("initIslands"), "{js}");
        assert!(!js.contains("mermaid"), "{js}");
    }

    #[test]
    fn synced_tabs_are_the_only_reason_to_emit_tabs_js() {
        let static_tabs = assemble_page_js(&input(r#"<div class="ox-tabs"></div>"#, ""));
        assert!(!static_tabs.contains("// ox-content:js:tabs:"), "{static_tabs}");

        let synced =
            assemble_page_js(&input(r#"<div class="ox-tabs" data-ox-tab-group="pkg"></div>"#, ""));
        assert!(synced.contains("// ox-content:js:tabs:start"), "{synced}");
        assert!(synced.contains("STORAGE_PREFIX"), "{synced}");
    }

    #[test]
    fn header_and_reader_chrome_are_feature_sections() {
        let mut header = input("<p>Hello</p>", "");
        header.header_nav_html = r#"<button aria-expanded="false"></button>"#;
        let js = assemble_page_js(&header);
        assert!(js.contains("// ox-content:js:header-chrome:start"), "{js}");

        let mut reader = input("<p>Hello</p>", "");
        reader.reader_needs_js = true;
        let js = assemble_page_js(&reader);
        assert!(js.contains("// ox-content:js:reader-chrome:start"), "{js}");
    }
}
