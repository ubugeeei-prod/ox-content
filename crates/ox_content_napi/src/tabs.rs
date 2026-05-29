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

    while let Some(open_at) = find_tag(html, cursor, "tabs") {
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
        let Some(close_start) = find_matching_close(html, tag.end, "tabs", "</tabs>") else {
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
    let mut tabs = Vec::new();
    let mut cursor = 0;
    while let Some(open_at) = find_tag(inner, cursor, "tab") {
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

        let Some(close_start) = find_matching_close(inner, tag.end, "tab", "</tab>") else {
            // Unterminated `<tab>`: stop collecting.
            break;
        };
        tabs.push(Tab { label, content: inner[tag.end..close_start].to_string() });
        cursor = close_start + "</tab>".len();
    }
    tabs
}

fn render_tabs(tabs: &[Tab], group: u32) -> String {
    let mut out = String::new();
    out.push_str("<div class=\"ox-tabs-container\"><div class=\"ox-tabs\" data-group=\"");
    out.push_str(&group.to_string());
    out.push_str("\"><div class=\"ox-tabs-header\">");
    for (index, tab) in tabs.iter().enumerate() {
        out.push_str("<input type=\"radio\" name=\"ox-tabs-");
        out.push_str(&group.to_string());
        out.push_str("\" id=\"ox-tab-");
        out.push_str(&group.to_string());
        out.push('-');
        out.push_str(&index.to_string());
        out.push('"');
        if index == 0 {
            out.push_str(" checked");
        }
        out.push_str("><label for=\"ox-tab-");
        out.push_str(&group.to_string());
        out.push('-');
        out.push_str(&index.to_string());
        out.push_str("\">");
        out.push_str(&escape_text(&tab.label));
        out.push_str("</label>");
    }
    out.push_str("</div>");
    for (index, tab) in tabs.iter().enumerate() {
        out.push_str("<div class=\"ox-tab-panel\" data-tab=\"");
        out.push_str(&index.to_string());
        out.push_str("\">");
        out.push_str(&tab.content);
        out.push_str("</div>");
    }
    out.push_str("</div><noscript><div class=\"ox-tabs-fallback\">");
    for (index, tab) in tabs.iter().enumerate() {
        out.push_str("<details");
        if index == 0 {
            out.push_str(" open");
        }
        out.push_str("><summary>");
        out.push_str(&escape_text(&tab.label));
        out.push_str("</summary><div class=\"ox-tabs-fallback-content\">");
        out.push_str(&tab.content);
        out.push_str("</div></details>");
    }
    out.push_str("</div></noscript></div>");
    out
}

/// A scanned start tag.
struct StartTag {
    /// Byte index just past the element name (start of the attribute region).
    name_end: usize,
    /// Byte index of the attribute region end (before `/` of `/>` or before `>`).
    inner_end: usize,
    /// Byte index just past the closing `>`.
    end: usize,
    self_closing: bool,
}

/// Scan the start tag beginning at `pos` (which must point at `<`), respecting
/// quoted attribute values so a `>` inside a value doesn't end the tag early.
fn scan_start_tag(html: &str, pos: usize) -> Option<StartTag> {
    let bytes = html.as_bytes();
    if bytes.get(pos) != Some(&b'<') {
        return None;
    }
    let mut i = pos + 1;
    while i < bytes.len() && !bytes[i].is_ascii_whitespace() && bytes[i] != b'>' && bytes[i] != b'/'
    {
        i += 1;
    }
    let name_end = i;
    let mut quote: Option<u8> = None;
    let mut tag_end = None;
    while i < bytes.len() {
        let b = bytes[i];
        match quote {
            Some(q) => {
                if b == q {
                    quote = None;
                }
            }
            None => {
                if b == b'"' || b == b'\'' {
                    quote = Some(b);
                } else if b == b'>' {
                    tag_end = Some(i);
                    break;
                }
            }
        }
        i += 1;
    }
    let tag_end = tag_end?;
    let self_closing = tag_end > pos && bytes[tag_end - 1] == b'/';
    let inner_end = if self_closing { tag_end - 1 } else { tag_end };
    Some(StartTag { name_end, inner_end, end: tag_end + 1, self_closing })
}

/// Find the next `<name` start tag at or after `from` with a proper element
/// boundary, so `<tab` never matches `<tabs`/`<table` and `<tabs` never matches
/// `<tabset>`.
fn find_tag(html: &str, from: usize, name: &str) -> Option<usize> {
    let needle = format!("<{name}");
    let bytes = html.as_bytes();
    let mut search = from;
    loop {
        let at = find_ci(html, search, &needle)?;
        let after = at + needle.len();
        let boundary = bytes.get(after).copied();
        if matches!(boundary, Some(b) if b == b'>' || b == b'/' || b.is_ascii_whitespace()) {
            return Some(at);
        }
        search = after;
    }
}

/// Given the position just past a `<name …>` start tag, find the byte offset of
/// its matching `close` literal, accounting for nested `<name …>` opens.
fn find_matching_close(html: &str, from: usize, name: &str, close: &str) -> Option<usize> {
    let mut depth = 1usize;
    let mut search = from;
    loop {
        let next_open = find_tag(html, search, name);
        let next_close = find_ci(html, search, close);
        match (next_open, next_close) {
            (Some(open_at), Some(close_at)) if open_at < close_at => {
                depth += 1;
                // Advance past this nested open tag's `>`.
                search = match scan_start_tag(html, open_at) {
                    Some(tag) => tag.end,
                    None => open_at + 1,
                };
            }
            (_, Some(close_at)) => {
                depth -= 1;
                if depth == 0 {
                    return Some(close_at);
                }
                search = close_at + close.len();
            }
            (_, None) => return None,
        }
    }
}

/// Read the value of `name` (case-insensitive) from a start tag's attribute
/// region. Supports quoted and unquoted values; missing value yields `""`.
fn attribute_value(attrs: &str, name: &str) -> Option<String> {
    let bytes = attrs.as_bytes();
    let mut i = 0;
    while i < bytes.len() {
        while i < bytes.len() && bytes[i].is_ascii_whitespace() {
            i += 1;
        }
        if i >= bytes.len() || bytes[i] == b'/' {
            break;
        }
        let name_start = i;
        while i < bytes.len()
            && !bytes[i].is_ascii_whitespace()
            && bytes[i] != b'='
            && bytes[i] != b'/'
        {
            i += 1;
        }
        let attr_name = &attrs[name_start..i];
        while i < bytes.len() && bytes[i].is_ascii_whitespace() {
            i += 1;
        }
        let value = if i < bytes.len() && bytes[i] == b'=' {
            i += 1;
            while i < bytes.len() && bytes[i].is_ascii_whitespace() {
                i += 1;
            }
            if i >= bytes.len() {
                String::new()
            } else if bytes[i] == b'"' || bytes[i] == b'\'' {
                let q = bytes[i];
                i += 1;
                let vs = i;
                while i < bytes.len() && bytes[i] != q {
                    i += 1;
                }
                let v = attrs[vs..i].to_string();
                if i < bytes.len() {
                    i += 1;
                }
                v
            } else {
                let vs = i;
                while i < bytes.len() && !bytes[i].is_ascii_whitespace() && bytes[i] != b'/' {
                    i += 1;
                }
                attrs[vs..i].to_string()
            }
        } else {
            String::new()
        };
        if attr_name.eq_ignore_ascii_case(name) {
            return Some(value);
        }
    }
    None
}

/// Escape text content the way `hast-util-to-html` does for the characters that
/// can occur in tab labels.
fn escape_text(value: &str) -> String {
    let mut out = String::with_capacity(value.len());
    for ch in value.chars() {
        match ch {
            '&' => out.push_str("&#x26;"),
            '<' => out.push_str("&#x3C;"),
            '>' => out.push_str("&#x3E;"),
            _ => out.push(ch),
        }
    }
    out
}

/// Case-insensitive ASCII substring search in `haystack[from..]`.
fn find_ci(haystack: &str, from: usize, needle: &str) -> Option<usize> {
    let hay = haystack.as_bytes();
    let pat = needle.as_bytes();
    if pat.is_empty() || from > hay.len() || hay.len() - from < pat.len() {
        return None;
    }
    let last = hay.len() - pat.len();
    for i in from..=last {
        if hay[i..i + pat.len()].eq_ignore_ascii_case(pat) {
            return Some(i);
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn expands_two_tabs_matching_characterization() {
        let result = transform_tabs(
            r#"<tabs><tab title="A">alpha</tab><tab title="B">beta</tab></tabs>"#,
            0,
        );
        assert_eq!(result.group_count, 1);
        assert_eq!(
            result.html,
            r#"<div class="ox-tabs-container"><div class="ox-tabs" data-group="0"><div class="ox-tabs-header"><input type="radio" name="ox-tabs-0" id="ox-tab-0-0" checked><label for="ox-tab-0-0">Tab 1</label><input type="radio" name="ox-tabs-0" id="ox-tab-0-1"><label for="ox-tab-0-1">Tab 2</label></div><div class="ox-tab-panel" data-tab="0">alpha</div><div class="ox-tab-panel" data-tab="1">beta</div></div><noscript><div class="ox-tabs-fallback"><details open><summary>Tab 1</summary><div class="ox-tabs-fallback-content">alpha</div></details><details><summary>Tab 2</summary><div class="ox-tabs-fallback-content">beta</div></details></div></noscript></div>"#
        );
    }

    #[test]
    fn uses_label_attribute_when_present() {
        let result = transform_tabs(r#"<tabs><tab label="First">x</tab></tabs>"#, 0);
        assert!(result.html.contains("<label for=\"ox-tab-0-0\">First</label>"));
        assert!(result.html.contains("<summary>First</summary>"));
    }

    #[test]
    fn numbers_groups_from_start_and_counts() {
        let result =
            transform_tabs(r#"<tabs><tab>a</tab></tabs> middle <tabs><tab>b</tab></tabs>"#, 5);
        assert_eq!(result.group_count, 2);
        assert!(result.html.contains(r#"data-group="5""#));
        assert!(result.html.contains(r#"data-group="6""#));
        assert!(result.html.contains(" middle "));
    }

    #[test]
    fn leaves_empty_tabs_untouched_and_uncounted() {
        let html = r#"<tabs>   </tabs>"#;
        let result = transform_tabs(html, 0);
        assert_eq!(result.group_count, 0);
        assert_eq!(result.html, html);
    }

    #[test]
    fn passes_through_without_tabs_marker() {
        let html = r#"<p>No tabs here, just a <table><tr><td>cell</td></tr></table>.</p>"#;
        let result = transform_tabs(html, 0);
        assert_eq!(result.group_count, 0);
        assert_eq!(result.html, html);
    }

    #[test]
    fn preserves_rich_inner_content() {
        let result =
            transform_tabs(r#"<tabs><tab label="Code"><pre><code>x</code></pre></tab></tabs>"#, 0);
        assert!(result
            .html
            .contains(r#"<div class="ox-tab-panel" data-tab="0"><pre><code>x</code></pre></div>"#));
    }
}
