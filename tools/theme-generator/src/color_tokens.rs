use crate::color_math::{backdrop, contrast, ensure_contrast, mix, mix_into, rgb_triplet_text};
use crate::colors::ColorMode;

pub(crate) fn tokens_for(
    mode: &ColorMode,
    other: &ColorMode,
    mode_name: &str,
) -> Vec<(&'static str, String)> {
    let code = if legibility(mode, &mode.code_bg) >= legibility(other, &mode.code_bg) {
        mode
    } else {
        other
    };
    let syntax_string =
        mode.syntax_value(|syntax| syntax.string.as_ref()).unwrap_or_else(|| code.green.clone());

    vec![
        ("color-code-line-highlight", mix(&mode.blue, 16)),
        ("color-code-line-warning", mix(&mode.yellow, 18)),
        ("color-code-line-warning-border", mode.yellow.clone()),
        ("color-code-line-error", mix(&mode.red, 20)),
        ("color-code-line-error-border", mode.red.clone()),
        ("color-code-line-add", mix(&mode.green, 16)),
        ("color-code-line-add-border", mode.green.clone()),
        ("color-code-line-remove", mix(&mode.red, 14)),
        ("color-code-line-remove-border", mode.red.clone()),
        ("color-code-line-focus", mix(&mode.primary, 16)),
        ("color-code-line-dim", dim_value(mode_name)),
        ("color-code-title-bg", mix_into(&mode.code_bg, 92, &mode.code_text)),
        ("color-code-title-text", mix_into(&mode.code_text, 88, &mode.code_bg)),
        ("color-code-title-border", mix(&mode.code_text, 22)),
        ("color-code-line-number", mix(&mode.code_text, 48)),
        ("color-code-frame-border", mix(&mode.border, 92)),
        ("color-backdrop", backdrop(&mode.bg)),
        ("surface-glass", mix_into(&mode.bg_alt, 62, &mode.bg)),
        ("surface-line", mix_into(&mode.border, 72, &mode.bg)),
        ("brand-violet", mode.magenta.clone()),
        ("brand-cyan", mode.cyan.clone()),
        ("brand-lime", mode.green.clone()),
        ("brand-coral", mode.red.clone()),
        ("brand-navy", mix_into(&mode.text, 70, &mode.bg)),
        ("brand-violet-rgb", rgb_triplet_text(&mode.magenta)),
        ("brand-cyan-rgb", rgb_triplet_text(&mode.cyan)),
        ("brand-lime-rgb", rgb_triplet_text(&mode.green)),
        ("brand-coral-rgb", rgb_triplet_text(&mode.red)),
        ("accent-a", mode.primary.clone()),
        ("accent-b", mode.magenta.clone()),
        ("accent-c", mode.cyan.clone()),
        ("accent-warm", mode.yellow.clone()),
        ("accent-cool", mode.blue.clone()),
        ("accent-a-ink", ensure_contrast(&mode.primary, &mode.bg, 4.5)),
        ("accent-b-ink", ensure_contrast(&mode.magenta, &mode.bg, 4.5)),
        ("accent-c-ink", ensure_contrast(&mode.cyan, &mode.bg, 4.5)),
        ("accent-warm-ink", ensure_contrast(&mode.yellow, &mode.bg, 4.5)),
        ("accent-cool-ink", ensure_contrast(&mode.blue, &mode.bg, 4.5)),
        ("accent-coral-ink", ensure_contrast(&mode.red, &mode.bg, 4.5)),
        (
            "syntax-foreground",
            syntax_or(mode, |syntax| syntax.foreground.as_ref(), &mode.code_text),
        ),
        ("syntax-background", syntax_or(mode, |syntax| syntax.background.as_ref(), &mode.code_bg)),
        (
            "syntax-token-comment",
            mode.syntax_value(|syntax| syntax.comment.as_ref())
                .unwrap_or_else(|| mix_into(&mode.code_text, 55, &mode.code_bg)),
        ),
        (
            "syntax-token-punctuation",
            mode.syntax_value(|syntax| syntax.punctuation.as_ref())
                .unwrap_or_else(|| mix_into(&mode.code_text, 72, &mode.code_bg)),
        ),
        ("syntax-token-keyword", syntax_or(mode, |syntax| syntax.keyword.as_ref(), &code.magenta)),
        ("syntax-token-string", syntax_string.clone()),
        (
            "syntax-token-string-expression",
            mode.syntax_value(|syntax| syntax.string_expression.as_ref()).unwrap_or(syntax_string),
        ),
        ("syntax-token-constant", syntax_or(mode, |syntax| syntax.constant.as_ref(), &code.yellow)),
        ("syntax-token-function", syntax_or(mode, |syntax| syntax.function.as_ref(), &code.blue)),
        ("syntax-token-parameter", syntax_or(mode, |syntax| syntax.parameter.as_ref(), &code.red)),
        ("syntax-token-link", syntax_or(mode, |syntax| syntax.link.as_ref(), &code.cyan)),
    ]
}

fn syntax_or(
    mode: &ColorMode,
    field: impl FnOnce(&crate::colors::SyntaxColors) -> Option<&String>,
    fallback: &str,
) -> String {
    mode.syntax_value(field).unwrap_or_else(|| fallback.to_string())
}

fn dim_value(mode_name: &str) -> String {
    if mode_name == "dark" { "0.35" } else { "0.45" }.to_string()
}

fn legibility(colors: &ColorMode, code_bg: &str) -> f64 {
    [&colors.red, &colors.green, &colors.yellow, &colors.blue, &colors.magenta, &colors.cyan]
        .into_iter()
        .map(|color| contrast(color, code_bg))
        .sum()
}
