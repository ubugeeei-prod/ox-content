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

/// C++ extends C the same way TypeScript extends JavaScript: its queries
/// declare only the additions, so C's have to come first.
fn cpp_highlights() -> String {
    format!("{}\n{}", tree_sitter_c::HIGHLIGHT_QUERY, tree_sitter_cpp::HIGHLIGHT_QUERY)
}

/// Markdown ships as a pair of grammars: a block one that leaves each run of
/// prose as a single `inline` node, and a separate inline one for what is
/// inside it. Injecting the second into the first is what gets emphasis, links
/// and code spans highlighted; the block grammar alone sees only headings,
/// fences and punctuation.
///
/// `injection.include-children` is load-bearing. Without it the injected layer
/// is created — the language callback even fires — and then emits nothing at
/// all, because the range handed to it has the content node's children carved
/// out of it. It fails silently, as unhighlighted prose.
fn markdown_block_injections() -> String {
    format!(
        "{}\n((inline) @injection.content \
         (#set! injection.language \"markdown_inline\") \
         (#set! injection.include-children))",
        tree_sitter_md::INJECTION_QUERY_BLOCK,
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
            "python",
            ["python", "py"],
            tree_sitter_python::LANGUAGE,
            tree_sitter_python::HIGHLIGHTS_QUERY,
            "",
            "",
        ),
        grammar!(
            "go",
            ["go", "golang"],
            tree_sitter_go::LANGUAGE,
            tree_sitter_go::HIGHLIGHTS_QUERY,
            "",
            "",
        ),
        grammar!(
            "java",
            ["java"],
            tree_sitter_java::LANGUAGE,
            tree_sitter_java::HIGHLIGHTS_QUERY,
            "",
            "",
        ),
        grammar!("c", ["c", "h"], tree_sitter_c::LANGUAGE, tree_sitter_c::HIGHLIGHT_QUERY, "", "",),
        grammar!(
            "cpp",
            ["cpp", "c++", "cc", "hpp", "cxx"],
            tree_sitter_cpp::LANGUAGE,
            cpp_highlights(),
            "",
            "",
        ),
        grammar!(
            "yaml",
            ["yaml", "yml"],
            tree_sitter_yaml::LANGUAGE,
            tree_sitter_yaml::HIGHLIGHTS_QUERY,
            "",
            "",
        ),
        grammar!(
            "markdown",
            ["markdown", "md"],
            tree_sitter_md::LANGUAGE,
            tree_sitter_md::HIGHLIGHT_QUERY_BLOCK,
            markdown_block_injections(),
            "",
        ),
        // Reachable only as an injection target, never as a fence tag.
        grammar!(
            "markdown_inline",
            [],
            tree_sitter_md::INLINE_LANGUAGE,
            tree_sitter_md::HIGHLIGHT_QUERY_INLINE,
            tree_sitter_md::INJECTION_QUERY_INLINE,
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
        grammar!(
            "toml",
            ["toml"],
            tree_sitter_toml_ng::LANGUAGE,
            tree_sitter_toml_ng::HIGHLIGHTS_QUERY,
            "",
            "",
        ),
        grammar!(
            "wgsl",
            ["wgsl"],
            tree_sitter_wgsl_bevy::LANGUAGE,
            include_str!("../queries/wgsl-highlights.scm"),
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

/// Resolves a grammar for an injected region.
///
/// An injection names its language however the source spelled it — a fenced
/// block inside Markdown says `ts`, not `typescript` — so this has to match
/// aliases too. Matching only the canonical name leaves injected regions
/// silently unhighlighted.
pub fn config_by_name(name: &str) -> Option<&'static HighlightConfiguration> {
    let index = grammars().iter().position(|grammar| {
        grammar.name == name || grammar.aliases.iter().any(|alias| alias.eq_ignore_ascii_case(name))
    })?;
    configs()[index].get_or_init(grammars()[index].build).as_ref()
}

/// Names for "this block has no syntax", rendered without tokenizing.
pub const PLAIN_LANGUAGES: &[&str] = &["text", "plaintext", "txt", "plain"];

/// Whether `lang` names a block that should render without tokenizing.
pub fn is_plain(lang: &str) -> bool {
    PLAIN_LANGUAGES.iter().any(|name| name.eq_ignore_ascii_case(lang))
}

/// Every alias this crate answers to, for the caller's capability check.
pub fn supported_languages() -> impl Iterator<Item = &'static str> {
    grammars()
        .iter()
        .flat_map(|grammar| grammar.aliases.iter().copied())
        .chain(PLAIN_LANGUAGES.iter().copied())
}
