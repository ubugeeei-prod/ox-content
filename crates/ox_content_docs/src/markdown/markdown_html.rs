//! HTML rendering (raw-HTML-laced Markdown) for generated API reference docs.
//!
//! Selected when `MarkdownDocsOptions::render_style` is `MarkdownRenderStyle::Html`
//! (the default). Child module of `markdown`; reuses the parent's
//! extraction/formatting/link helpers via `super::` and emits the ox-content theme
//! HTML structures (`<details>`, stats, member tables, prose blocks, …).

use std::sync::OnceLock;

use regex::Regex;

use super::{
    cached_regex, clean_summary_text, doc_kind_plural, doc_page_href, effective_members_format,
    effective_parameters_format, entry_anchor, file_stem, format_count_label, format_kind_label,
    generate_source_href, get_entry_badges, member_anchor, normalize_signature,
    parse_example_block, process_doc_text, EntryStats, MarkdownDisplayFormat, MarkdownDocsOptions,
    MarkdownLinkContext, MarkdownPathStrategy, RegexCache, DOC_KIND_ORDER,
};
use crate::model::{
    ApiDocEntry, ApiDocMember, ApiDocModule, ApiDocTag, ApiParamDoc, ApiTypeParamDoc,
};
use crate::string_builder::{join3, StringBuilder};
use std::collections::HashMap;

fn escape_html(value: &str) -> String {
    // Most inputs here are symbol names, type annotations, and kind labels,
    // which usually contain no escapable HTML bytes. The early scan keeps that
    // path to one pass plus `to_string()`, while the escaping path still does a
    // single allocation sized near the input. This replaces chained
    // `replace()` calls that performed five full-string passes and could
    // allocate five intermediate strings for every rendered cell.
    if !value.bytes().any(|b| matches!(b, b'&' | b'<' | b'>' | b'"' | b'\'')) {
        return value.to_string();
    }
    let mut out = String::with_capacity(value.len() + 16);
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

fn render_doc_inline_html(text: &str, context: Option<&MarkdownLinkContext<'_>>) -> String {
    render_inline_html(&process_doc_text(text, context))
}

fn replace_line_breaks_html(html: String) -> String {
    if html.contains('\n') {
        html.replace('\n', "<br>")
    } else {
        html
    }
}

fn render_inline_html(text: &str) -> String {
    static TOKEN_RE: RegexCache = OnceLock::new();

    let Some(token_re) = cached_regex(
        &TOKEN_RE,
        r"`([^`]+)`|\[([^\]]+)\]\(([^)]+)\)|\*\*([^*]+)\*\*|__([^_]+)__|\*([^*]+)\*|_([^_]+)_",
    ) else {
        return replace_line_breaks_html(escape_html(text));
    };
    // Check for a token before allocating the output buffer or entering the
    // capture iterator. Plain prose dominates generated descriptions, and for
    // that case escaping plus newline replacement is enough.
    if !token_re.is_match(text) {
        return replace_line_breaks_html(escape_html(text));
    }

    let mut html = String::new();
    let mut last_index = 0;

    for captures in token_re.captures_iter(text) {
        let Some(mat) = captures.get(0) else {
            continue;
        };
        html.push_str(&escape_html(&text[last_index..mat.start()]));

        if let Some(code) = captures.get(1) {
            html.push_str("<code>");
            html.push_str(&escape_html(code.as_str()));
            html.push_str("</code>");
        } else if let (Some(label), Some(href)) = (captures.get(2), captures.get(3)) {
            html.push_str("<a href=\"");
            html.push_str(&escape_html(href.as_str()));
            html.push_str("\">");
            html.push_str(&render_inline_html(label.as_str()));
            html.push_str("</a>");
        } else if let Some(strong) = captures.get(4).or_else(|| captures.get(5)) {
            html.push_str("<strong>");
            html.push_str(&render_inline_html(strong.as_str()));
            html.push_str("</strong>");
        } else if let Some(emphasis) = captures.get(6).or_else(|| captures.get(7)) {
            html.push_str("<em>");
            html.push_str(&render_inline_html(emphasis.as_str()));
            html.push_str("</em>");
        }

        last_index = mat.end();
    }

    html.push_str(&escape_html(&text[last_index..]));
    replace_line_breaks_html(html)
}

// The four block-start regexes are cached once and shared between the
// value-returning helpers, which need captures, and `is_markdown_block_start`,
// which only needs a boolean and therefore uses allocation-free `is_match`.
// This keeps list/paragraph continuation checks from rebuilding regexes or
// allocating capture groups on every line.
fn fence_re() -> Option<&'static Regex> {
    static FENCE_RE: RegexCache = OnceLock::new();
    cached_regex(&FENCE_RE, r"^```([\w-]+)?\s*$")
}

fn heading_re() -> Option<&'static Regex> {
    static HEADING_RE: RegexCache = OnceLock::new();
    cached_regex(&HEADING_RE, r"^(#{1,6})\s+(.*)$")
}

fn ordered_re() -> Option<&'static Regex> {
    static ORDERED_RE: RegexCache = OnceLock::new();
    cached_regex(&ORDERED_RE, r"^\d+\.\s+(.*)$")
}

fn unordered_re() -> Option<&'static Regex> {
    static UNORDERED_RE: RegexCache = OnceLock::new();
    cached_regex(&UNORDERED_RE, r"^[-*+]\s+(.*)$")
}

fn is_fence_start(line: &str) -> Option<String> {
    fence_re()?
        .captures(line.trim())
        .map(|captures| captures.get(1).map_or("text", |value| value.as_str()).to_string())
}

fn heading_match(line: &str) -> Option<(usize, String)> {
    heading_re()?.captures(line.trim()).map(|captures| {
        (
            captures.get(1).map_or(1, |value| value.as_str().len()).min(6),
            captures.get(2).map_or("", |value| value.as_str()).trim().to_string(),
        )
    })
}

fn ordered_list_item(line: &str) -> Option<String> {
    ordered_re()?
        .captures(line.trim())
        .and_then(|captures| captures.get(1).map(|value| value.as_str().to_string()))
}

fn unordered_list_item(line: &str) -> Option<String> {
    unordered_re()?
        .captures(line.trim())
        .and_then(|captures| captures.get(1).map(|value| value.as_str().to_string()))
}

fn is_markdown_block_start(line: &str) -> bool {
    // Only a boolean is needed, so use `is_match` (no capture allocation) and
    // trim once. The regexes are `^…$`-anchored, so `is_match(trimmed)` is true
    // exactly when the corresponding `captures(trimmed)` was `Some`.
    let trimmed = line.trim();
    fence_re().is_some_and(|re| re.is_match(trimmed))
        || heading_re().is_some_and(|re| re.is_match(trimmed))
        || ordered_re().is_some_and(|re| re.is_match(trimmed))
        || unordered_re().is_some_and(|re| re.is_match(trimmed))
}

fn render_markdown_blocks_html(text: &str) -> String {
    // This renderer handles the small Markdown subset embedded in generated
    // API descriptions. It walks the line slice once and emits blocks as soon
    // as they are recognized. Continuation checks use cached regexes and the
    // inline renderer's own token precheck so ordinary paragraphs do not pay
    // for full Markdown parsing.
    static ORDERED_CONTINUATION_RE: RegexCache = OnceLock::new();
    static UNORDERED_CONTINUATION_RE: RegexCache = OnceLock::new();

    let lines: Vec<&str> =
        text.split('\n').map(|line| line.strip_suffix('\r').unwrap_or(line)).collect();
    let mut blocks = Vec::new();
    let mut index = 0;
    let ordered_continuation_re = cached_regex(&ORDERED_CONTINUATION_RE, r"^ {0,1}\d+\.\s+");
    let unordered_continuation_re = cached_regex(&UNORDERED_CONTINUATION_RE, r"^[-*+]\s+");

    while index < lines.len() {
        let line = lines[index];
        let trimmed = line.trim();

        if trimmed.is_empty() {
            index += 1;
            continue;
        }

        if let Some(language) = is_fence_start(line) {
            let mut code_lines = Vec::new();
            index += 1;

            while index < lines.len() && !lines[index].trim().starts_with("```") {
                code_lines.push(lines[index]);
                index += 1;
            }

            if index < lines.len() {
                index += 1;
            }

            blocks.push(render_code_block_html(&code_lines.join("\n"), &language));
            continue;
        }

        if let Some((level, content)) = heading_match(line) {
            let content = render_inline_html(&content);
            let mut block = StringBuilder::with_capacity(content.len() + 9);
            block.push_str("<h");
            block.push_usize(level);
            block.push_char('>');
            block.push_str(&content);
            block.push_str("</h");
            block.push_usize(level);
            block.push_char('>');
            blocks.push(block.into_string());
            index += 1;
            continue;
        }

        if let Some(first_item) = ordered_list_item(line) {
            let mut items = Vec::new();
            let mut current = Some(first_item);

            while index < lines.len() {
                let Some(item_text) = current.take().or_else(|| ordered_list_item(lines[index]))
                else {
                    break;
                };

                let mut item_lines = vec![item_text.trim().to_string()];
                index += 1;

                while index < lines.len() {
                    let continuation = lines[index];
                    let continuation_trimmed = continuation.trim();

                    if continuation_trimmed.is_empty()
                        || is_markdown_block_start(continuation)
                        || ordered_continuation_re
                            .is_some_and(|re| re.is_match(continuation_trimmed))
                    {
                        break;
                    }

                    item_lines.push(continuation_trimmed.to_string());
                    index += 1;
                }

                items.push(join3("<li>", &render_inline_html(&item_lines.join(" ")), "</li>"));

                if index < lines.len() && lines[index].trim().is_empty() {
                    break;
                }
            }

            blocks.push(join3("<ol>\n", &items.join("\n"), "\n</ol>"));
            continue;
        }

        if let Some(first_item) = unordered_list_item(line) {
            let mut items = Vec::new();
            let mut current = Some(first_item);

            while index < lines.len() {
                let Some(item_text) = current.take().or_else(|| unordered_list_item(lines[index]))
                else {
                    break;
                };

                let mut item_lines = vec![item_text.trim().to_string()];
                index += 1;

                while index < lines.len() {
                    let continuation = lines[index];
                    let continuation_trimmed = continuation.trim();

                    if continuation_trimmed.is_empty()
                        || is_markdown_block_start(continuation)
                        || unordered_continuation_re
                            .is_some_and(|re| re.is_match(continuation_trimmed))
                    {
                        break;
                    }

                    item_lines.push(continuation_trimmed.to_string());
                    index += 1;
                }

                items.push(join3("<li>", &render_inline_html(&item_lines.join(" ")), "</li>"));

                if index < lines.len() && lines[index].trim().is_empty() {
                    break;
                }
            }

            blocks.push(join3("<ul>\n", &items.join("\n"), "\n</ul>"));
            continue;
        }

        let mut paragraph_lines = vec![trimmed.to_string()];
        index += 1;

        while index < lines.len() {
            let next_line = lines[index];
            let next_trimmed = next_line.trim();

            if next_trimmed.is_empty() || is_markdown_block_start(next_line) {
                break;
            }

            paragraph_lines.push(next_trimmed.to_string());
            index += 1;
        }

        blocks.push(join3("<p>", &render_inline_html(&paragraph_lines.join(" ")), "</p>"));
    }

    join3("<div class=\"ox-api-entry__prose\">\n", &blocks.join("\n"), "\n</div>")
}

fn render_code_block_html(code: &str, language: &str) -> String {
    let code = escape_html(code);
    let mut out = StringBuilder::with_capacity(
        "<pre><code class=\"language-\"></code></pre>".len() + language.len() + code.len(),
    );
    out.push_str("<pre><code class=\"language-");
    out.push_str(language);
    out.push_str("\">");
    out.push_str(&code);
    out.push_str("</code></pre>");
    out.into_string()
}

fn render_highlighted_inline_code_html(code: &str, class_name: &str, language: &str) -> String {
    let class_name = escape_html(class_name);
    let code = escape_html(code);
    let mut out = StringBuilder::with_capacity(
        "<code class=\" language-\"></code>".len() + class_name.len() + language.len() + code.len(),
    );
    out.push_str("<code class=\"");
    out.push_str(&class_name);
    out.push_str(" language-");
    out.push_str(language);
    out.push_str("\">");
    out.push_str(&code);
    out.push_str("</code>");
    out.into_string()
}

pub(super) fn render_details_controls_html(target_selector: &str) -> String {
    let mut out = StringBuilder::with_capacity(260 + target_selector.len());
    out.push_str("<div class=\"ox-api-controls\" data-ox-api-target=\"");
    out.push_str(target_selector);
    out.push_str("\" role=\"toolbar\" aria-label=\"Reference display controls\">
<button type=\"button\" class=\"ox-api-controls__button\" data-ox-api-toggle=\"expand\">Open all</button>
<button type=\"button\" class=\"ox-api-controls__button\" data-ox-api-toggle=\"collapse\">Close all</button>
</div>");
    out.into_string()
}

pub(super) fn render_stats_html(stats: &EntryStats, module_count: Option<usize>) -> String {
    let mut rendered_items = StringBuilder::new();
    if let Some(module_count) = module_count {
        push_stat_html(&mut rendered_items, "modules", module_count, None);
    }

    push_stat_html(&mut rendered_items, "symbols", stats.entries, None);

    for kind in DOC_KIND_ORDER {
        if let Some(count) = stats.by_kind.get(kind).copied().filter(|count| *count > 0) {
            push_stat_html(&mut rendered_items, doc_kind_plural(kind), count, None);
        }
    }

    if stats.params > 0 {
        push_stat_html(&mut rendered_items, "parameters", stats.params, None);
    }
    if stats.members > 0 {
        push_stat_html(&mut rendered_items, "members", stats.members, None);
    }
    if stats.returns > 0 {
        push_stat_html(&mut rendered_items, "returns", stats.returns, None);
    }
    if stats.examples > 0 {
        push_stat_html(&mut rendered_items, "examples", stats.examples, None);
    }
    if stats.deprecated > 0 {
        push_stat_html(&mut rendered_items, "deprecated", stats.deprecated, Some("warning"));
    }

    let rendered_items = rendered_items.into_string();
    let mut out = StringBuilder::with_capacity(rendered_items.len() + 80);
    out.push_str("<div class=\"ox-api-stats\" aria-label=\"API reference summary\">\n");
    out.push_str(&rendered_items);
    out.push_str("\n</div>");
    out.into_string()
}

fn push_stat_html(out: &mut StringBuilder, label: &str, value: usize, tone: Option<&str>) {
    if !out.is_empty() {
        out.push_char('\n');
    }
    out.push_str("<span class=\"ox-api-stat");
    if let Some(tone) = tone {
        out.push_str(" ox-api-stat--");
        out.push_str(tone);
    }
    out.push_str("\">\n  <strong>");
    out.push_usize(value);
    out.push_str("</strong>\n  <span>");
    out.push_str(label);
    out.push_str("</span>\n</span>");
}

fn render_entry_badges_html(entry: &ApiDocEntry, class_name: &str) -> String {
    let badges = get_entry_badges(entry);
    if badges.is_empty() {
        return String::new();
    }

    let mut rendered = StringBuilder::new();
    for badge in badges {
        rendered.push_str("<span class=\"ox-api-badge");
        if let Some(tone) = badge.tone {
            rendered.push_str(" ox-api-badge--");
            rendered.push_str(tone);
        }
        rendered.push_str("\">");
        rendered.push_str(&escape_html(&badge.label));
        rendered.push_str("</span>");
    }

    let rendered = rendered.into_string();
    let mut out = StringBuilder::with_capacity(class_name.len() + rendered.len() + 23);
    out.push_str("<span class=\"");
    out.push_str(class_name);
    out.push_str("\">");
    out.push_str(&rendered);
    out.push_str("</span>");
    out.into_string()
}

fn render_overview_html_item(
    entry: &ApiDocEntry,
    href: &str,
    context: Option<&MarkdownLinkContext<'_>>,
) -> String {
    let signature = normalize_signature(entry.signature.as_deref());
    let summary = clean_summary_text(&process_doc_text(&entry.description, context), 88);
    let meta = render_entry_badges_html(entry, "ox-api-module__meta");
    let href = escape_html(href);
    let heading = if let Some(signature) = signature {
        let content = render_highlighted_inline_code_html(
            &signature,
            "ox-api-module__signature ox-api-module__signature--highlighted",
            "typescript",
        );
        let mut heading = StringBuilder::with_capacity(href.len() + content.len() + 43);
        heading.push_str("<a href=\"");
        heading.push_str(&href);
        heading.push_str("\" class=\"ox-api-module__link\">");
        heading.push_str(&content);
        heading.push_str("</a>");
        heading.into_string()
    } else {
        let name = escape_html(&entry.name);
        let mut heading = StringBuilder::with_capacity(href.len() + name.len() + 80);
        heading.push_str("<a href=\"");
        heading.push_str(&href);
        heading.push_str("\" class=\"ox-api-module__link\"><code class=\"ox-api-module__name\">");
        heading.push_str(&name);
        heading.push_str("</code></a>");
        heading.into_string()
    };

    let summary_html = if summary.is_empty() {
        String::new()
    } else {
        join3("<span class=\"ox-api-module__summary\">", &render_inline_html(&summary), "</span>")
    };
    let kind = escape_html(format_kind_label(&entry.kind));
    let mut item = StringBuilder::with_capacity(
        kind.len() + heading.len() + summary_html.len() + meta.len() + 92,
    );
    item.push_str("<li><span class=\"ox-api-module__kind\">");
    item.push_str(&kind);
    item.push_str("</span><div class=\"ox-api-module__item\">");
    item.push_str(&heading);
    item.push_str(&summary_html);
    item.push_str(&meta);
    item.push_str("</div></li>");
    item.into_string()
}

fn render_params_list_html(
    params: &[ApiParamDoc],
    context: Option<&MarkdownLinkContext<'_>>,
) -> String {
    let mut rows = StringBuilder::new();
    for param in params {
        if !rows.is_empty() {
            rows.push_char('\n');
        }
        let description = render_member_param_description(param);
        rows.push_str("<li class=\"ox-api-entry__param\">\n  <div class=\"ox-api-entry__param-heading\">\n    <code class=\"ox-api-entry__param-name\">");
        rows.push_str(&escape_html(&param.name));
        rows.push_str("</code>\n    <code class=\"ox-api-entry__param-type\">");
        rows.push_str(&escape_html(&param.type_annotation));
        rows.push_str("</code>\n  </div>\n  ");
        if !description.is_empty() {
            rows.push_str("<p class=\"ox-api-entry__param-description\">");
            rows.push_str(&render_doc_inline_html(&description, context));
            rows.push_str("</p>");
        }
        rows.push_str("\n</li>");
    }
    let rows = rows.into_string();

    let mut out = StringBuilder::with_capacity(rows.len() + 125);
    out.push_str(
        "<div class=\"ox-api-entry__section ox-api-entry__section--params\">
<h4>Parameters</h4>
<ul class=\"ox-api-entry__params\">
",
    );
    out.push_str(&rows);
    out.push_str(
        "
</ul>
</div>",
    );
    out.into_string()
}

fn render_params_table_html(
    params: &[ApiParamDoc],
    context: Option<&MarkdownLinkContext<'_>>,
) -> String {
    let mut rows = StringBuilder::new();
    for param in params {
        if !rows.is_empty() {
            rows.push_char('\n');
        }
        let description = render_member_param_description(param);
        rows.push_str("<tr>\n  <td><code>");
        rows.push_str(&escape_html(&param.name));
        rows.push_str("</code></td>\n  <td><code>");
        rows.push_str(&escape_html(&param.type_annotation));
        rows.push_str("</code></td>\n  <td>");
        rows.push_str(&render_doc_inline_html(&description, context));
        rows.push_str("</td>\n</tr>");
    }
    let rows = rows.into_string();

    let mut out = StringBuilder::with_capacity(rows.len() + 220);
    out.push_str(
        "<div class=\"ox-api-entry__section ox-api-entry__section--params\">
<h4>Parameters</h4>
<table class=\"ox-api-entry__params-table\">
<thead><tr><th>Name</th><th>Type</th><th>Description</th></tr></thead>
<tbody>
",
    );
    out.push_str(&rows);
    out.push_str(
        "
</tbody>
</table>
</div>",
    );
    out.into_string()
}

fn render_type_parameter_name_html(type_param: &ApiTypeParamDoc) -> String {
    let mut name = StringBuilder::new();
    name.push_str("<code>");
    name.push_str(&escape_html(&type_param.name));
    name.push_str("</code>");
    if let Some(constraint) = &type_param.constraint {
        name.push_str(" <em>extends</em> <code>");
        name.push_str(&escape_html(constraint));
        name.push_str("</code>");
    }
    if let Some(default) = &type_param.default {
        name.push_str(" = <code>");
        name.push_str(&escape_html(default));
        name.push_str("</code>");
    }
    name.into_string()
}

fn render_type_parameters_table_html(
    type_parameters: &[ApiTypeParamDoc],
    context: Option<&MarkdownLinkContext<'_>>,
) -> String {
    let mut rows = StringBuilder::new();
    for type_param in type_parameters {
        if !rows.is_empty() {
            rows.push_char('\n');
        }
        rows.push_str("<tr>\n  <td>");
        rows.push_str(&render_type_parameter_name_html(type_param));
        rows.push_str("</td>\n  <td>");
        rows.push_str(&render_doc_inline_html(&type_param.description, context));
        rows.push_str("</td>\n</tr>");
    }
    let rows = rows.into_string();

    let mut out = StringBuilder::with_capacity(rows.len() + 235);
    out.push_str(
        "<div class=\"ox-api-entry__section ox-api-entry__section--type-parameters\">
<h4>Type Parameters</h4>
<table class=\"ox-api-entry__type-parameters-table\">
<thead><tr><th>Name</th><th>Description</th></tr></thead>
<tbody>
",
    );
    out.push_str(&rows);
    out.push_str(
        "
</tbody>
</table>
</div>",
    );
    out.into_string()
}

fn render_type_parameters_list_html(
    type_parameters: &[ApiTypeParamDoc],
    context: Option<&MarkdownLinkContext<'_>>,
) -> String {
    let mut items = StringBuilder::new();
    for type_param in type_parameters {
        if !items.is_empty() {
            items.push_char('\n');
        }
        items.push_str("<li class=\"ox-api-entry__type-parameter\">\n  <div class=\"ox-api-entry__type-parameter-heading\">");
        items.push_str(&render_type_parameter_name_html(type_param));
        items.push_str("</div>\n  ");
        if !type_param.description.is_empty() {
            items.push_str("<p class=\"ox-api-entry__type-parameter-description\">");
            items.push_str(&render_doc_inline_html(&type_param.description, context));
            items.push_str("</p>");
        }
        items.push_str("\n</li>");
    }
    let items = items.into_string();

    let mut out = StringBuilder::with_capacity(items.len() + 145);
    out.push_str(
        "<div class=\"ox-api-entry__section ox-api-entry__section--type-parameters\">
<h4>Type Parameters</h4>
<ul class=\"ox-api-entry__type-parameters\">
",
    );
    out.push_str(&items);
    out.push_str(
        "
</ul>
</div>",
    );
    out.into_string()
}

fn render_tag_list_html(tags: &[ApiDocTag], context: Option<&MarkdownLinkContext<'_>>) -> String {
    let mut items = StringBuilder::new();
    for tag in tags {
        items.push_str("<li><span class=\"ox-api-entry__tag-name\">@");
        items.push_str(&escape_html(&tag.tag));
        items.push_str("</span><span class=\"ox-api-entry__tag-value\">");
        items.push_str(&render_doc_inline_html(&tag.value, context));
        items.push_str("</span></li>");
    }
    let items = items.into_string();

    let mut out = StringBuilder::with_capacity(items.len() + 115);
    out.push_str(
        "<div class=\"ox-api-entry__section ox-api-entry__section--tags\">
<h4>Tags</h4>
<ul class=\"ox-api-entry__tags\">",
    );
    out.push_str(&items);
    out.push_str(
        "</ul>
</div>",
    );
    out.into_string()
}

fn render_member_flags(member: &ApiDocMember) -> String {
    let mut flags = Vec::new();
    if member.optional {
        flags.push("optional");
    }
    if member.readonly {
        flags.push("readonly");
    }
    if member.r#static {
        flags.push("static");
    }
    if member.private {
        flags.push("private");
    }

    let mut html = String::new();
    for flag in flags {
        html.push_str("<span class=\"ox-api-badge\">");
        html.push_str(flag);
        html.push_str("</span>");
    }
    html
}

fn render_member_type_html(member: &ApiDocMember) -> String {
    let value = member
        .signature
        .as_deref()
        .or(member.type_annotation.as_deref())
        .or_else(|| member.returns.as_ref().map(|returns| returns.type_annotation.as_str()));

    value.map_or_else(String::new, |value| {
        render_highlighted_inline_code_html(value, "ox-api-entry__member-type", "typescript")
    })
}

fn render_member_description_html(
    member: &ApiDocMember,
    options: &MarkdownDocsOptions,
    context: Option<&MarkdownLinkContext<'_>>,
) -> String {
    let mut blocks = Vec::new();

    if !member.description.is_empty() {
        blocks.push(join3(
            "<div class=\"ox-api-entry__member-description\">",
            &render_doc_inline_html(&member.description, context),
            "</div>",
        ));
    }

    if !member.params.is_empty() {
        blocks.push(render_member_params_html(&member.params, options, context));
    }

    if let Some(returns) = &member.returns {
        if !returns.description.is_empty() {
            blocks.push(join3(
                "<div class=\"ox-api-entry__member-return\"><span>Returns</span> ",
                &render_doc_inline_html(&returns.description, context),
                "</div>",
            ));
        }
    }

    blocks.join("")
}

fn render_member_param_description(param: &ApiParamDoc) -> String {
    let mut description = param.description.clone();
    let mut flags = String::new();
    if param.optional {
        flags.push_str("optional");
    }
    if let Some(default_value) = &param.default_value {
        if !flags.is_empty() {
            flags.push_str(" · ");
        }
        flags.push_str("default: ");
        flags.push_str(default_value);
    }
    if !flags.is_empty() {
        if !description.is_empty() {
            description.push_str(" — ");
        }
        description.push_str(&flags);
    }
    description
}

fn render_member_params_html(
    params: &[ApiParamDoc],
    options: &MarkdownDocsOptions,
    context: Option<&MarkdownLinkContext<'_>>,
) -> String {
    if effective_parameters_format(options) == MarkdownDisplayFormat::Table {
        let mut rows = StringBuilder::new();
        for param in params {
            let description = render_member_param_description(param);
            rows.push_str("<tr><td><code>");
            rows.push_str(&escape_html(&param.name));
            rows.push_str("</code></td><td><code>");
            rows.push_str(&escape_html(&param.type_annotation));
            rows.push_str("</code></td><td>");
            rows.push_str(&render_doc_inline_html(&description, context));
            rows.push_str("</td></tr>");
        }
        let rows = rows.into_string();

        return join3(
            "<table class=\"ox-api-entry__member-params-table\"><thead><tr><th>Name</th><th>Type</th><th>Description</th></tr></thead><tbody>",
            &rows,
            "</tbody></table>",
        );
    }

    let mut items = StringBuilder::new();
    for param in params {
        let description = render_member_param_description(param);
        items.push_str("<li><code>");
        items.push_str(&escape_html(&param.name));
        items.push_str("</code>");
        if !description.is_empty() {
            items.push_char(' ');
            items.push_str(&render_doc_inline_html(&description, context));
        }
        items.push_str("</li>");
    }
    join3("<ul class=\"ox-api-entry__member-params\">", &items.into_string(), "</ul>")
}

fn render_member_table_html(
    entry_name: &str,
    title: &str,
    members: &[&ApiDocMember],
    options: &MarkdownDocsOptions,
    context: Option<&MarkdownLinkContext<'_>>,
) -> String {
    if members.is_empty() {
        return String::new();
    }

    let mut rows = StringBuilder::new();
    for member in members {
        if !rows.is_empty() {
            rows.push_char('\n');
        }
        rows.push_str("<tr id=\"");
        rows.push_str(&escape_html(&member_anchor(
            entry_name,
            member,
            context.map_or(MarkdownPathStrategy::Flat, |context| context.options.path_strategy),
        )));
        rows.push_str("\">\n  <td><code>");
        rows.push_str(&escape_html(&member.name));
        rows.push_str("</code>");
        rows.push_str(&render_member_flags(member));
        rows.push_str("</td>\n  <td><span class=\"ox-api-entry__member-kind\">");
        rows.push_str(&escape_html(&member.kind));
        rows.push_str("</span></td>\n  <td>");
        rows.push_str(&render_member_type_html(member));
        rows.push_str("</td>\n  <td>");
        rows.push_str(&render_member_description_html(member, options, context));
        rows.push_str("</td>\n</tr>");
    }
    let rows = rows.into_string();

    let title = escape_html(title);
    let mut out = StringBuilder::with_capacity(rows.len() + title.len() + 235);
    out.push_str(
        "<div class=\"ox-api-entry__member-group\">
<h5>",
    );
    out.push_str(&title);
    out.push_str(
        "</h5>
<table class=\"ox-api-entry__members-table\">
<thead><tr><th>Name</th><th>Kind</th><th>Type</th><th>Description</th></tr></thead>
<tbody>
",
    );
    out.push_str(&rows);
    out.push_str(
        "
</tbody>
</table>
</div>",
    );
    out.into_string()
}

fn render_member_list_html(
    entry_name: &str,
    title: &str,
    members: &[&ApiDocMember],
    options: &MarkdownDocsOptions,
    context: Option<&MarkdownLinkContext<'_>>,
) -> String {
    if members.is_empty() {
        return String::new();
    }

    let mut items = StringBuilder::new();
    for member in members {
        if !items.is_empty() {
            items.push_char('\n');
        }
        items.push_str("<li id=\"");
        items.push_str(&escape_html(&member_anchor(
            entry_name,
            member,
            context.map_or(MarkdownPathStrategy::Flat, |context| context.options.path_strategy),
        )));
        items.push_str("\" class=\"ox-api-entry__member\">\n  <div class=\"ox-api-entry__member-heading\">\n    <code class=\"ox-api-entry__member-name\">");
        items.push_str(&escape_html(&member.name));
        items.push_str("</code>");
        items.push_str(&render_member_flags(member));
        items.push_str("\n    <span class=\"ox-api-entry__member-kind\">");
        items.push_str(&escape_html(&member.kind));
        items.push_str("</span>\n    ");
        items.push_str(&render_member_type_html(member));
        items.push_str("\n  </div>\n  ");
        items.push_str(&render_member_description_html(member, options, context));
        items.push_str("\n</li>");
    }
    let items = items.into_string();

    let title = escape_html(title);
    let mut out = StringBuilder::with_capacity(items.len() + title.len() + 135);
    out.push_str(
        "<div class=\"ox-api-entry__member-group\">
<h5>",
    );
    out.push_str(&title);
    out.push_str(
        "</h5>
<ul class=\"ox-api-entry__members-list\">
",
    );
    out.push_str(&items);
    out.push_str(
        "
</ul>
</div>",
    );
    out.into_string()
}

fn render_member_group_html(
    entry: &ApiDocEntry,
    title: &str,
    members: &[&ApiDocMember],
    options: &MarkdownDocsOptions,
    context: Option<&MarkdownLinkContext<'_>>,
) -> String {
    if effective_members_format(options, &entry.kind, title) == MarkdownDisplayFormat::List {
        render_member_list_html(&entry.name, title, members, options, context)
    } else {
        render_member_table_html(&entry.name, title, members, options, context)
    }
}

fn render_members_html(
    entry: &ApiDocEntry,
    options: &MarkdownDocsOptions,
    context: Option<&MarkdownLinkContext<'_>>,
) -> String {
    if entry.members.is_empty() {
        return String::new();
    }

    // Bucket the members lazily: each `match` arm below only uses a subset of
    // these groups (the default arm uses none of them), so computing every
    // bucket up front wasted a full `members` pass + `Vec` per unused group.
    let members = entry.members.as_slice();
    let methods = |is_static: bool| {
        members
            .iter()
            .filter(|member| {
                member.r#static == is_static
                    && matches!(member.kind.as_str(), "method" | "getter" | "setter")
            })
            .collect::<Vec<_>>()
    };
    let properties = |is_static: bool| {
        members
            .iter()
            .filter(|member| member.r#static == is_static && member.kind == "property")
            .collect::<Vec<_>>()
    };

    let mut groups = Vec::new();
    match entry.kind.as_str() {
        "class" => {
            let constructors =
                members.iter().filter(|member| member.kind == "constructor").collect::<Vec<_>>();
            groups.push(render_member_group_html(
                entry,
                "Constructors",
                &constructors,
                options,
                context,
            ));
            groups.push(render_member_group_html(
                entry,
                "Static Methods",
                &methods(true),
                options,
                context,
            ));
            groups.push(render_member_group_html(
                entry,
                "Methods",
                &methods(false),
                options,
                context,
            ));
            groups.push(render_member_group_html(
                entry,
                "Static Properties",
                &properties(true),
                options,
                context,
            ));
            groups.push(render_member_group_html(
                entry,
                "Properties",
                &properties(false),
                options,
                context,
            ));
        }
        "interface" => {
            groups.push(render_member_group_html(
                entry,
                "Properties",
                &properties(false),
                options,
                context,
            ));
            groups.push(render_member_group_html(
                entry,
                "Methods",
                &methods(false),
                options,
                context,
            ));
        }
        "type" => {
            let enum_members =
                members.iter().filter(|member| member.kind == "enumMember").collect::<Vec<_>>();
            groups.push(render_member_group_html(
                entry,
                "Properties",
                &properties(false),
                options,
                context,
            ));
            groups.push(render_member_group_html(
                entry,
                "Methods",
                &methods(false),
                options,
                context,
            ));
            groups.push(render_member_group_html(
                entry,
                "Enum Members",
                &enum_members,
                options,
                context,
            ));
        }
        _ => groups.push(render_member_group_html(
            entry,
            "Members",
            &members.iter().collect::<Vec<_>>(),
            options,
            context,
        )),
    }

    let groups = groups.into_iter().filter(|group| !group.is_empty()).collect::<Vec<_>>();
    if groups.is_empty() {
        return String::new();
    }

    join3(
        "<div class=\"ox-api-entry__section ox-api-entry__section--members\">
<h4>Members</h4>
",
        &groups.join("\n"),
        "
</div>",
    )
}

fn render_entry_body_html(
    entry: &ApiDocEntry,
    options: &MarkdownDocsOptions,
    link_context: Option<&MarkdownLinkContext<'_>>,
) -> String {
    let processed_description = process_doc_text(&entry.description, link_context);
    // Entries with an empty `file` (e.g. symbols re-exported from an external
    // package) have no source in the consumer's repo, so emit no source link.
    let source_href =
        options.github_url.as_ref().filter(|_| !entry.file.is_empty()).map(|github_url| {
            generate_source_href(&entry.file, github_url, Some(entry.line), Some(entry.end_line))
        });
    let mut body = String::new();

    if !processed_description.is_empty() {
        body.push_str(&render_markdown_blocks_html(&processed_description));
        body.push('\n');
    }

    if let Some(signature) = &entry.signature {
        body.push_str(
            "<div class=\"ox-api-entry__section ox-api-entry__section--signature\">
<h4>Signature</h4>
",
        );
        body.push_str(&render_code_block_html(signature, "typescript"));
        body.push_str(
            "
</div>\n",
        );
    }

    if let Some(source_href) = source_href {
        body.push_str(
            "<p class=\"ox-api-entry__source\"><a class=\"ox-api-entry__source-link\" href=\"",
        );
        body.push_str(&escape_html(&source_href));
        body.push_str("\" target=\"_blank\" rel=\"noopener noreferrer\">View source<span class=\"ox-api-entry__source-icon\" aria-hidden=\"true\"></span></a></p>\n");
    }

    if !entry.type_parameters.is_empty() {
        if effective_parameters_format(options) == MarkdownDisplayFormat::List {
            body.push_str(&render_type_parameters_list_html(&entry.type_parameters, link_context));
        } else {
            body.push_str(&render_type_parameters_table_html(&entry.type_parameters, link_context));
        }
        body.push('\n');
    }

    if !entry.members.is_empty() {
        body.push_str(&render_members_html(entry, options, link_context));
        body.push('\n');
    }

    if !entry.params.is_empty() {
        if effective_parameters_format(options) == MarkdownDisplayFormat::Table {
            body.push_str(&render_params_table_html(&entry.params, link_context));
        } else {
            body.push_str(&render_params_list_html(&entry.params, link_context));
        }
        body.push('\n');
    }

    if let Some(returns) = &entry.returns {
        body.push_str(
            "<div class=\"ox-api-entry__section ox-api-entry__section--returns\">
<h4>Returns</h4>
<div class=\"ox-api-entry__return\">
  <code class=\"ox-api-entry__return-type\">",
        );
        body.push_str(&escape_html(&returns.type_annotation));
        body.push_str(
            "</code>
  ",
        );
        if !returns.description.is_empty() {
            body.push_str("<p class=\"ox-api-entry__return-description\">");
            body.push_str(&render_doc_inline_html(&returns.description, link_context));
            body.push_str("</p>");
        }
        body.push_str(
            "
</div>
</div>\n",
        );
    }

    if !entry.examples.is_empty() {
        let mut examples_html = StringBuilder::new();
        for (index, example) in entry.examples.iter().enumerate() {
            if !examples_html.is_empty() {
                examples_html.push_char('\n');
            }
            let (code, language) = parse_example_block(example);
            examples_html.push_str(
                "<div class=\"ox-api-entry__example\">
<div class=\"ox-api-entry__example-heading\">Example ",
            );
            examples_html.push_usize(index + 1);
            examples_html.push_str(
                "</div>
",
            );
            examples_html.push_str(&render_code_block_html(&code, &language));
            examples_html.push_str(
                "
</div>",
            );
        }

        body.push_str(
            "<div class=\"ox-api-entry__section ox-api-entry__section--examples\">
<h4>Examples</h4>
",
        );
        body.push_str(&examples_html.into_string());
        body.push_str(
            "
</div>\n",
        );
    }

    if !entry.tags.is_empty() {
        body.push_str(&render_tag_list_html(&entry.tags, link_context));
        body.push('\n');
    }

    body.trim().to_string()
}

pub(super) fn render_entry_html(
    entry: &ApiDocEntry,
    options: &MarkdownDocsOptions,
    link_context: Option<&MarkdownLinkContext<'_>>,
) -> String {
    let processed_description = process_doc_text(&entry.description, link_context);
    let summary_signature = normalize_signature(entry.signature.as_deref());
    let body = render_entry_body_html(entry, options, link_context);

    let summary_description = clean_summary_text(
        &processed_description,
        if summary_signature.is_some() { 80 } else { 120 },
    );
    let summary_heading = if let Some(summary_signature) = summary_signature {
        render_highlighted_inline_code_html(
            &summary_signature,
            "ox-api-entry__signature ox-api-entry__signature--highlighted",
            "typescript",
        )
    } else {
        join3("<code class=\"ox-api-entry__name\">", &escape_html(&entry.name), "</code>")
    };
    let summary_description = if summary_description.is_empty() {
        String::new()
    } else {
        join3(
            "<span class=\"ox-api-entry__description\">",
            &render_inline_html(&summary_description),
            "</span>",
        )
    };
    let badges = render_entry_badges_html(entry, "ox-api-entry__meta");
    let kind = escape_html(format_kind_label(&entry.kind));
    let mut summary = StringBuilder::with_capacity(
        kind.len() + summary_heading.len() + summary_description.len() + badges.len() + 92,
    );
    summary.push_str("<span class=\"ox-api-entry__kind\">");
    summary.push_str(&kind);
    summary.push_str("</span><span class=\"ox-api-entry__summary-main\">");
    summary.push_str(&summary_heading);
    summary.push_str(&summary_description);
    summary.push_str(&badges);
    summary.push_str("</span>");
    let summary = summary.into_string();
    let anchor = entry_anchor(&entry.name);

    let mut out = StringBuilder::with_capacity(anchor.len() + summary.len() + body.len() + 120);
    out.push_str("<details id=\"");
    out.push_str(&anchor);
    out.push_str(
        "\" class=\"ox-api-entry\">
  <summary>",
    );
    out.push_str(&summary);
    out.push_str(
        "</summary>
  <div class=\"ox-api-entry__body\">
",
    );
    out.push_str(&body);
    out.push_str(
        "
  </div>
</details>

",
    );
    out.into_string()
}

pub(super) fn render_entry_page_html(
    entry: &ApiDocEntry,
    options: &MarkdownDocsOptions,
    link_context: Option<&MarkdownLinkContext<'_>>,
) -> String {
    let body = render_entry_body_html(entry, options, link_context);
    let anchor = entry_anchor(&entry.name);
    let mut out = StringBuilder::with_capacity(anchor.len() + body.len() + 64);
    out.push_str("<div id=\"");
    out.push_str(&anchor);
    out.push_str(
        "\" class=\"ox-api-entry ox-api-entry--page\">
",
    );
    out.push_str(&body);
    out.push_str(
        "
</div>
",
    );
    out.into_string()
}

pub(super) fn render_module_section_html(
    doc: &ApiDocModule,
    options: &MarkdownDocsOptions,
    file_name: &str,
    display_name: &str,
    count_label: &str,
    link_context: Option<&MarkdownLinkContext<'_>>,
) -> String {
    let mut markdown = String::new();
    markdown.push_str(
        "<details class=\"ox-api-module\">
  <summary>
    <span class=\"ox-api-module__title\"><a href=\"",
    );
    markdown.push_str(&escape_html(&doc_page_href(options, file_name, None)));
    markdown.push_str("\">");
    markdown.push_str(&escape_html(display_name));
    markdown.push_str(
        "</a></span>
    <span class=\"ox-api-module__count\">",
    );
    markdown.push_str(count_label);
    markdown.push_str(
        "</span>
  </summary>
  <div class=\"ox-api-module__body\">
    <ul class=\"ox-api-module__list\">
",
    );

    for entry in &doc.entries {
        let href = doc_page_href(options, file_name, Some(&entry_anchor(&entry.name)));
        markdown.push_str("      ");
        markdown.push_str(&render_overview_html_item(entry, &href, link_context));
        markdown.push('\n');
    }

    markdown.push_str(
        "    </ul>
  </div>
</details>

",
    );

    markdown
}

pub(super) fn render_module_index_html(
    docs: &[ApiDocModule],
    options: &MarkdownDocsOptions,
    doc_to_file: Option<&HashMap<String, String>>,
    display_format: MarkdownDisplayFormat,
    link_context: Option<&MarkdownLinkContext<'_>>,
) -> String {
    let items = docs
        .iter()
        .map(|doc| {
            let display_name = file_stem(&doc.file);
            let mut file_name = display_name.clone();

            if let Some(doc_to_file) = doc_to_file {
                if let Some(mapped) = doc_to_file.get(&doc.file) {
                    file_name.clone_from(mapped);
                }
            } else if file_name == "index" {
                file_name = "index-module".to_string();
            }

            let count_label = format_count_label(doc.entries.len(), "symbol", Some("symbols"));
            let href = doc_page_href(options, &file_name, None);
            let summary = clean_summary_text(&process_doc_text(&doc.description, link_context), 88);
            (display_name, href, count_label, summary)
        })
        .collect::<Vec<_>>();

    if display_format == MarkdownDisplayFormat::Table {
        let mut rows = StringBuilder::new();
        for (display_name, href, count_label, summary) in &items {
            if !rows.is_empty() {
                rows.push_char('\n');
            }
            rows.push_str("<tr><td><a href=\"");
            rows.push_str(&escape_html(href));
            rows.push_str("\">");
            rows.push_str(&escape_html(display_name));
            rows.push_str("</a></td><td>");
            rows.push_str(&escape_html(count_label));
            rows.push_str("</td><td>");
            rows.push_str(&render_inline_html(summary));
            rows.push_str("</td></tr>");
        }
        let rows = rows.into_string();

        let mut out = StringBuilder::with_capacity(rows.len() + 150);
        out.push_str(
            "<table class=\"ox-api-modules-table\">
<thead><tr><th>Module</th><th>Symbols</th><th>Description</th></tr></thead>
<tbody>
",
        );
        out.push_str(&rows);
        out.push_str(
            "
</tbody>
</table>

",
        );
        return out.into_string();
    }

    let mut rows = StringBuilder::new();
    for (display_name, href, count_label, summary) in &items {
        if !rows.is_empty() {
            rows.push_char('\n');
        }
        rows.push_str("<li><a href=\"");
        rows.push_str(&escape_html(href));
        rows.push_str("\">");
        rows.push_str(&escape_html(display_name));
        rows.push_str("</a><span class=\"ox-api-module__count\">");
        rows.push_str(&escape_html(count_label));
        rows.push_str("</span>");
        if !summary.is_empty() {
            rows.push_str("<span class=\"ox-api-module__summary\">");
            rows.push_str(&render_inline_html(summary));
            rows.push_str("</span>");
        }
        rows.push_str("</li>");
    }
    let rows = rows.into_string();

    let mut out = StringBuilder::with_capacity(rows.len() + 40);
    out.push_str(
        "<ul class=\"ox-api-modules-list\">
",
    );
    out.push_str(&rows);
    out.push_str(
        "
</ul>

",
    );
    out.into_string()
}
