//! Grammar registry.
//!
//! Each entry owns a configured [`HighlightConfiguration`], built once and
//! reused for every block in the process. Building one parses the grammar's
//! highlight queries, which is the only meaningful setup cost — about 25 ms
//! for the whole set, against 153 ms for the previous highlighter's grammars.

use std::sync::OnceLock;

use tree_sitter_highlight::HighlightConfiguration;

use crate::theme::CAPTURE_NAMES;

/// A language this crate can highlight.
struct Grammar {
    /// Canonical name, also the name tree-sitter reports for injections.
    name: &'static str,
    /// Every alias a fenced code block may spell it as, canonical name first.
    aliases: &'static [&'static str],
    build: fn() -> Option<HighlightConfiguration>,
}

macro_rules! grammar {
    ($name:literal, [$($alias:literal),* $(,)?], $lang:expr, $highlights:expr, $injections:expr, $locals:expr $(,)?) => {
        Grammar {
            name: $name,
            aliases: &[$($alias),*],
            build: || {
                let mut config = HighlightConfiguration::new(
                    $lang.into(),
                    $name,
                    AsRef::<str>::as_ref(&$highlights),
                    AsRef::<str>::as_ref(&$injections),
                    AsRef::<str>::as_ref(&$locals),
                )
                .ok()?;
                config.configure(CAPTURE_NAMES);
                Some(config)
            },
        }
    };
}

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

fn grammars() -> &'static [Grammar] {
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
            ["tsx"],
            tree_sitter_typescript::LANGUAGE_TSX,
            tsx_highlights(),
            tree_sitter_javascript::INJECTIONS_QUERY,
            typescript_locals(),
        ),
        grammar!(
            "javascript",
            ["javascript", "js", "cjs", "mjs", "jsx"],
            tree_sitter_javascript::LANGUAGE,
            javascript_highlights(),
            tree_sitter_javascript::INJECTIONS_QUERY,
            tree_sitter_javascript::LOCALS_QUERY,
        ),
        grammar!(
            "rust",
            ["rust", "rs"],
            tree_sitter_rust::LANGUAGE,
            tree_sitter_rust::HIGHLIGHTS_QUERY,
            tree_sitter_rust::INJECTIONS_QUERY,
            "",
        ),
        grammar!(
            "json",
            ["json"],
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
            "html",
            ["html"],
            tree_sitter_html::LANGUAGE,
            tree_sitter_html::HIGHLIGHTS_QUERY,
            tree_sitter_html::INJECTIONS_QUERY,
            "",
        ),
        grammar!(
            "bash",
            ["bash", "sh", "shell", "zsh", "shellscript"],
            tree_sitter_bash::LANGUAGE,
            tree_sitter_bash::HIGHLIGHT_QUERY,
            "",
            "",
        ),
    ]
}

/// Configurations indexed the same way as [`grammars`], built on first use.
fn configs() -> &'static [OnceLock<Option<HighlightConfiguration>>] {
    static CONFIGS: OnceLock<Vec<OnceLock<Option<HighlightConfiguration>>>> = OnceLock::new();
    CONFIGS.get_or_init(|| grammars().iter().map(|_| OnceLock::new()).collect())
}

/// The configuration for `lang`, or `None` when no grammar claims it.
///
/// Only the grammar a document actually uses is built, so a page of shell
/// snippets never pays for the TypeScript queries.
pub fn config_for(lang: &str) -> Option<&'static HighlightConfiguration> {
    let index = grammars()
        .iter()
        .position(|grammar| grammar.aliases.iter().any(|alias| alias.eq_ignore_ascii_case(lang)))?;
    configs()[index].get_or_init(grammars()[index].build).as_ref()
}

/// Resolves a grammar by the name tree-sitter reports for an injected region.
pub fn config_by_name(name: &str) -> Option<&'static HighlightConfiguration> {
    let index = grammars().iter().position(|grammar| grammar.name == name)?;
    configs()[index].get_or_init(grammars()[index].build).as_ref()
}

/// Every alias this crate answers to, for the caller's capability check.
pub fn supported_languages() -> impl Iterator<Item = &'static str> {
    grammars().iter().flat_map(|grammar| grammar.aliases.iter().copied())
}
