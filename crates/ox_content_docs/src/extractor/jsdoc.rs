use ox_jsdoc::parser::{
    parse_batch_to_bytes as parse_jsdoc_batch_to_bytes, BatchItem as JsdocBatchItem,
    ParseOptions as JsdocParseOptions,
};
use oxc_ast::ast::Comment;
use rustc_hash::{FxBuildHasher, FxHashMap, FxHashSet};

use super::{DocTag, DocVisitor};

/// Pre-parsed JSDoc data for one comment: `(raw, description, tags)`.
pub(super) type ParsedJsdoc = (String, String, Vec<DocTag>);

/// JSDoc/TSDoc tags that mark a leading comment as a module/file comment.
pub(super) const MODULE_MARKER_TAGS: [&str; 3] = ["module", "packageDocumentation", "fileoverview"];

/// Extract the JSDoc content body from a comment with a single allocation.
fn extract_raw_jsdoc(comment: &Comment, source: &str) -> String {
    let content = comment.content_span().source_text(source);
    let trimmed = content.strip_prefix('*').unwrap_or(content);
    trimmed.trim_matches('\n').to_string()
}

/// Parse a single JSDoc comment into `(raw, description, tags)` directly from its
/// own span.
///
/// Unlike [`build_jsdoc_cache`], this does not key by `attached_to`, so it stays
/// correct for comments that share an `attached_to` target — e.g. two leading
/// file comments (`/** … @module */` followed by `/** @author … */`) that the
/// parser both attaches to the first statement. Keying by `attached_to` would let
/// the second comment overwrite the first in the cache.
pub(super) fn parse_jsdoc_payload(source: &str, comment: &Comment) -> ParsedJsdoc {
    let raw = extract_raw_jsdoc(comment, source);
    let items = [JsdocBatchItem {
        source_text: comment.span.source_text(source),
        base_offset: comment.span.start,
    }];
    let options = JsdocParseOptions { preserve_whitespace: true, ..JsdocParseOptions::default() };
    let result = parse_jsdoc_batch_to_bytes(&items, options);

    if result.diagnostics.is_empty() {
        if let Ok(source_file) =
            ox_jsdoc::decoder::source_file::LazySourceFile::new(&result.binary_bytes)
        {
            if let Some(Some(root)) = source_file.asts().next() {
                let doc = root
                    .description_text(false)
                    .map_or_else(String::new, |description| description.trim().to_string());
                // Module-entry parse happens once per file, so always format the
                // tag value (cheap, and the raw path may surface it).
                let tags =
                    root.tags().map(|tag| DocVisitor::convert_jsdoc_tag(tag, true)).collect();
                return (raw, doc, tags);
            }
        }
    }

    let (doc, tags) = DocVisitor::parse_jsdoc_fallback(&raw);
    (raw, doc, tags)
}

/// Pre-parses every JSDoc comment in the program with a single batch call.
///
/// Oxc exposes comments separately from declarations, and the visitor resolves
/// documentation by `attached_to` while walking the AST. Parsing each comment
/// on demand repeats decoder setup and makes the visitor pay parse cost at
/// every declaration. This cache batches decoder input once, stores the parsed
/// `(raw, description, tags)` payload by `attached_to`, and lets the visitor do
/// a cheap hash lookup in the hot declaration path. Comments that fail the
/// batch parser still fall back individually so diagnostics do not poison the
/// whole file.
pub(super) fn build_jsdoc_cache(
    source: &str,
    comments: &[Comment],
    capture_raw: bool,
) -> FxHashMap<u32, ParsedJsdoc> {
    profile_span!("docs::parse_jsdoc");
    let jsdoc_comments: Vec<&Comment> =
        comments.iter().filter(|comment| comment.is_jsdoc()).collect();
    if jsdoc_comments.is_empty() {
        return FxHashMap::default();
    }

    let items: Vec<JsdocBatchItem<'_>> = jsdoc_comments
        .iter()
        .map(|comment| JsdocBatchItem {
            source_text: comment.span.source_text(source),
            base_offset: comment.span.start,
        })
        .collect();

    let options = JsdocParseOptions { preserve_whitespace: true, ..JsdocParseOptions::default() };
    let result = parse_jsdoc_batch_to_bytes(&items, options);

    let failed: FxHashSet<u32> =
        result.diagnostics.iter().map(|diagnostic| diagnostic.root_index).collect();

    let mut cache: FxHashMap<u32, ParsedJsdoc> =
        FxHashMap::with_capacity_and_hasher(jsdoc_comments.len(), FxBuildHasher);

    if let Ok(source_file) =
        ox_jsdoc::decoder::source_file::LazySourceFile::new(&result.binary_bytes)
    {
        for (index, (comment, root)) in jsdoc_comments.iter().zip(source_file.asts()).enumerate() {
            let (raw, doc, tags) = match root {
                Some(root) if !failed.contains(&(index as u32)) => {
                    let doc = root
                        .description_text(false)
                        .map_or_else(String::new, |description| description.trim().to_string());
                    let tags = root
                        .tags()
                        .map(|tag| DocVisitor::convert_jsdoc_tag(tag, capture_raw))
                        .collect();
                    let raw = capture_raw.then(|| extract_raw_jsdoc(comment, source));
                    (raw.unwrap_or_default(), doc, tags)
                }
                _ => {
                    let raw = extract_raw_jsdoc(comment, source);
                    let (doc, tags) = DocVisitor::parse_jsdoc_fallback(&raw);
                    (if capture_raw { raw } else { String::new() }, doc, tags)
                }
            };
            cache.insert(comment.attached_to, (raw, doc, tags));
        }
    } else {
        for comment in &jsdoc_comments {
            let raw = extract_raw_jsdoc(comment, source);
            let (doc, tags) = DocVisitor::parse_jsdoc_fallback(&raw);
            cache.insert(
                comment.attached_to,
                (if capture_raw { raw } else { String::new() }, doc, tags),
            );
        }
    }

    cache
}
