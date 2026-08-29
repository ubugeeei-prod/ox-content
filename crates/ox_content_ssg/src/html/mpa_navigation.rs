//! Pre-paint theme restoration and progressive MPA navigation enhancement.

use super::ThemeConfig;

pub(super) const MPA_NAVIGATION_CSS: &str = include_str!("mpa_navigation.css");
pub(super) const THEME_BOOTSTRAP_JS: &str = include_str!("theme_bootstrap.js");

pub(super) fn view_transitions_enabled(theme: Option<&ThemeConfig>) -> bool {
    theme.and_then(|theme| theme.view_transitions).unwrap_or(true)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn view_transitions_default_on_and_allow_theme_opt_out() {
        assert!(view_transitions_enabled(None));
        assert!(view_transitions_enabled(Some(&ThemeConfig::default())));
        assert!(!view_transitions_enabled(Some(&ThemeConfig {
            view_transitions: Some(false),
            ..ThemeConfig::default()
        })));
    }

    #[test]
    fn reduced_motion_does_not_enable_cross_document_transitions() {
        assert!(MPA_NAVIGATION_CSS.contains("prefers-reduced-motion: no-preference"));
        assert!(MPA_NAVIGATION_CSS.contains("@view-transition"));
        assert!(!MPA_NAVIGATION_CSS.contains("prefers-reduced-motion: reduce"));
    }

    #[test]
    fn root_cross_fade_is_overridden_to_stay_opaque() {
        // The UA cross-fade adds the two root snapshots together with
        // `plus-lighter`; anything the root leaves transparent is then counted
        // twice and the page brightens partway through the navigation.
        assert!(MPA_NAVIGATION_CSS.contains("::view-transition-old(root)"));
        assert!(MPA_NAVIGATION_CSS.contains("::view-transition-new(root)"));
        assert!(MPA_NAVIGATION_CSS.contains("mix-blend-mode: normal"));
    }

    #[test]
    fn persistent_chrome_is_named_out_of_the_root_snapshot() {
        for name in ["octc-header", "octc-sidebar", "octc-mobile-footer"] {
            assert!(MPA_NAVIGATION_CSS.contains(name), "missing {name}");
        }
    }

    #[test]
    fn root_element_owns_the_canvas_background() {
        // A `body`-only background is propagated to the canvas, which leaves the
        // repaint behind a cross-document transition to the UA default color.
        assert!(super::super::SSG_CSS.contains("background-color: var(--octc-color-bg);"));
    }

    #[test]
    fn bootstrap_accepts_only_supported_stored_preferences() {
        assert!(THEME_BOOTSTRAP_JS.contains("stored === \"light\" || stored === \"dark\""));
        assert!(THEME_BOOTSTRAP_JS.contains("removeAttribute(\"data-theme\")"));
        assert!(THEME_BOOTSTRAP_JS.contains("try"));
        assert!(THEME_BOOTSTRAP_JS.contains("catch"));
        assert!(!THEME_BOOTSTRAP_JS.contains("{{"));
        assert!(!THEME_BOOTSTRAP_JS.contains("</script"));
    }
}
