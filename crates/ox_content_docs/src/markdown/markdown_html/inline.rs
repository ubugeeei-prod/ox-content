use rustc_hash::FxHashSet;
use std::sync::OnceLock;

use super::super::{
    cached_regex, collapse_type_annotation_whitespace, process_doc_text, resolve_type_fragments,
    MarkdownLinkContext, RegexCache, TypeFragment,
};
use crate::string_builder::StringBuilder;

pub(super) fn escape_html(value: &str) -> String {
    // Most inputs here are symbol names, type annotations, and kind labels,
    // which usually contain no escapable HTML bytes. The early scan keeps that
    // path to one pass plus `to_string()`, while the escaping path still does a
    // single allocation sized near the input.
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

pub(super) fn render_doc_inline_html(
    text: &str,
    context: Option<&MarkdownLinkContext<'_>>,
) -> String {
    render_inline_html(&process_doc_text(text, context))
}

fn replace_line_breaks_html(html: String) -> String {
    if html.contains('\n') {
        html.replace('\n', "<br>")
    } else {
        html
    }
}

pub(super) fn render_inline_html(text: &str) -> String {
    static TOKEN_RE: RegexCache = OnceLock::new();

    let Some(token_re) = cached_regex(
        &TOKEN_RE,
        r"`([^`]+)`|\[([^\]]+)\]\(([^)]+)\)|\*\*([^*]+)\*\*|__([^_]+)__|\*([^*]+)\*|_([^_]+)_",
    ) else {
        return replace_line_breaks_html(escape_html(text));
    };
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

pub(super) fn render_code_block_html(code: &str, language: &str) -> String {
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

pub(super) fn render_highlighted_inline_code_html(
    code: &str,
    class_name: &str,
    language: &str,
) -> String {
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

/// Inner HTML for a TypeScript type annotation: escaped text with `<a>` anchors
/// for known symbols. Type annotations are always rendered as inline code, so
/// multiline generic formatting is collapsed before link resolution.
pub(super) fn render_type_inner_html(
    value: &str,
    context: Option<&MarkdownLinkContext<'_>>,
    skip: &FxHashSet<&str>,
) -> String {
    let value = collapse_type_annotation_whitespace(value);
    match resolve_type_fragments(&value, context, skip) {
        None => escape_html(&value),
        Some(fragments) => {
            let mut out = String::new();
            for fragment in fragments {
                match fragment {
                    TypeFragment::Text(text) | TypeFragment::Code(text) => {
                        out.push_str(&escape_html(&text));
                    }
                    TypeFragment::Link { name, href } => {
                        out.push_str("<a href=\"");
                        out.push_str(&escape_html(&href));
                        out.push_str("\">");
                        out.push_str(&escape_html(&name));
                        out.push_str("</a>");
                    }
                }
            }
            out
        }
    }
}
