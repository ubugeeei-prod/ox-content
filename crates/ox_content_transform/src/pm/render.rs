use std::fmt::Write;

use super::commands::convert_command;
use super::{PmOptions, PM_GROUP_KEY};

/// The package managers we expand to, in display order.
const PACKAGE_MANAGERS: [&str; 4] = ["npm", "pnpm", "yarn", "bun"];

/// Pull the bare npm command out of a `<pm>` element's inner HTML.
///
/// The element may contain plain text (`<pm>npm i vite</pm>`) or a rendered code
/// block (`<pm><pre><code>npm i vite</code></pre></pm>`); both are supported. We
/// strip any tags, decode the handful of entities a renderer can introduce, and
/// collapse whitespace.
pub(super) fn extract_command(inner: &str) -> String {
    let mut text = String::with_capacity(inner.len());
    let mut in_tag = false;
    for ch in inner.chars() {
        match ch {
            '<' => in_tag = true,
            '>' => in_tag = false,
            _ if !in_tag => text.push(ch),
            _ => {}
        }
    }
    let decoded = decode_entities(&text);
    decoded.split_whitespace().collect::<Vec<_>>().join(" ")
}

/// Decode the small set of HTML entities a renderer may emit inside code.
fn decode_entities(value: &str) -> String {
    value
        .replace("&amp;", "&")
        .replace("&lt;", "<")
        .replace("&gt;", ">")
        .replace("&quot;", "\"")
        .replace("&#39;", "'")
        .replace("&#x27;", "'")
}

/// Render the expanded package-manager tab widget.
///
/// This mirrors [`crate::tabs`] output exactly (same classes, ARIA-free CSS
/// `:has()` widget, `<noscript>` `<details>` fallback) so the shared tab CSS and
/// keyboard runtime apply unchanged. The only additions are the converted code
/// bodies and the opt-in `data-ox-tab-group` attribute.
pub(super) fn render_pm(command: &str, group: u32, options: PmOptions) -> String {
    let group_str = group.to_string();
    let commands: Vec<String> =
        PACKAGE_MANAGERS.iter().map(|pm| convert_command(command, pm)).collect();

    let mut out = String::new();
    out.push_str("<div class=\"ox-tabs-container\"><div class=\"ox-tabs\" data-group=\"");
    out.push_str(&group_str);
    out.push('"');
    if options.sync {
        out.push_str(" data-ox-tab-group=\"");
        out.push_str(PM_GROUP_KEY);
        out.push('"');
    }
    out.push_str("><div class=\"ox-tabs-header\">");
    for (index, pm) in PACKAGE_MANAGERS.iter().enumerate() {
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
        out.push_str(pm);
        out.push_str("</label>");
    }
    out.push_str("</div>");
    for (index, command) in commands.iter().enumerate() {
        out.push_str("<div class=\"ox-tab-panel\" data-tab=\"");
        let _ = write!(out, "{index}");
        out.push_str("\">");
        out.push_str(&code_block(command));
        out.push_str("</div>");
    }
    out.push_str("</div><noscript><div class=\"ox-tabs-fallback\">");
    for (index, (pm, command)) in PACKAGE_MANAGERS.iter().zip(commands.iter()).enumerate() {
        out.push_str("<details");
        if index == 0 {
            out.push_str(" open");
        }
        out.push_str("><summary>");
        out.push_str(pm);
        out.push_str("</summary><div class=\"ox-tabs-fallback-content\">");
        out.push_str(&code_block(command));
        out.push_str("</div></details>");
    }
    out.push_str("</div></noscript></div>");
    out
}

/// Wrap a converted command in a `<pre><code>` block, escaping its text.
fn code_block(command: &str) -> String {
    let mut out = String::with_capacity(command.len() + 24);
    out.push_str("<pre><code>");
    out.push_str(&escape_html(command));
    out.push_str("</code></pre>");
    out
}

/// Escape HTML special characters in command text.
fn escape_html(value: &str) -> String {
    let mut out = String::with_capacity(value.len());
    for ch in value.chars() {
        match ch {
            '&' => out.push_str("&amp;"),
            '<' => out.push_str("&lt;"),
            '>' => out.push_str("&gt;"),
            '"' => out.push_str("&quot;"),
            '\'' => out.push_str("&#39;"),
            _ => out.push(ch),
        }
    }
    out
}
