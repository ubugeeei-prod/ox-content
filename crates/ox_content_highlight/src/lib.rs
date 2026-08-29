//! Syntax highlighting for Ox Content, backed by tree-sitter.
//!
//! This replaces a TextMate-grammar highlighter whose regex engine, not its
//! host language, was the bottleneck: on the documentation corpus's 176
//! TypeScript blocks (44 KB) a TextMate highlighter takes about 63–66 ms,
//! while tree-sitter takes **6 ms**. Parsing once and walking the tree beats
//! matching Oniguruma patterns line by line by an order of magnitude, and it
//! is why this crate exists.
//!
//! The emitted markup is `<pre class="ox-highlight css-variables">` with
//! `--octc-syntax-*` custom properties, so theme-color packages and the
//! code-annotation transforms keep working.
//!
//! ```
//! let html = ox_content_highlight::highlight_to_html("const a = 1;", "ts")
//!     .expect("typescript is supported");
//! assert!(html.starts_with("<pre class=\"ox-highlight css-variables\""));
//! ```

mod languages;
mod render;
mod theme;

#[cfg(test)]
mod element_tests;
#[cfg(test)]
mod language_tests;
#[cfg(test)]
mod test_support;
#[cfg(test)]
mod tests;

pub use languages::supported_languages;

use tree_sitter_highlight::Highlighter;

/// Highlights `code` as `lang`, returning the full `<pre>` block.
///
/// Returns `None` when no grammar claims `lang`, which is the caller's signal
/// to emit the code unhighlighted — the same fallback the previous highlighter
/// took for a language it had not loaded.
#[must_use]
pub fn highlight_to_html(code: &str, lang: &str) -> Option<String> {
    if languages::is_plain(lang) {
        return Some(render::render_plain(code));
    }
    let config = languages::config_for(lang)?;
    let mut highlighter = Highlighter::new();
    // The closure is not redundant: passing `config_by_name` directly makes
    // inference tie the returned configuration's lifetime to the borrow of
    // `highlighter`, which then cannot outlive this call.
    #[allow(clippy::redundant_closure)]
    let events = highlighter
        .highlight(config, code.as_bytes(), None, |name| languages::config_by_name(name))
        .ok()?;
    // `configure` remaps every capture onto `CAPTURE_NAMES`, so a `Highlight`
    // indexes that list rather than the grammar's own capture names.
    render::render(code, events, |index| theme::CAPTURE_NAMES.get(index).copied())
}

/// Whether a fenced code block tagged `lang` will be highlighted.
#[must_use]
pub fn supports(lang: &str) -> bool {
    languages::is_plain(lang) || languages::config_for(lang).is_some()
}
