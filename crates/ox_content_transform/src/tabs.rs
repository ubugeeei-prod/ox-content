//! Static tabs transform (Rust port of the TS `transformTabs`).
//!
//! Rewrites `<tabs><tab>…</tab></tabs>` blocks in already-rendered HTML into a
//! no-JavaScript, CSS `:has()`-driven tab widget plus a `<details>` `<noscript>`
//! fallback. This replaces a `rehype-parse` + `rehype-stringify` round-trip on
//! the JS side; the Rust renderer's HTML is a rehype fixed-point, so emitting the
//! widget structure and copying each tab's inner HTML verbatim reproduces the
//! previous output byte-for-byte.
//!
//! Group numbering is stateful (each `<tabs>` gets a unique `data-group` used by
//! generated CSS). To keep that state in one place, the caller passes the next
//! group index in and gets back the number of groups consumed — the counter
//! itself stays on the JS side. The exact output is pinned by the
//! `embed-transform` characterization tests in `@ox-content/vite-plugin`.

mod render;
mod scan;
#[cfg(test)]
mod tests;

use crate::html_scan::find_ci;

use render::render_tabs;
use scan::{attribute_value, find_matching_close, find_tag, scan_start_tag};

struct Tab {
    label: String,
    /// Raw inner HTML of the `<tab>` element, copied verbatim.
    content: String,
}

/// Result of [`transform_tabs`]: the rewritten HTML and how many `<tabs>` groups
/// were expanded (groups with no `<tab>` children are left untouched and not
/// counted).
pub struct TabsTransform {
    pub html: String,
    pub group_count: u32,
}

/// Expand every `<tabs>` block in `html`, numbering groups from `start_group`.
pub fn transform_tabs(html: &str, start_group: u32) -> TabsTransform {
    if find_ci(html, 0, "<tabs").is_none() {
        return TabsTransform { html: html.to_string(), group_count: 0 };
    }

    let mut out = String::with_capacity(html.len());
    let mut cursor = 0;
    let mut next_group = start_group;

    while let Some(open_at) = find_tag(html, cursor, "<tabs") {
        // Emit everything up to the `<tabs>`.
        out.push_str(&html[cursor..open_at]);

        let Some(tag) = scan_start_tag(html, open_at) else {
            // Malformed start tag: emit `<` and move on.
            out.push('<');
            cursor = open_at + 1;
            continue;
        };

        if tag.self_closing {
            // `<tabs/>` has no children — leave it as-is.
            out.push_str(&html[open_at..tag.end]);
            cursor = tag.end;
            continue;
        }

        // Find the matching `</tabs>` accounting for nested `<tabs>`.
        let Some(close_start) = find_matching_close(html, tag.end, "<tabs", "</tabs>") else {
            out.push_str(&html[open_at..tag.end]);
            cursor = tag.end;
            continue;
        };
        let inner = &html[tag.end..close_start];
        let block_end = close_start + "</tabs>".len();

        let tabs = parse_tabs(inner);
        if tabs.is_empty() {
            // No `<tab>` children: leave the whole block untouched.
            out.push_str(&html[open_at..block_end]);
        } else {
            out.push_str(&render_tabs(&tabs, next_group));
            next_group += 1;
        }
        cursor = block_end;
    }

    out.push_str(&html[cursor..]);
    TabsTransform { html: out, group_count: next_group - start_group }
}

/// Collect the direct `<tab>` children of a `<tabs>` block's inner HTML.
fn parse_tabs(inner: &str) -> Vec<Tab> {
    // Scan the already-rendered HTML slice and copy panel contents verbatim.
    // This avoids a DOM allocation while preserving the previous rehype output
    // for arbitrary Markdown-rendered HTML inside a tab.
    let mut tabs = Vec::new();
    let mut cursor = 0;
    while let Some(open_at) = find_tag(inner, cursor, "<tab") {
        let Some(tag) = scan_start_tag(inner, open_at) else {
            cursor = open_at + 1;
            continue;
        };
        let label = attribute_value(&inner[tag.name_end..tag.inner_end], "label")
            .filter(|value| !value.is_empty())
            .unwrap_or_else(|| format!("Tab {}", tabs.len() + 1));

        if tag.self_closing {
            tabs.push(Tab { label, content: String::new() });
            cursor = tag.end;
            continue;
        }

        let Some(close_start) = find_matching_close(inner, tag.end, "<tab", "</tab>") else {
            // Unterminated `<tab>`: stop collecting.
            break;
        };
        tabs.push(Tab { label, content: inner[tag.end..close_start].to_string() });
        cursor = close_start + "</tab>".len();
    }
    tabs
}
