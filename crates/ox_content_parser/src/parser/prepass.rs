//! Fused document pre-pass: link reference definitions + footnote labels.
//!
//! Both collectors used to run their own full line scan over the source.
//! This module fuses them into a single scan and dispatches on each line's
//! first byte, so the common case (prose, HTML, or code content that cannot
//! affect either collector) costs one memchr line skip and a flag write
//! instead of quote-stripping, trimming, and fence-classifying every line.
//!
//! Behavior is intentionally identical to the two previous passes,
//! including their asymmetry: the reference collector classifies fences on
//! the quote-stripped line (so a quoted fence line opens a fence for it)
//! while the footnote collector classifies fences on the raw line (so the
//! same line does not). The two fence states are tracked independently,
//! and the footnote scan still visits every line of a multi-line reference
//! definition chunk the reference side skips over.

use std::rc::Rc;
use std::sync::LazyLock;

use memchr::{memmem, memrchr};

use super::footnote::{normalize_footnote_label, parse_footnote_opener, FootnoteLabels};
use super::line_scan::{line_end as scan_line_end, next_line_start};
use super::reference::{
    closes_paragraph_context, fence_open, is_fence_close, strip_quote_markers, ReferenceDef,
    ReferenceMap,
};
use super::Parser;
#[allow(unused_imports)]
use crate::{profile_span, profile_span_detail};

/// Three-byte fence-run searchers, built once for the process.
///
/// A closing fence line holds at least three of its fence byte in a row, so
/// the needle is a necessary condition for one — enough to skip straight to
/// the first line that could possibly close an open fence. The one-shot
/// `memmem::find` would rebuild its SIMD prefilter for every fence in the
/// document.
static BACKTICK_RUN: LazyLock<memmem::Finder<'static>> =
    LazyLock::new(|| memmem::Finder::new("```"));
static TILDE_RUN: LazyLock<memmem::Finder<'static>> = LazyLock::new(|| memmem::Finder::new("~~~"));

/// Start of the first line at or after `from` holding a run of three
/// `fence_byte`s, or `None` when the rest of the source holds none.
fn next_fence_run_line(bytes: &[u8], from: usize, fence_byte: u8) -> Option<usize> {
    // `from` is one past a newline, which is one past the end when the last
    // line of the document is unterminated.
    let from = from.min(bytes.len());
    let finder = if fence_byte == b'`' { &*BACKTICK_RUN } else { &*TILDE_RUN };
    let at = from + finder.find(&bytes[from..])?;
    // `from` is a line start, so the match's line starts at or after it.
    Some(memrchr(b'\n', &bytes[from..at]).map_or(from, |off| from + off + 1))
}

impl<'a> Parser<'a> {
    /// Runs the fused pre-pass for a root parser. Returns the
    /// document-wide reference map and footnote label set that are shared
    /// with sub-parsers.
    ///
    /// Either collection comes back as `None` when it stayed empty, so the
    /// common document — no link reference definitions, no footnotes — never
    /// allocates an `Rc` for a map nothing will read.
    pub(super) fn build_prepass(
        &self,
    ) -> (Option<Rc<ReferenceMap<'a>>>, Option<Rc<FootnoteLabels>>) {
        profile_span!("parser::build_prepass");
        // Cheap bails. Both collectors need a `[` somewhere, and both a
        // literal `]:`: a reference definition's label must be closed by
        // `]` immediately followed by `:` (`[label]:`), and a footnote
        // opener requires the same (`[^label]:`). A `]` and `:` split
        // across a line break never parses as a definition, so absence of
        // the contiguous needle proves the scan would collect nothing.
        // Typical link-only documents skip the whole line scan here.
        if !self.source.contains('[')
            || memchr::memmem::find(self.source.as_bytes(), b"]:").is_none()
        {
            return (None, None);
        }
        let collect_footnotes = self.options.footnotes && self.source.contains("[^");

        let mut definitions = ReferenceMap::default();
        let mut labels = FootnoteLabels::default();
        let bytes = self.source.as_bytes();
        let mut pos = 0;
        let mut def_fence: Option<(u8, usize)> = None;
        let mut foot_fence: Option<(u8, usize)> = None;
        let mut paragraph_open = false;

        while pos < bytes.len() {
            profile_span_detail!("parser::prepass_line");
            let first = bytes[pos];

            // Blank line: closes any open paragraph and is invisible to
            // both fence trackers and both collectors.
            if first == b'\n' {
                if def_fence.is_none() {
                    paragraph_open = false;
                }
                pos += 1;
                continue;
            }

            // Fast lane: a line starting with any byte outside this set
            // cannot open or close a fence, start a definition or footnote
            // label, or close a paragraph. It only keeps (or opens)
            // paragraph context while the reference collector is outside a
            // fence.
            if !matches!(
                first,
                b'[' | b' ' | b'\t' | b'>' | b'`' | b'~' | b'-' | b'=' | b'*' | b'#'
            ) {
                if def_fence.is_none() {
                    paragraph_open = true;
                }
                pos = next_line_start(bytes, pos);
                continue;
            }

            // ATX-heading-shaped line: closes paragraph context outside a
            // fence and is invisible to fences and both collectors.
            if first == b'#' {
                if def_fence.is_none() {
                    paragraph_open = false;
                }
                pos = next_line_start(bytes, pos);
                continue;
            }

            let line_end = scan_line_end(bytes, pos);
            let line = &self.source[pos..line_end];

            // Footnote side first: the reference side `continue`s out of
            // the loop body on its fence transitions.
            if collect_footnotes {
                footnote_scan_line(line, first, &mut foot_fence, &mut labels);
            }

            let stripped = strip_quote_markers(line);
            let trimmed = stripped.trim_start_matches([' ', '\t']);

            if let Some((fence_byte, fence_len)) = def_fence {
                if is_fence_close(trimmed, fence_byte, fence_len) {
                    def_fence = None;
                    pos = line_end + 1;
                    continue;
                }
                // Nothing inside a fence is collected and `paragraph_open`
                // is frozen while one is open, so the only line left worth
                // stopping on is the one that closes it — and that needs
                // three fence bytes in a row. Skip straight to it instead of
                // walking the block's contents line by line.
                //
                // The footnote collector is the exception: it tracks its own
                // fence on the raw line and looks for `[^` openers, so when
                // it is running every line still has to be visited. Only
                // 1-2 files in 100 across the bundled corpora contain a
                // `[^` at all, so the skip still applies to almost every
                // real document.
                if collect_footnotes {
                    pos = line_end + 1;
                    continue;
                }
                match next_fence_run_line(bytes, line_end + 1, fence_byte) {
                    Some(next) => pos = next,
                    // An unterminated fence swallows the rest of the
                    // document, so there is nothing left to collect.
                    None => break,
                }
                continue;
            }
            if let Some(open) = fence_open(trimmed) {
                def_fence = Some(open);
                paragraph_open = false;
                pos = line_end + 1;
                continue;
            }
            if trimmed.is_empty() {
                paragraph_open = false;
                pos = line_end + 1;
                continue;
            }

            if !paragraph_open && stripped.len() - trimmed.len() <= 3 && trimmed.starts_with('[') {
                // Candidate: join the stripped lines of this paragraph
                // chunk and parse as many definitions as it holds.
                let (chunk, line_starts) = self.join_stripped_chunk(pos);
                let mut offset = 0;
                while let Some(parsed) = self.parse_reference_definition(&chunk[offset..]) {
                    definitions
                        .entry(Self::normalize_reference_label(parsed.label))
                        .or_insert(ReferenceDef { url: parsed.url, title: parsed.title });
                    offset += parsed.consumed;
                }
                // Skip the source lines the parsed prefix covered so fence
                // tracking stays aligned (definition text can't open
                // fences). A leftover suffix starts a paragraph.
                let consumed_lines = chunk[..offset].matches('\n').count();
                if consumed_lines > 0 {
                    let next_pos = line_starts.get(consumed_lines).copied().unwrap_or(bytes.len());
                    // The footnote collector's scan is line-independent, so
                    // it must still see the definition's continuation lines
                    // the reference side jumps over.
                    if collect_footnotes {
                        let mut foot_pos = line_end + 1;
                        while foot_pos < next_pos {
                            let foot_line_end = scan_line_end(bytes, foot_pos);
                            footnote_scan_line(
                                &self.source[foot_pos..foot_line_end],
                                bytes[foot_pos],
                                &mut foot_fence,
                                &mut labels,
                            );
                            foot_pos = foot_line_end + 1;
                        }
                    }
                    pos = next_pos;
                    continue;
                }
            }

            paragraph_open = !closes_paragraph_context(trimmed);
            pos = line_end + 1;
        }

        (
            (!definitions.is_empty()).then(|| Rc::new(definitions)),
            (!labels.is_empty()).then(|| Rc::new(labels)),
        )
    }
}

/// One line of the footnote-label scan: raw-line fence tracking plus the
/// `[^label]:` opener check. Mirrors the former standalone footnote
/// pre-pass exactly (no quote stripping).
fn footnote_scan_line(
    line: &str,
    first: u8,
    foot_fence: &mut Option<(u8, usize)>,
    labels: &mut FootnoteLabels,
) {
    let raw_trimmed = line.trim_start_matches([' ', '\t']);
    if let Some((fence_byte, fence_len)) = *foot_fence {
        if is_fence_close(raw_trimmed, fence_byte, fence_len) {
            *foot_fence = None;
        }
    } else if let Some(open) = fence_open(raw_trimmed) {
        *foot_fence = Some(open);
    } else if matches!(first, b'[' | b' ') {
        // An opener starts with `[^` after at most three spaces, so only
        // these first bytes can begin one.
        if let Some((label, _)) = parse_footnote_opener(line) {
            labels.insert(normalize_footnote_label(label));
        }
    }
}
