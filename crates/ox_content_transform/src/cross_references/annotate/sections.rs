//! Numbering `<h1>`–`<h6>` hierarchically.

use super::super::html::{
    append_data_attrs, escape_url_fragment, is_word_byte, read_attr, should_track_target,
    text_content,
};
use super::super::types::{CrossReferenceEntry, CrossReferenceKind, CrossReferencesOptions};
use super::{Registry, TrackedTarget};
use crate::html_scan::find_ci;
use std::fmt::Write as _;

/// Numbers `<h1>`–`<h6>` hierarchically: `1`, `1.2`, `1.2.1`.
pub fn annotate_sections(
    html: &str,
    options: &CrossReferencesOptions,
    registry: &mut Registry,
) -> String {
    let mut counters = [0usize; 6];
    let mut out = String::with_capacity(html.len());
    let mut cursor = 0usize;

    while let Some(open) = find_heading(html, cursor) {
        let depth = open.depth;
        counters[depth - 1] += 1;
        // A deeper level restarts once its parent moves on.
        counters[depth..].fill(0);

        out.push_str(&html[cursor..open.start]);
        let attrs = &html[open.attrs_start..open.attrs_end];
        let body = &html[open.body_start..open.body_end];

        match read_attr(attrs, "id").filter(|id| should_track_target(id)) {
            None => out.push_str(&html[open.start..open.end]),
            Some(id) => {
                let number = section_number(&counters, depth);
                let text = format!("{} {}", options.labels.section, number);
                registry.register(
                    options.duplicates,
                    TrackedTarget {
                        entry: CrossReferenceEntry {
                            href: format!("#{}", escape_url_fragment(&id)),
                            id,
                            kind: CrossReferenceKind::Section,
                            number: number.clone(),
                            label: options.labels.section.clone(),
                            text: text.clone(),
                            title: Some(text_content(body)),
                        },
                        position: open.start,
                    },
                );
                let _ = write!(
                    out,
                    "<h{depth}{}>{body}</h{depth}>",
                    append_data_attrs(attrs, CrossReferenceKind::Section, &number, &text)
                );
            }
        }
        cursor = open.end;
    }
    out.push_str(&html[cursor..]);
    out
}

/// `1.2.1`, skipping levels that were never opened.
///
/// The empty-string case is a heading whose own counter is zero, which cannot
/// happen here — it is incremented immediately above — but the original fell
/// back to the raw counter, so this keeps that shape.
fn section_number(counters: &[usize; 6], depth: usize) -> String {
    let parts: Vec<String> =
        counters[..depth].iter().filter(|value| **value != 0).map(usize::to_string).collect();
    if parts.is_empty() { counters[depth - 1].to_string() } else { parts.join(".") }
}

struct Heading {
    depth: usize,
    start: usize,
    end: usize,
    attrs_start: usize,
    attrs_end: usize,
    body_start: usize,
    body_end: usize,
}

/// `<hN …>…</hN>` with matching N, non-greedy body.
fn find_heading(html: &str, from: usize) -> Option<Heading> {
    let bytes = html.as_bytes();
    let mut cursor = from;

    while let Some(open) = memchr::memchr(b'<', &bytes[cursor..]).map(|rel| cursor + rel) {
        cursor = open + 1;
        let Some(marker) = bytes.get(open + 1) else { break };
        if !marker.eq_ignore_ascii_case(&b'h') {
            continue;
        }
        let Some(digit) = bytes.get(open + 2).copied() else { continue };
        if !(b'1'..=b'6').contains(&digit) {
            continue;
        }
        // `\b` after the digit.
        if bytes.get(open + 3).is_some_and(|byte| is_word_byte(*byte)) {
            continue;
        }
        let Some(attrs_end) = html[open + 3..].find('>').map(|rel| open + 3 + rel) else {
            continue;
        };
        let close = format!("</h{}>", digit as char);
        let Some(close_at) = find_ci(html, attrs_end + 1, &close) else {
            continue;
        };
        return Some(Heading {
            depth: usize::from(digit - b'0'),
            start: open,
            end: close_at + close.len(),
            attrs_start: open + 3,
            attrs_end,
            body_start: attrs_end + 1,
            body_end: close_at,
        });
    }
    None
}
