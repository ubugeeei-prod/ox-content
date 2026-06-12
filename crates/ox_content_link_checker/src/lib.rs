//! Dead link checker for Ox Content Markdown.
//!
//! Resolves every `Link` / `Image` / link reference definition emitted
//! by the parser against the filesystem and against the document's own
//! heading slugs. The checker is intentionally **offline-only**:
//! external HTTP links pass through with no network call so the same
//! binary is safe to run in CI without timeouts, retries, or rate
//! limits, and produces deterministic output across runs. A future
//! `http-check` feature flag can layer network checks on top without
//! changing this contract.
//!
//! ```
//! use ox_content_link_checker::{check_source, CheckOptions};
//! use std::path::PathBuf;
//!
//! let source = "[broken](missing.md)\n";
//! let opts = CheckOptions::for_file(PathBuf::from("/tmp/doc.md"));
//! let diagnostics = check_source(source, &opts);
//! assert_eq!(diagnostics.len(), 1);
//! ```

use std::path::Path;

use ox_content_allocator::Allocator;
use ox_content_parser::{Parser, ParserOptions};

use anchors::collect_anchors;
use line_index::LineIndex;
pub use types::{CheckOptions, Diagnostic, LinkKind, Severity};
use walker::Walker;

mod anchors;
mod line_index;
mod target;
mod types;
mod walker;

/// Run the checker over a Markdown source string and return the
/// (possibly empty) list of diagnostics.
#[must_use]
pub fn check_source(source: &str, options: &CheckOptions) -> Vec<Diagnostic> {
    let allocator = Allocator::for_source_len(source.len());
    let parser = Parser::with_options(&allocator, source, ParserOptions::gfm());
    let Ok(document) = parser.parse() else {
        return Vec::new();
    };

    let line_index = LineIndex::new(source);
    let anchors = collect_anchors(source, &document);
    let base_dir = options.file_path.parent().map(Path::to_path_buf);

    let mut walker = Walker::new(
        &line_index,
        &anchors,
        base_dir.as_deref(),
        options.src_dir.as_deref(),
        &options.ignore_patterns,
    );
    walker.walk(&document.children);
    walker.into_diagnostics()
}

#[cfg(test)]
mod tests;
