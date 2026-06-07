//! Document link resolution for Markdown / MDC documents.
//!
//! Turns every Markdown link and image into an LSP `DocumentLink` so the
//! editor can ctrl/cmd-click straight to the target:
//!   * external URIs (`https:`, `mailto:`, …) open as-is,
//!   * relative paths resolve against the document's directory,
//!   * root-absolute paths (`/guide.md`) resolve against the workspace
//!     root when one is known, else the document's directory.
//!
//! Pure in-page anchors (`#section`) and reference targets the parser
//! could not resolve are skipped — there's nothing to navigate to.

use std::path::Path;

use ox_content_allocator::Allocator;
use ox_content_ast::{walk_link, Image, Link, Span, Visit};
use ox_content_parser::{Parser, ParserOptions};
use tower_lsp::lsp_types::{DocumentLink, Url};

use crate::document::TextDocumentState;
use crate::frontmatter::parse_frontmatter;

/// Computes the clickable links in `document`. `doc_path` is the file the
/// document was opened from (used as the base for relative links) and
/// `root` is the workspace root used to anchor root-absolute links.
#[must_use]
pub fn document_links(
    document: &TextDocumentState,
    doc_path: &Path,
    root: Option<&Path>,
) -> Vec<DocumentLink> {
    let source = document.text();
    let (content, base_offset) = match parse_frontmatter(document).block {
        Some(block) => (&source[block.block_end_offset..], block.block_end_offset),
        None => (source, 0),
    };

    let allocator = Allocator::for_source_len(content.len());
    let parser = Parser::with_options(&allocator, content, ParserOptions::gfm());
    let Ok(ast) = parser.parse() else {
        return Vec::new();
    };

    let mut collector = LinkCollector { found: Vec::new() };
    collector.visit_document(&ast);

    collector
        .found
        .into_iter()
        .filter_map(|(span, url)| {
            let target = resolve_target(url, doc_path, root)?;
            let (start, len) = url_subspan(content, span, url);
            let range = document.range_from_offsets(base_offset + start, base_offset + start + len);
            Some(DocumentLink { range, target: Some(target), tooltip: None, data: None })
        })
        .collect()
}

/// Collects `(span, url)` for every link and image in document order.
/// Implementing `Visit` lets the AST walker reach links nested anywhere —
/// inside headings, lists, tables, blockquotes, emphasis, and so on.
struct LinkCollector<'a> {
    found: Vec<(Span, &'a str)>,
}

impl<'a> Visit<'a> for LinkCollector<'a> {
    fn visit_link(&mut self, link: &Link<'a>) {
        self.found.push((link.span, link.url));
        // Keep walking: a link's label can itself contain an image.
        walk_link(self, link);
    }

    fn visit_image(&mut self, image: &Image<'a>) {
        self.found.push((image.span, image.url));
    }
}

/// Locates the URL inside the link's source text so the clickable range
/// hugs the target rather than the whole `[label](url)` construct. Falls
/// back to the entire span for reference-style links whose URL lives in a
/// definition elsewhere and therefore isn't present in the span text.
fn url_subspan(content: &str, span: Span, url: &str) -> (usize, usize) {
    let span_start = span.start as usize;
    let text = &content[span_start..span.end as usize];
    if !url.is_empty() {
        if let Some(rel) = text.rfind(url) {
            return (span_start + rel, url.len());
        }
    }
    (span_start, text.len())
}

/// Resolves a raw Markdown URL into a navigable `Url`, or `None` when
/// there's nothing to point at (empty, in-page anchor, or unparsable).
fn resolve_target(url: &str, doc_path: &Path, root: Option<&Path>) -> Option<Url> {
    let url = url.trim();
    if url.is_empty() || url.starts_with('#') {
        return None;
    }

    // Already a full URI (http, https, mailto, …): hand it back verbatim.
    if let Ok(parsed) = Url::parse(url) {
        return Some(parsed);
    }

    // Local path — drop any `#fragment` / `?query` before touching the
    // filesystem so `./other.md#section` still resolves to `other.md`.
    let path_part = url.split(['#', '?']).next().unwrap_or(url);
    if path_part.is_empty() {
        return None;
    }

    let resolved = if let Some(stripped) = path_part.strip_prefix('/') {
        let base = root.or_else(|| doc_path.parent())?;
        base.join(stripped)
    } else {
        doc_path.parent()?.join(path_part)
    };
    Url::from_file_path(normalize(&resolved)).ok()
}

/// Collapses `.` and `..` components lexically so the resulting URI is the
/// clean path the editor shows (`/proj/guide.md`, not
/// `/proj/docs/../guide.md`). Markdown links are URL-relative, so lexical
/// resolution — not symlink-aware `canonicalize` — is the correct model,
/// and it works for files that don't exist yet.
fn normalize(path: &Path) -> std::path::PathBuf {
    use std::path::Component;
    let mut out = std::path::PathBuf::new();
    for component in path.components() {
        match component {
            Component::ParentDir => {
                out.pop();
            }
            Component::CurDir => {}
            other => out.push(other),
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    fn links(source: &str) -> Vec<DocumentLink> {
        let document = TextDocumentState::new(source.to_string());
        let doc_path = PathBuf::from("/proj/docs/page.md");
        let root = PathBuf::from("/proj");
        document_links(&document, &doc_path, Some(&root))
    }

    fn target_path(link: &DocumentLink) -> PathBuf {
        link.target.as_ref().expect("link has a target").to_file_path().expect("file URI")
    }

    #[test]
    fn external_url_is_handed_back_verbatim() {
        let result = links("See [site](https://example.com/docs).\n");
        assert_eq!(result.len(), 1);
        let target = result[0].target.as_ref().unwrap();
        assert_eq!(target.scheme(), "https");
        assert_eq!(target.as_str(), "https://example.com/docs");
    }

    #[test]
    fn relative_link_resolves_against_document_dir() {
        let result = links("[next](./other.md)\n");
        assert_eq!(result.len(), 1);
        assert_eq!(target_path(&result[0]), PathBuf::from("/proj/docs/other.md"));
    }

    #[test]
    fn relative_link_with_anchor_strips_the_fragment() {
        let result = links("[deep](../guide.md#install)\n");
        assert_eq!(result.len(), 1);
        assert_eq!(target_path(&result[0]), PathBuf::from("/proj/guide.md"));
    }

    #[test]
    fn root_absolute_link_resolves_against_workspace_root() {
        let result = links("[home](/index.md)\n");
        assert_eq!(result.len(), 1);
        assert_eq!(target_path(&result[0]), PathBuf::from("/proj/index.md"));
    }

    #[test]
    fn image_target_is_linked() {
        let result = links("![logo](./assets/logo.png)\n");
        assert_eq!(result.len(), 1);
        assert_eq!(target_path(&result[0]), PathBuf::from("/proj/docs/assets/logo.png"));
    }

    #[test]
    fn in_page_anchor_is_skipped() {
        assert!(links("[top](#intro)\n").is_empty());
    }

    #[test]
    fn link_range_hugs_the_url_not_the_label() {
        // `[label](./x.md)` — the clickable range should cover `./x.md`,
        // starting after `[label](`.
        let result = links("[label](./x.md)\n");
        assert_eq!(result.len(), 1);
        let range = result[0].range;
        assert_eq!(range.start.character, "[label](".len() as u32);
        assert_eq!(range.end.character, "[label](./x.md".len() as u32);
    }

    #[test]
    fn finds_links_nested_inside_other_constructs() {
        // A link inside a heading and another inside a list item should
        // both be discovered by the visitor walk.
        let source = "# Title with [a](./a.md)\n\n- item [b](./b.md)\n";
        let mut result = links(source);
        result.sort_by_key(|link| link.range.start.line);
        assert_eq!(result.len(), 2);
        assert_eq!(target_path(&result[0]), PathBuf::from("/proj/docs/a.md"));
        assert_eq!(target_path(&result[1]), PathBuf::from("/proj/docs/b.md"));
    }

    #[test]
    fn links_account_for_frontmatter_offset() {
        let source = "---\ntitle: Doc\n---\n\n[next](./other.md)\n";
        let result = links(source);
        assert_eq!(result.len(), 1);
        // The link sits on line 4 of the original document, not line 0 of
        // the frontmatter-stripped body.
        assert_eq!(result[0].range.start.line, 4);
        assert_eq!(target_path(&result[0]), PathBuf::from("/proj/docs/other.md"));
    }
}
