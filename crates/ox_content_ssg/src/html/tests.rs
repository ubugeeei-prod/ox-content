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

mod aside;
mod mpa_navigation;
mod navigation_state;
mod rendering;
mod theme;

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
fn default_theme_surfaces_stay_flat() {
    let default_css = [
        super::SSG_CSS,
        super::header_chrome::HEADER_CHROME_CSS,
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
