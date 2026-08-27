//! JavaScript family and adjacent web grammars.

use super::Grammar;

/// TypeScript ships only the declarations it adds to JavaScript, so its
/// queries are meaningless without the JavaScript ones in front. Without this
/// concatenation a TypeScript block highlights as plain foreground text —
/// every keyword and comment silently unmatched.
fn typescript_highlights() -> String {
    format!(
        "{}\n{}",
        tree_sitter_javascript::HIGHLIGHT_QUERY,
        tree_sitter_typescript::HIGHLIGHTS_QUERY,
    )
}

/// TSX additionally needs the JSX declarations.
fn tsx_highlights() -> String {
    format!(
        "{}\n{}\n{}",
        tree_sitter_javascript::HIGHLIGHT_QUERY,
        tree_sitter_javascript::JSX_HIGHLIGHT_QUERY,
        tree_sitter_typescript::HIGHLIGHTS_QUERY,
    )
}

/// JSX is valid in plain `.js`/`.jsx`, so the base grammar carries it too.
fn javascript_highlights() -> String {
    format!(
        "{}\n{}",
        tree_sitter_javascript::HIGHLIGHT_QUERY,
        tree_sitter_javascript::JSX_HIGHLIGHT_QUERY,
    )
}

fn typescript_locals() -> String {
    format!("{}\n{}", tree_sitter_javascript::LOCALS_QUERY, tree_sitter_typescript::LOCALS_QUERY)
}

pub(super) fn grammars() -> &'static [Grammar] {
    &[
        grammar!(
            "typescript",
            ["typescript", "ts", "cts", "mts"],
            tree_sitter_typescript::LANGUAGE_TYPESCRIPT,
            typescript_highlights(),
            tree_sitter_javascript::INJECTIONS_QUERY,
            typescript_locals(),
        ),
        grammar!(
            "tsx",
            ["tsx", "typescriptreact"],
            tree_sitter_typescript::LANGUAGE_TSX,
            tsx_highlights(),
            tree_sitter_javascript::INJECTIONS_QUERY,
            typescript_locals(),
        ),
        grammar!(
            "javascript",
            ["javascript", "js", "cjs", "mjs", "jsx", "javascriptreact", "flow"],
            tree_sitter_javascript::LANGUAGE,
            javascript_highlights(),
            tree_sitter_javascript::INJECTIONS_QUERY,
            tree_sitter_javascript::LOCALS_QUERY,
        ),
        grammar!(
            "json",
            ["json", "jsonc", "json5", "webmanifest"],
            tree_sitter_json::LANGUAGE,
            tree_sitter_json::HIGHLIGHTS_QUERY,
            "",
            "",
        ),
        grammar!(
            "css",
            ["css"],
            tree_sitter_css::LANGUAGE,
            tree_sitter_css::HIGHLIGHTS_QUERY,
            "",
            "",
        ),
        grammar!(
            "less",
            ["less"],
            tree_sitter_less::language(),
            tree_sitter_less::HIGHLIGHTS_QUERY,
            "",
            "",
        ),
        // `mdx` is here rather than with JavaScript because MDX is not
        // JavaScript. A document may hold adjacent top-level elements —
        // `<A />` on one line, `<B />` on the next — which MDX reads as two
        // blocks and JavaScript reads as one expression: `(<A />) < B / …`,
        // because automatic semicolon insertion will not break before `<`.
        // The JavaScript grammar therefore mis-tokenised every second element
        // as comparison and division operators. No native MDX grammar is
        // published, and HTML tokenises component tags consistently, so MDX's
        // `import` / `export` and `{expression}` parts stay plain rather than
        // being coloured wrongly.
        grammar!(
            "html",
            ["html", "vue", "svelte", "astro", "angular", "mdx"],
            tree_sitter_html::LANGUAGE,
            tree_sitter_html::HIGHLIGHTS_QUERY,
            tree_sitter_html::INJECTIONS_QUERY,
            "",
        ),
    ]
}
