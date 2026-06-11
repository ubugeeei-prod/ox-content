use super::ThemeConfig;

/// Generates CSS variable overrides for theme colors.
pub(super) fn generate_theme_css(theme: &ThemeConfig) -> String {
    let mut css = String::new();

    // Light mode colors
    if let Some(ref colors) = theme.colors {
        let mut vars = Vec::new();
        if let Some(ref v) = colors.primary {
            vars.push(format!("--octc-color-primary: {v};"));
        }
        if let Some(ref v) = colors.primary_hover {
            vars.push(format!("--octc-color-primary-hover: {v};"));
        }
        if let Some(ref v) = colors.background {
            vars.push(format!("--octc-color-bg: {v};"));
        }
        if let Some(ref v) = colors.background_alt {
            vars.push(format!("--octc-color-bg-alt: {v};"));
        }
        if let Some(ref v) = colors.text {
            vars.push(format!("--octc-color-text: {v};"));
        }
        if let Some(ref v) = colors.text_muted {
            vars.push(format!("--octc-color-text-muted: {v};"));
        }
        if let Some(ref v) = colors.border {
            vars.push(format!("--octc-color-border: {v};"));
        }
        if let Some(ref v) = colors.code_background {
            vars.push(format!("--octc-color-code-bg: {v};"));
        }
        if let Some(ref v) = colors.code_text {
            vars.push(format!("--octc-color-code-text: {v};"));
        }
        if !vars.is_empty() {
            css.push_str(":root {\n  ");
            css.push_str(&vars.join("\n  "));
            css.push_str("\n}\n");
        }
    }

    // Dark mode colors
    if let Some(ref colors) = theme.dark_colors {
        let mut vars = Vec::new();
        if let Some(ref v) = colors.primary {
            vars.push(format!("--octc-color-primary: {v};"));
        }
        if let Some(ref v) = colors.primary_hover {
            vars.push(format!("--octc-color-primary-hover: {v};"));
        }
        if let Some(ref v) = colors.background {
            vars.push(format!("--octc-color-bg: {v};"));
        }
        if let Some(ref v) = colors.background_alt {
            vars.push(format!("--octc-color-bg-alt: {v};"));
        }
        if let Some(ref v) = colors.text {
            vars.push(format!("--octc-color-text: {v};"));
        }
        if let Some(ref v) = colors.text_muted {
            vars.push(format!("--octc-color-text-muted: {v};"));
        }
        if let Some(ref v) = colors.border {
            vars.push(format!("--octc-color-border: {v};"));
        }
        if let Some(ref v) = colors.code_background {
            vars.push(format!("--octc-color-code-bg: {v};"));
        }
        if let Some(ref v) = colors.code_text {
            vars.push(format!("--octc-color-code-text: {v};"));
        }
        if !vars.is_empty() {
            css.push_str("[data-theme=\"dark\"] {\n  ");
            css.push_str(&vars.join("\n  "));
            css.push_str("\n}\n");
            css.push_str("@media (prefers-color-scheme: dark) {\n  :root:not([data-theme=\"light\"]) {\n    ");
            css.push_str(&vars.join("\n    "));
            css.push_str("\n  }\n}\n");
        }
    }

    // Layout overrides
    if let Some(ref layout) = theme.layout {
        let mut vars = Vec::new();
        if let Some(ref v) = layout.sidebar_width {
            vars.push(format!("--octc-sidebar-width: {v};"));
        }
        if let Some(ref v) = layout.header_height {
            vars.push(format!("--octc-header-height: {v};"));
        }
        if let Some(ref v) = layout.max_content_width {
            vars.push(format!("--octc-max-content-width: {v};"));
        }
        if !vars.is_empty() {
            css.push_str(":root {\n  ");
            css.push_str(&vars.join("\n  "));
            css.push_str("\n}\n");
        }
    }

    // Font overrides
    if let Some(ref fonts) = theme.fonts {
        let mut vars = Vec::new();
        if let Some(ref v) = fonts.sans {
            vars.push(format!("--octc-font-sans: {v};"));
        }
        if let Some(ref v) = fonts.mono {
            vars.push(format!("--octc-font-mono: {v};"));
        }
        if !vars.is_empty() {
            css.push_str(":root {\n  ");
            css.push_str(&vars.join("\n  "));
            css.push_str("\n}\n");
        }
    }

    // Custom CSS
    if let Some(ref custom_css) = theme.css {
        css.push_str(custom_css);
    }

    css
}
