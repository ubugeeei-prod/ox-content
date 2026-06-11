//! Package-manager install tabs transform.
//!
//! Authors write a single npm-style command inside a `<pm>` element, e.g.
//!
//! ```html
//! <pm>npm install -D vite</pm>
//! ```
//!
//! and the post-render transform expands it into the same accessible, no-JS tab
//! widget produced by [`crate::tabs`], with one tab per package manager
//! (npm/pnpm/yarn/bun, in that order). Each tab body is a code block containing
//! the command converted to that package manager's equivalent.
//!
//! The conversion is implemented natively here (no shell-out, no JS). It covers
//! the common install/run/exec verbs; unrecognized commands are passed through
//! unchanged so authoring never silently breaks.
//!
//! Like the generic tabs transform this operates on already-rendered HTML and
//! reuses the exact `ox-tabs` markup so styling and keyboard navigation stay
//! consistent. When `sync` is enabled a `data-ox-tab-group` attribute is emitted
//! on the group element so the client runtime can keep groups in sync via
//! `localStorage`; when disabled the attribute is omitted and the output matches
//! the generic widget byte-for-byte (modulo the converted command bodies).

use crate::html_scan::find_ci;

mod commands;
mod render;
mod scan;
#[cfg(test)]
mod tests;

use render::{extract_command, render_pm};
use scan::{find_matching_close, find_tag, scan_start_tag};

/// The natural group key used for synced package-manager tabs.
pub const PM_GROUP_KEY: &str = "pkg-manager";

/// Options for [`transform_pm`].
#[derive(Debug, Clone, Copy, Default)]
pub struct PmOptions {
    /// When `true`, emit a `data-ox-tab-group` attribute so the client runtime
    /// syncs the active package manager across every pm tab group on the page.
    /// Off by default.
    pub sync: bool,
}

/// Result of [`transform_pm`]: rewritten HTML and the number of `<pm>` groups
/// expanded (so the caller can advance its shared tab-group counter).
pub struct PmTransform {
    pub html: String,
    pub group_count: u32,
}

/// Expand every `<pm>` block in `html`, numbering groups from `start_group`.
pub fn transform_pm(html: &str, start_group: u32, options: PmOptions) -> PmTransform {
    if find_ci(html, 0, "<pm").is_none() {
        return PmTransform { html: html.to_string(), group_count: 0 };
    }

    let mut out = String::with_capacity(html.len());
    let mut cursor = 0;
    let mut next_group = start_group;

    while let Some(open_at) = find_tag(html, cursor, "<pm") {
        out.push_str(&html[cursor..open_at]);

        let Some(tag) = scan_start_tag(html, open_at) else {
            out.push('<');
            cursor = open_at + 1;
            continue;
        };

        if tag.self_closing {
            out.push_str(&html[open_at..tag.end]);
            cursor = tag.end;
            continue;
        }

        let Some(close_start) = find_matching_close(html, tag.end, "<pm", "</pm>") else {
            out.push_str(&html[open_at..tag.end]);
            cursor = tag.end;
            continue;
        };
        let inner = &html[tag.end..close_start];
        let block_end = close_start + "</pm>".len();

        let command = extract_command(inner);
        if command.is_empty() {
            out.push_str(&html[open_at..block_end]);
        } else {
            out.push_str(&render_pm(&command, next_group, options));
            next_group += 1;
        }
        cursor = block_end;
    }

    out.push_str(&html[cursor..]);
    PmTransform { html: out, group_count: next_group - start_group }
}
