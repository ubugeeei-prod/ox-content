fn snapshot_text(value: &str) -> String {
    let mut rendered = String::new();
    for segment in value.split_inclusive('\n') {
        let (line, has_newline) =
            segment.strip_suffix('\n').map_or((segment, false), |line| (line, true));
        let trimmed = line.trim_end_matches([' ', '\t']);
        rendered.push_str(trimmed);
        for ch in line[trimmed.len()..].chars() {
            match ch {
                ' ' => rendered.push_str("<sp>"),
                '\t' => rendered.push_str("<tab>"),
                _ => rendered.push(ch),
            }
        }
        if has_newline {
            rendered.push('\n');
        }
    }
    rendered
}

mod abbr;
mod aside;
mod citations;
mod contributors;
mod image_gallery;
mod lazy_widgets;
mod mobile_css;
mod mpa_navigation;
mod nav_active;
mod navigation_state;
mod rendering;
mod social;
mod theme;
mod theme_quality;
mod timeline;

#[test]
fn search_keydown_ignores_ime_composition() {
    assert!(super::SSG_JS.contains("if (e.isComposing || e.keyCode === 229) return;"));
}

#[test]
fn mobile_menu_script_keeps_state_and_focus_synchronized() {
    assert!(
        super::SSG_JS
            .contains("const setMenuOpen = (open, trigger = null, restoreFocus = false) =>"),
        "desktop and mobile menu controls need one state transition"
    );
    assert!(
        super::SSG_JS.contains("document.body.classList.toggle(\"menu-open\", open);")
            && super::SSG_JS.contains("setAttribute(\"aria-expanded\", String(open))"),
        "scroll locking and expanded state must follow the visual sheet"
    );
    assert!(
        super::SSG_JS.contains("e.key === \"Escape\"")
            && super::SSG_JS.contains("setMenuOpen(false, null, true)")
            && super::SSG_JS.contains("triggerToRestore?.focus()"),
        "only keyboard dismissal should force focus back to the opener"
    );
}

#[test]
fn ssg_css_inlines_shared_magic_links_stylesheet() {
    let magic_links = include_str!("../plugins/magic-links.css");
    assert!(
        super::SSG_CSS.contains(magic_links),
        "built-in SSG must include the published magic-links stylesheet"
    );
    assert!(
        !super::SSG_CSS.contains("/* @include magic-links.css */"),
        "the include marker must be expanded before pages are rendered"
    );
}

#[test]
fn default_theme_surfaces_stay_flat() {
    let default_css = [
        super::SSG_CSS.as_str(),
        super::CONTRIBUTORS_CSS,
        super::FILE_TREE_CSS,
        super::IMAGE_GALLERY_CSS,
        super::TIMELINE_CSS,
        super::CITATIONS_CSS,
        super::not_by_ai::NOT_BY_AI_CSS,
        super::KBD_CSS,
        super::ABBR_CSS,
        super::DEFINITION_LIST_CSS,
        super::header_chrome::HEADER_CHROME_CSS,
        super::heading_permalinks::HEADING_PERMALINK_CSS,
        super::reader_chrome::READER_CHROME_CSS,
    ]
    .join("\n");

    for decorative_effect in ["box-shadow", "linear-gradient(", "radial-gradient("] {
        assert!(
            !default_css.contains(decorative_effect),
            "default theme chrome must use flat surfaces instead of {decorative_effect}"
        );
    }
}

#[test]
fn card_css_collapses_leading_block_margin() {
    assert!(
        super::SSG_CSS.contains(".content .ox-card > :first-child,")
            && super::SSG_CSS.contains(".content .ox-card > :first-of-type,")
            && super::SSG_CSS.contains(".content .ox-link-card > :first-of-type {")
            && super::SSG_CSS.contains("margin-top: 0;"),
        "card headings inherit section margin-top and must not pad the preview"
    );
}

#[test]
fn theme_runtime_restores_preferences_on_history_and_storage_events() {
    assert!(
        super::SSG_JS.contains("window.addEventListener(\"pageshow\", syncThemePreference)")
            && super::SSG_JS.contains("window.addEventListener(\"storage\"")
    );
    assert!(
        super::SSG_JS.contains("return stored === \"light\" || stored === \"dark\"")
            && super::SSG_JS.contains("document.documentElement.removeAttribute(\"data-theme\")")
    );
    assert!(
        super::SSG_JS.contains("try {\n      localStorage.setItem(\"theme\", theme);")
            && super::SSG_JS.contains("The visual preference still applies")
    );
}
