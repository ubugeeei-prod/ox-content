use super::Context;
use ox_content_ast::Span;
use regex::Regex;
use std::sync::LazyLock;

static HTML: LazyLock<Option<Regex>> =
    LazyLock::new(|| Regex::new(r"</?([A-Za-z][A-Za-z0-9-]*)\b[^>]*>").ok());
static REVERSED: LazyLock<Option<Regex>> =
    LazyLock::new(|| Regex::new(r"\([^\n()]+\)\[(?:https?://|/)[^\]\n]+\]").ok());
pub(super) fn check(ctx: &mut Context<'_, '_>) {
    let doc = ctx.document;
    for &span in &doc.html {
        let raw = ctx.raw(span);
        if raw.trim_start().starts_with("<!--") {
            continue;
        }
        for found in HTML.as_ref().into_iter().flat_map(|re| re.captures_iter(raw)) {
            if found
                .get(0)
                .is_some_and(|v| super::contains(&doc.comments, span.start + v.start() as u32))
            {
                continue;
            }
            if found.get(0).is_some_and(|v| v.as_str().starts_with("</")) {
                continue;
            }
            let tag = &found[1];
            let table =
                doc.tables.iter().any(|table| table.start <= span.start && span.end <= table.end);
            let allowed = ctx
                .settings
                .strings(33, "allowed_elements")
                .into_iter()
                .flatten()
                .chain(
                    table
                        .then(|| ctx.settings.strings(33, "table_allowed_elements"))
                        .flatten()
                        .into_iter()
                        .flatten(),
                )
                .any(|value| value.as_str().is_some_and(|v| v.eq_ignore_ascii_case(tag)));
            if !allowed && let Some(found) = found.get(0) {
                ctx.report(
                    33,
                    Span::new(span.start + found.start() as u32, span.start + found.end() as u32),
                );
            }
        }
    }
    let mut emphasis = [None, None];
    for &(span, strong) in &doc.emphasis {
        let index = usize::from(strong);
        let marker = if ctx.raw(span).starts_with('*') { "asterisk" } else { "underscore" };
        let initial = *emphasis[index].get_or_insert(marker);
        let rule = if strong { 50 } else { 49 };
        let required = ctx.settings.string(rule, "style", "consistent");
        if marker != if required == "consistent" { initial } else { required } {
            if required == "underscore"
                && (ctx.source.text[..span.start as usize]
                    .chars()
                    .next_back()
                    .is_some_and(|c| c.is_ascii_alphanumeric() || c == '_')
                    || ctx.source.text[span.end as usize..]
                        .chars()
                        .next()
                        .is_some_and(|c| c.is_ascii_alphanumeric() || c == '_'))
            {
                continue;
            }
            let length = if strong { 2 } else { 1 };
            ctx.report(rule, Span::new(span.start, span.start + length));
            ctx.report(rule, Span::new(span.end - length, span.end));
        }
    }
    for &span in &doc.text {
        let raw = ctx.raw(span);
        if ctx.settings.on(11) {
            for found in REVERSED.as_ref().into_iter().flat_map(|re| re.find_iter(raw)) {
                ctx.report(
                    11,
                    Span::new(span.start + found.start() as u32, span.start + found.end() as u32),
                );
            }
        }
    }
    super::emphasis::spaces(ctx);
    for &span in &doc.inline_code {
        let raw = ctx.raw(span);
        let ticks = raw.bytes().take_while(|&c| c == b'`').count();
        if ticks == 0 || raw.len() < ticks * 2 {
            continue;
        }
        let body = &raw[ticks..raw.len() - ticks];
        if body.trim().is_empty() {
            continue;
        }
        let leading = body.len() - body.trim_start_matches([' ', '\t']).len();
        let trailing = body.len() - body.trim_end_matches([' ', '\t']).len();
        let padding =
            usize::from(body.starts_with(' ') && body.ends_with(' ') && !body.trim().is_empty());
        if leading > padding
            && !(padding == 0 && leading == 1 && body.trim_start().starts_with('`'))
        {
            ctx.report(
                38,
                Span::new(span.start + ticks as u32, span.start + (ticks + leading) as u32),
            );
        }
        if trailing > padding && !(padding == 0 && trailing == 1 && body.trim_end().ends_with('`'))
        {
            ctx.report(
                38,
                Span::new(span.end - (ticks + trailing) as u32, span.end - ticks as u32),
            );
        }
    }
    links(ctx);
}

fn links(ctx: &mut Context<'_, '_>) {
    let doc = ctx.document;
    for link in &doc.links {
        let raw = ctx.raw(link.span);
        if !link.image
            && (raw.starts_with("http://")
                || raw.starts_with("https://")
                || raw.starts_with("www.")
                || link.url.starts_with("mailto:") && !raw.starts_with(['<', '[']))
            && !super::contains(&doc.html_contents, link.span.start)
        {
            ctx.report(34, link.span);
        }
        if !link.image && (link.url.is_empty() || link.url == "#") {
            ctx.report(42, link.span);
        }
        if link.image && link.label.trim().is_empty() {
            ctx.report(45, link.span);
        }
        if let Some(text) = raw.strip_prefix("![").or_else(|| raw.strip_prefix('[')) {
            if let Some(end) = label_end(text) {
                let label = &text[..end];
                let leading = label.len() - label.trim_start().len();
                let trailing = label.len() - label.trim_end().len();
                if leading > 0 && !link.image {
                    ctx.report(
                        39,
                        Span::new(link.span.start + 1, link.span.start + 1 + leading as u32),
                    );
                }
                if trailing > 0 && !link.image {
                    ctx.report(
                        39,
                        Span::new(
                            link.span.start + 1 + (end - trailing) as u32,
                            link.span.start + 1 + end as u32,
                        ),
                    );
                }
                let kind = if text[end + 1..].starts_with('(') {
                    "inline"
                } else if text[end + 1..].starts_with("[]") {
                    "collapsed"
                } else if text[end + 1..].starts_with('[') {
                    "full"
                } else {
                    "shortcut"
                };
                let url_inline = kind == "inline" && link.label == link.url && !link.image;
                if !ctx.settings.boolean(54, kind, true)
                    || (url_inline && !ctx.settings.boolean(54, "url_inline", true))
                {
                    ctx.report(54, link.span);
                }
            }
        } else if raw.starts_with('<') && !ctx.settings.boolean(54, "autolink", true) {
            ctx.report(54, link.span);
        }
        if !link.image && ctx.settings.on(59) && !raw.contains(['`', '<']) {
            let label = normalize(link.label.as_str());
            let prohibited = ctx.settings.strings(59, "prohibited_texts");
            let invalid = if let Some(words) = prohibited {
                words
                    .iter()
                    .filter_map(serde_json::Value::as_str)
                    .any(|word| label == normalize(word))
            } else {
                ["click here", "here", "link", "more"].contains(&label.as_str())
            };
            if invalid {
                ctx.report(59, link.span);
            }
        }
    }
}

pub(super) fn label_end(text: &str) -> Option<usize> {
    let mut depth = 0;
    let mut escaped = false;
    for (at, c) in text.char_indices() {
        if escaped {
            escaped = false;
            continue;
        }
        if c == '\\' {
            escaped = true;
            continue;
        }
        if c == '[' {
            depth += 1;
        }
        if c == ']' {
            if depth == 0 {
                return Some(at);
            }
            depth -= 1;
        }
    }
    None
}
pub(super) fn normalize(value: &str) -> compact_str::CompactString {
    let mut out = compact_str::CompactString::default();
    for c in value.chars() {
        if c.is_ascii_alphanumeric() {
            out.push(c.to_ascii_lowercase());
        } else if !out.ends_with(' ') {
            out.push(' ');
        }
    }
    out.truncate(out.trim_end().len());
    if out.starts_with(' ') {
        out.remove(0);
    }
    out
}
