//! Grammar registry.
//!
//! Each entry owns a configured [`HighlightConfiguration`], built once and
//! reused for every block in the process. Building one parses the grammar's
//! highlight queries, which is the only meaningful setup cost — about 25 ms
//! for the whole set, against 153 ms for the previous highlighter's grammars.

use std::sync::OnceLock;

use tree_sitter_highlight::HighlightConfiguration;

/// A language this crate can highlight.
#[derive(Clone, Copy)]
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
                let mut config = ::tree_sitter_highlight::HighlightConfiguration::new(
                    $lang.into(),
                    $name,
                    ::std::convert::AsRef::<str>::as_ref(&$highlights),
                    ::std::convert::AsRef::<str>::as_ref(&$injections),
                    ::std::convert::AsRef::<str>::as_ref(&$locals),
                )
                .ok()?;
                config.configure($crate::theme::CAPTURE_NAMES);
                Some(config)
            },
        }
    };
}

mod extra;
mod markup;
mod systems;
mod web;

fn grammars() -> impl Iterator<Item = &'static Grammar> {
    web::grammars()
        .iter()
        .chain(systems::grammars().iter())
        .chain(markup::grammars().iter())
        .chain(extra::grammars().iter())
}

/// Configurations indexed the same way as [`grammars`], built on first use.
fn configs() -> &'static [OnceLock<Option<HighlightConfiguration>>] {
    static CONFIGS: OnceLock<Vec<OnceLock<Option<HighlightConfiguration>>>> = OnceLock::new();
    CONFIGS.get_or_init(|| grammars().map(|_| OnceLock::new()).collect())
}

/// The configuration for `lang`, or `None` when no grammar claims it.
///
/// Only the grammar a document actually uses is built, so a page of shell
/// snippets never pays for the TypeScript queries.
pub fn config_for(lang: &str) -> Option<&'static HighlightConfiguration> {
    let (index, grammar) = grammars().enumerate().find(|(_, grammar)| {
        grammar.aliases.iter().any(|alias| alias.eq_ignore_ascii_case(lang))
    })?;
    configs()[index].get_or_init(grammar.build).as_ref()
}

/// Resolves a grammar for an injected region.
///
/// An injection names its language however the source spelled it — a fenced
/// block inside Markdown says `ts`, not `typescript` — so this has to match
/// aliases too. Matching only the canonical name leaves injected regions
/// silently unhighlighted.
pub fn config_by_name(name: &str) -> Option<&'static HighlightConfiguration> {
    let (index, grammar) = grammars().enumerate().find(|(_, grammar)| {
        grammar.name == name || grammar.aliases.iter().any(|alias| alias.eq_ignore_ascii_case(name))
    })?;
    configs()[index].get_or_init(grammar.build).as_ref()
}

/// Names for "this block has no syntax", rendered without tokenizing.
pub const PLAIN_LANGUAGES: &[&str] = &[
    "text",
    "plaintext",
    "txt",
    "plain",
    "dotenv",
    "env",
    ".env",
    "gitignore",
    "dockerignore",
    "npmrc",
    "yarnrc",
    "properties",
    "ini",
    "conf",
    "config",
];

/// Whether `lang` names a block that should render without tokenizing.
pub fn is_plain(lang: &str) -> bool {
    PLAIN_LANGUAGES.iter().any(|name| name.eq_ignore_ascii_case(lang))
}

/// Every alias this crate answers to, for the caller's capability check.
pub fn supported_languages() -> impl Iterator<Item = &'static str> {
    grammars()
        .flat_map(|grammar| grammar.aliases.iter().copied())
        .chain(PLAIN_LANGUAGES.iter().copied())
}
