//! Highlighting every code block in a rendered document, without leaving Rust.
//!
//! The previous path walked the whole page through an HTML parser and
//! serializer to find `<pre><code>` elements, then parsed each highlighted
//! block back into a tree to splice it in. On the documentation corpus that
//! cost 139 ms for the page round trips and another 38 ms for the per-block
//! re-parses, against 14 ms for the highlighting itself — the plumbing was an
//! order of magnitude more expensive than the work.
//!
//! This finds the blocks with the same scanner the merge step already uses,
//! and splices the result straight back into the original HTML.

use rayon::prelude::*;
use rustc_hash::FxHashMap;

use super::scan::{collect_inline_code, collect_pre_blocks};
use super::text::code_text;
use super::{
    extract_code_block_metadata, merge_highlighted_code_block, merge_highlighted_inline_code,
};

/// Outcome of highlighting a document's code blocks.
pub struct HighlightedDocument {
    /// The document with every handled block replaced.
    pub html: String,
    /// Languages of elements this pass could not read at all, which means the
    /// caller's HTML-parser-based highlighter has to produce the whole page.
    /// Non-empty leaves `html` untouched and `pending` empty.
    pub skipped: Vec<String>,
    /// Well-formed elements whose language has no grammar here, in document
    /// order. The caller highlights each one however it likes and hands the
    /// results back to [`apply_pending_highlights`]; the elements are still
    /// in `html`, unchanged.
    pub pending: Vec<PendingBlock>,
}

/// An element left for the caller's highlighter.
pub struct PendingBlock {
    /// The `language-…` class the element carries.
    pub language: String,
    /// The element's source text, already unescaped.
    pub source: String,
}

/// What kind of element a scanned region holds.
///
/// The two are highlighted from the same source text but spliced back
/// differently: a block keeps its `<pre>` wrapper, an inline element must not
/// grow one.
enum Region {
    Block,
    Inline,
}

/// How much distinct source a page needs before its highlighting is spread
/// across threads.
///
/// Counted in bytes rather than snippets because that is what the work is:
/// a page of thirty one-word member types is a fraction of the parsing of one
/// long code sample. Below this a page's whole highlight pass is shorter than
/// the hand-off costs, which keeps small pages — and the tests — on a plain
/// sequential walk.
pub(super) const PARALLEL_SOURCE_BYTES: usize = 4096;

/// An element this pass has claimed, with the text to highlight it from.
struct Claim {
    start: usize,
    end: usize,
    region: Region,
    language: String,
    source: String,
}

/// Highlights every `class="language-…"` code element in `html`, or none.
///
/// `supports` answers whether a language has a grammar; `highlight` receives
/// an element's source text and language and returns a full replacement
/// `<pre>` element. Attributes, classes and per-line metadata on the original
/// are carried over by the same merges the rehype path used, so annotated
/// blocks and API signatures come out identical.
///
/// An element whose language has no grammar here is *pending* rather than
/// fatal: it is well formed, so the caller can highlight that one element on
/// its own and splice the result back with [`apply_pending_highlights`],
/// without an HTML parser ever seeing the page. Only an element this pass
/// cannot read — markup where a text scan and a real parser would disagree —
/// is `skipped`, and that does surrender the whole document, because the
/// caller's pass will redo every element anyway and highlighting them first
/// would be work thrown away. Claims are collected by a scan that costs no
/// highlighting, so surrendering is cheap.
pub fn highlight_code_blocks(
    html: &str,
    supports: impl Fn(&str) -> bool,
    highlight: impl Fn(&str, &str) -> Option<String> + Sync,
) -> HighlightedDocument {
    let mut regions: Vec<(usize, usize, Region)> = collect_pre_blocks(html)
        .into_iter()
        .map(|(start, end)| (start, end, Region::Block))
        .chain(
            collect_inline_code(html).into_iter().map(|(start, end)| (start, end, Region::Inline)),
        )
        .collect();
    if regions.is_empty() {
        return HighlightedDocument {
            html: html.to_string(),
            skipped: Vec::new(),
            pending: Vec::new(),
        };
    }
    // Both scanners walk the document in order and neither can overlap the
    // other, so ordering by start offset is enough to splice in one pass.
    regions.sort_by_key(|&(start, _, _)| start);

    let mut claims = Vec::with_capacity(regions.len());
    let mut skipped = Vec::new();
    let mut pending = Vec::new();

    for (start, end, region) in regions {
        let element = &html[start..end];
        let Some(language) = element_language(element, &region) else {
            // No language to highlight with. An inline `<code>` without one is
            // ordinary prose markup rather than a snippet we declined, so it
            // is not reported as skipped.
            continue;
        };

        let Some(source) = code_text(element) else {
            skipped.push(language);
            continue;
        };

        if supports(&language) {
            claims.push(Claim { start, end, region, language, source });
        } else {
            pending.push(PendingBlock { language, source });
        }
    }

    if !skipped.is_empty() {
        return HighlightedDocument { html: html.to_string(), skipped, pending: Vec::new() };
    }

    // A page repeats its short snippets heavily — `string` and `boolean` alone
    // account for hundreds of member types across the corpus, and 31% of a
    // page's elements are a source it already holds. Highlighting is keyed by
    // nothing but the text and the language, so each distinct pair is parsed
    // once and the result reused. The merge still runs per element, which is
    // what carries each one's own classes and line metadata.
    let mut work: Vec<(&str, &str)> = Vec::with_capacity(claims.len());
    let mut work_at: FxHashMap<(&str, &str), usize> = FxHashMap::default();
    let mut slots = Vec::with_capacity(claims.len());

    for claim in &claims {
        let key = (claim.language.as_str(), claim.source.as_str());
        let at = *work_at.entry(key).or_insert_with(|| {
            work.push(key);
            work.len() - 1
        });
        slots.push(at);
    }

    // Each snippet is parsed and walked on its own — nothing is shared between
    // them but the grammar tables, which are immutable once built — so the
    // distinct list is pure independent work. Below the threshold a page does
    // not have enough of it to pay for handing work to other threads.
    let bytes: usize = work.iter().map(|(_, source)| source.len()).sum();
    let rendered: Vec<Option<String>> = if bytes >= PARALLEL_SOURCE_BYTES {
        work.par_iter().map(|&(language, source)| highlight(source, language)).collect()
    } else {
        work.iter().map(|&(language, source)| highlight(source, language)).collect()
    };

    // `supports` promised every claim, so a failure here is the highlighter
    // contradicting itself rather than an expected fallback; hand back the
    // original so the caller's own pass produces the page.
    if let Some(at) = rendered.iter().position(Option::is_none) {
        return HighlightedDocument {
            html: html.to_string(),
            skipped: vec![work[at].0.to_string()],
            pending: Vec::new(),
        };
    }
    let rendered: Vec<String> = rendered.into_iter().flatten().collect();

    let mut out = String::with_capacity(html.len() * 2);
    let mut cursor = 0;

    for (claim, at) in claims.iter().zip(slots) {
        let element = &html[claim.start..claim.end];
        let highlighted = &rendered[at];
        let merged = match claim.region {
            Region::Block => {
                Some(merge_highlighted_code_block(element, highlighted, Some(&claim.language)))
            }
            Region::Inline => merge_highlighted_inline_code(element, highlighted),
        };
        let Some(merged) = merged else {
            skipped.push(claim.language.clone());
            continue;
        };

        out.push_str(&html[cursor..claim.start]);
        out.push_str(&merged);
        cursor = claim.end;
    }

    // A merge only fails on markup the claim scan said was well formed, so
    // this is the module contradicting itself rather than an expected
    // fallback; hand back the original and let the caller's pass produce the
    // page.
    if !skipped.is_empty() {
        return HighlightedDocument { html: html.to_string(), skipped, pending: Vec::new() };
    }

    out.push_str(&html[cursor..]);
    HighlightedDocument { html: out, skipped, pending }
}

/// Splices the caller's highlighting back over the elements
/// [`highlight_code_blocks`] left pending.
///
/// `replacements` lines up with the `pending` list it returned: entry `i` is a
/// full `<pre>` element for pending block `i`, or an empty string to leave
/// that one as it is. `supports` must answer as it did then, since it is what
/// tells the two scans apart — an element it claims was already highlighted
/// and is not touched again.
pub fn apply_pending_highlights(
    html: &str,
    replacements: &[String],
    supports: impl Fn(&str) -> bool,
) -> String {
    if replacements.is_empty() {
        return html.to_string();
    }

    let mut regions: Vec<(usize, usize, Region)> = collect_pre_blocks(html)
        .into_iter()
        .map(|(start, end)| (start, end, Region::Block))
        .chain(
            collect_inline_code(html).into_iter().map(|(start, end)| (start, end, Region::Inline)),
        )
        .collect();
    regions.sort_by_key(|&(start, _, _)| start);

    let mut out = String::with_capacity(html.len() * 2);
    let mut cursor = 0;
    let mut next = 0;

    for (start, end, region) in regions {
        if next >= replacements.len() {
            break;
        }
        let element = &html[start..end];
        let Some(language) = element_language(element, &region) else {
            continue;
        };
        if supports(&language) || code_text(element).is_none() {
            continue;
        }

        let replacement = replacements[next].as_str();
        next += 1;

        // An empty replacement means the caller's highlighter had no grammar
        // for this block either. A block still picks up the `data-language`
        // the merge writes — the tree walk reached the same state by leaving
        // the element in place and merging the original metadata back over it
        // — while an inline element is left exactly as it arrived, because
        // that merge only ever ran over `<pre>` blocks.
        let merged = match (&region, replacement.is_empty()) {
            (Region::Block, true) => {
                Some(merge_highlighted_code_block(element, element, Some(&language)))
            }
            (Region::Block, false) => {
                Some(merge_highlighted_code_block(element, replacement, Some(&language)))
            }
            (Region::Inline, true) => None,
            (Region::Inline, false) => merge_highlighted_inline_code(element, replacement),
        };
        let Some(merged) = merged else {
            continue;
        };

        out.push_str(&html[cursor..start]);
        out.push_str(&merged);
        cursor = end;
    }

    out.push_str(&html[cursor..]);
    out
}

/// The language to highlight the element as, if it should be highlighted.
///
/// A block without a `language-` class still gets highlighted, as plain text —
/// that is what a bare ``` fence produces, and the rehype pass rendered those
/// with the same wrapper markup as any other block. An inline `<code>` without
/// one is ordinary prose markup instead, and is left alone.
fn element_language(element: &str, region: &Region) -> Option<String> {
    match region {
        Region::Block => Some(
            extract_code_block_metadata(element).language.unwrap_or_else(|| "text".to_string()),
        ),
        Region::Inline => super::find_next_start_tag(element, 0)?
            .tag
            .class_names()
            .iter()
            .find_map(|class_name| class_name.strip_prefix("language-"))
            .map(ToString::to_string),
    }
}
