use super::KBD_CSS;
use super::utils::{page_content_contains_any, wrap_css_section};

pub(super) fn push_content_plugin_css(css_sections: &mut Vec<String>, content: &str) {
    if page_content_contains_any(content, &["ox-kbd"]) {
        css_sections.push(wrap_css_section("plugin-kbd", KBD_CSS));
    }
}
