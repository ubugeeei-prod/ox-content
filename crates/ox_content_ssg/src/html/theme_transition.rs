//! Opt-in circular reveal for a same-document theme change.
//!
//! Separate from [`super::mpa_navigation`], which covers cross-document
//! navigation. The two lifecycles share the view-transition pseudo tree but
//! not their styling: this one is scoped to an attribute the runtime holds for
//! the length of a single toggle.

use super::ThemeConfig;

pub(super) const THEME_TRANSITION_CSS: &str = include_str!("theme_transition.css");
pub(super) const THEME_TRANSITION_JS: &str = include_str!("theme_transition_runtime.js");

/// The only shape shipped so far. Anything else is treated as "off" rather
/// than failing a build, so a newer config cannot break an older renderer.
const CIRCLE: &str = "circle";

/// Opt-in: an omitted or unrecognised value leaves the toggle immediate.
pub(super) fn toggle_transition_enabled(theme: Option<&ThemeConfig>) -> bool {
    theme.and_then(|theme| theme.toggle_transition.as_deref()) == Some(CIRCLE)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn theme(value: Option<&str>) -> ThemeConfig {
        ThemeConfig { toggle_transition: value.map(str::to_string), ..ThemeConfig::default() }
    }

    #[test]
    fn the_reveal_is_opt_in() {
        assert!(!toggle_transition_enabled(None));
        assert!(!toggle_transition_enabled(Some(&ThemeConfig::default())));
        assert!(toggle_transition_enabled(Some(&theme(Some("circle")))));
    }

    #[test]
    fn an_unknown_shape_leaves_the_toggle_immediate() {
        assert!(!toggle_transition_enabled(Some(&theme(Some("spiral")))));
        assert!(!toggle_transition_enabled(Some(&theme(Some("")))));
    }

    #[test]
    fn the_stylesheet_cannot_reach_the_navigation_transition() {
        // Every rule has to be behind the runtime's attribute. An unscoped
        // `::view-transition-old(root)` here would also land on cross-document
        // navigation, where the UA cross-fade has to be left alone.
        for line in THEME_TRANSITION_CSS.lines() {
            let line = line.trim();
            if !line.contains("::view-transition") {
                continue;
            }
            assert!(
                line.starts_with(":root[data-ox-theme-transition"),
                "unscoped view-transition rule: {line}"
            );
        }
    }

    #[test]
    fn the_runtime_falls_back_instead_of_throwing() {
        assert!(THEME_TRANSITION_JS.contains("prefers-reduced-motion: reduce"));
        assert!(THEME_TRANSITION_JS.contains("startViewTransition"));
        // A skipped transition rejects `finished`; both arms have to settle.
        assert!(THEME_TRANSITION_JS.contains("transition.finished.then(settled, settled)"));
        assert!(!THEME_TRANSITION_JS.contains("{{"));
        assert!(!THEME_TRANSITION_JS.contains("</script"));
    }
}
