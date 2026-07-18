//! Low-level output helpers for the HTML renderer.
//!
//! These methods sit between visitor code and raw string buffers. They centralize
//! escaping, autolinking, heading-id emission, and raw HTML handling so the visitor
//! methods can describe document structure without duplicating output mechanics.

use std::fmt::{Display, Write as _};

use ox_content_ast::{Heading, Node};

use super::super::autolink::find_autolink_match;
use super::super::escape::{write_escaped_into, write_url_escaped_into};
use super::super::heading::{collect_heading_text_into, slugify_heading_into};
use super::HtmlRenderer;

impl HtmlRenderer {
    pub(in crate::html::renderer) fn write(&mut self, s: &str) {
        self.output.push_str(s);
    }

    pub(in crate::html::renderer) fn write_display(&mut self, value: impl Display) {
        write!(self.output, "{value}").expect("writing to String should not fail");
    }

    pub(in crate::html::renderer) fn write_escaped(&mut self, s: &str) {
        crate::profile_span!("renderer::write_escaped");
        write_escaped_into(&mut self.output, s);
    }

    /// Walks `s` and emits an `<a>` tag for each registered URL pattern match.
    ///
    /// The caller has already gated on the autolink option and link nesting
    /// state, so this routine can focus on the hot loop: use the per-render
    /// first-byte index to jump to possible URL starts, escape the intervening
    /// non-URL text in chunks, then write the matched URL once for `href` and
    /// once for visible text. If the index is absent, we fail open by emitting
    /// escaped text rather than rebuilding the index here.
    pub(in crate::html::renderer) fn write_text_with_autolinks(&mut self, s: &str) {
        crate::profile_span!("renderer::write_text_with_autolinks");
        let bytes = s.as_bytes();
        // Reuse the per-render first-byte index (see `autolink_index`). If it's
        // absent the caller's gating slipped — fall back to emitting the text
        // verbatim rather than rebuilding the index here.
        let Some(index) = self.autolink_index.as_ref() else {
            write_escaped_into(&mut self.output, s);
            return;
        };
        // Borrow the relevant fields disjointly so the URL scan (which only
        // reads `options`/`autolink_index`) and the output writes can coexist.
        let patterns = &self.options.autolink_patterns;
        let target_blank = self.options.autolink_target_blank;
        let out = &mut self.output;
        let mut cursor = 0usize;
        while cursor < bytes.len() {
            let Some((match_start, url_end)) = find_autolink_match(s, cursor, patterns, index)
            else {
                break;
            };
            // Emit the literal text preceding the URL.
            if match_start > cursor {
                write_escaped_into(out, &s[cursor..match_start]);
            }
            let url = &s[match_start..url_end];
            out.push_str("<a href=\"");
            write_url_escaped_into(out, url);
            out.push('"');
            if target_blank {
                out.push_str(" target=\"_blank\" rel=\"noopener noreferrer\"");
            }
            out.push('>');
            // The visible text is the URL itself; escape it like any text.
            write_escaped_into(out, url);
            out.push_str("</a>");
            cursor = url_end;
        }
        if cursor < bytes.len() {
            write_escaped_into(out, &s[cursor..]);
        }
    }

    pub(in crate::html::renderer) fn write_url_escaped(&mut self, s: &str) {
        write_url_escaped_into(&mut self.output, s);
    }

    pub(in crate::html::renderer) fn sanitized_url<'a>(
        &self,
        url: &'a str,
        fallback: &'static str,
    ) -> &'a str {
        if !self.options.sanitize {
            return url;
        }

        let trimmed =
            url.trim_matches(|ch: char| ch.is_ascii_control() || ch.is_ascii_whitespace());

        if Self::is_safe_url(trimmed) {
            trimmed
        } else {
            fallback
        }
    }

    pub(in crate::html::renderer) fn is_safe_url(url: &str) -> bool {
        if url.bytes().any(|byte| byte.is_ascii_control()) {
            return false;
        }

        let Some(colon_index) = url.find(':') else {
            return true;
        };

        let first_path_marker = url.find(&['/', '?', '#'][..]).unwrap_or(usize::MAX);
        if first_path_marker < colon_index {
            return true;
        }

        let scheme = url[..colon_index]
            .chars()
            .filter(|ch| !ch.is_ascii_whitespace())
            .map(|ch| ch.to_ascii_lowercase())
            .collect::<String>();

        matches!(scheme.as_str(), "http" | "https" | "mailto" | "tel")
    }

    pub(in crate::html::renderer) fn write_html_value(&mut self, value: &str) {
        if self.options.sanitize {
            self.write_escaped(value);
            return;
        }

        // URL rewriting produces an owned string; the tag filter then runs
        // over whichever form we ended up with so both paths get filtered.
        let rewritten = if self.options.convert_md_links {
            Some(self.rewrite_html_root_urls(value))
        } else {
            None
        };
        let value = rewritten.as_deref().unwrap_or(value);

        if self.options.disallow_raw_html && crate::html::tagfilter::needs_filtering(value) {
            crate::html::tagfilter::write_filtered_into(&mut self.output, value);
        } else {
            self.write(value);
        }
    }

    pub(in crate::html::renderer) fn visit_inline_node(&mut self, node: &Node<'_>) {
        // Text is the overwhelmingly common child of paragraphs / headings
        // / links / emphasis / strong, etc. — on the bundled corpora it
        // accounts for roughly 60-70% of inline visits. Inlining the
        // write here skips the trait's 20-arm `walk_node` match and the
        // `visit_text` wrapper, both of which are the only thing
        // `visit_text` would do anyway (escape into `self.output`).
        match node {
            Node::Text(text) => {
                // The autolink builtin lives on this hot path too: when
                // the flag is on (and we're not already inside an `<a>`)
                // we have to scan the text for URLs before escaping. The
                // common case — flag off — collapses back to the original
                // single `write_escaped_into` call thanks to the early
                // boolean check.
                // `autolink_index` is `Some` iff `autolink_urls` and a non-empty
                // pattern list (computed once at `render()` entry), so this one
                // Option check replaces the three field reads.
                if self.autolink_index.is_some() && !self.in_link {
                    self.write_text_with_autolinks(text.value);
                } else {
                    write_escaped_into(&mut self.output, text.value);
                }
            }
            Node::Html(html) => self.write_html_value(html.value),
            _ => self.render_node(node),
        }
    }

    /// Writes the heading's slugified id directly into `self.output`.
    ///
    /// This avoids allocating a return `String`. The unique-heading path pays
    /// for exactly one owned string, the slug clone that becomes the map key.
    /// The duplicate-heading path pays for zero additional strings because the
    /// existing scratch slug is written directly and the numeric suffix is
    /// formatted into `self.output`.
    pub(in crate::html::renderer) fn write_heading_id(&mut self, heading: &Heading<'_>) {
        crate::profile_span!("renderer::write_heading_id");
        use std::fmt::Write as _;

        self.heading_text_scratch.clear();
        collect_heading_text_into(&heading.children, &mut self.heading_text_scratch);
        self.heading_slug_scratch.clear();
        slugify_heading_into(&self.heading_text_scratch, &mut self.heading_slug_scratch);

        // Cheap lookup first — avoids cloning the slug on the duplicate
        // path. The `entry()` API would force us to materialize an owned
        // key up front, defeating the point.
        if let Some(count) = self.heading_id_counts.get_mut(self.heading_slug_scratch.as_str()) {
            let n = *count;
            *count += 1;
            self.output.push_str(&self.heading_slug_scratch);
            // `write!` into `String` is infallible; the formatter pushes bytes
            // directly into the existing buffer with no temporary allocation.
            let _ = write!(self.output, "-{n}");
            return;
        }

        self.output.push_str(&self.heading_slug_scratch);
        let key = self.heading_slug_scratch.clone();
        self.heading_id_counts.insert(key, 1);
    }
}
