use std::fmt::Write;

use super::Tab;

pub(super) fn render_tabs(tabs: &[Tab], group: u32) -> String {
    // The group number is interpolated many times below; format it once and
    // reuse the string instead of allocating per use. The per-tab index is
    // written directly into `out` via `write!`, avoiding a temporary `String`
    // per number.
    let group_str = group.to_string();

    let mut out = String::new();
    out.push_str("<div class=\"ox-tabs-container\"><div class=\"ox-tabs\" data-group=\"");
    out.push_str(&group_str);
    out.push_str("\"><div class=\"ox-tabs-header\">");
    for (index, tab) in tabs.iter().enumerate() {
        out.push_str("<input type=\"radio\" name=\"ox-tabs-");
        out.push_str(&group_str);
        out.push_str("\" id=\"ox-tab-");
        let _ = write!(out, "{group_str}-{index}");
        out.push('"');
        if index == 0 {
            out.push_str(" checked");
        }
        out.push_str("><label for=\"ox-tab-");
        let _ = write!(out, "{group_str}-{index}");
        out.push_str("\">");
        out.push_str(&escape_text(&tab.label));
        out.push_str("</label>");
    }
    out.push_str("</div>");
    for (index, tab) in tabs.iter().enumerate() {
        out.push_str("<div class=\"ox-tab-panel\" data-tab=\"");
        let _ = write!(out, "{index}");
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
