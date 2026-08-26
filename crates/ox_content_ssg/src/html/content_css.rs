use super::utils::{page_content_contains_any, wrap_css_section};
use super::{ABBR_CSS, DEFINITION_LIST_CSS, KBD_CSS};

pub(super) fn push_content_plugin_css(css_sections: &mut Vec<String>, content: &str) {
    if page_content_contains_any(content, &["ox-kbd"]) {
        css_sections.push(wrap_css_section("plugin-kbd", KBD_CSS));
    }
    if page_content_contains_any(content, &["ox-abbr"]) {
        css_sections.push(wrap_css_section("plugin-abbr", ABBR_CSS));
    }
    if page_content_contains_any(content, &["ox-definition-list"]) {
        css_sections.push(wrap_css_section("plugin-definition-list", DEFINITION_LIST_CSS));
    }
}
