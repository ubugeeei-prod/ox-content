use super::{Context, prefix};
use crate::lint::source::frontmatter_end;
use rustc_hash::FxHashSet;
use std::sync::LazyLock;
static PUNCTUATION_ESCAPE: LazyLock<Option<regex::Regex>> = LazyLock::new(|| {
    regex::Regex::new(r"(?:&(?:#[xX]?[0-9a-fA-F]+|[A-Za-z][A-Za-z0-9]*);|:[a-zA-Z0-9_+-]+:)$").ok()
});

pub(super) fn check(ctx: &mut Context<'_, '_>) {
    let doc = ctx.document;
    let mut previous = u8::from(title(ctx, 1));
    let mut style = "";
    let mut single = title(ctx, 25);
    let first_content = first_content(ctx);
    let top_level = ctx.settings.number(25, "level", 1);
    let is_title =
        single
            || doc.headings.iter().find(|h| usize::from(h.level) == top_level).is_some_and(|h| {
                first_content == Some(ctx.source.line_index(h.span.start as usize))
            });
    let mut seen: [FxHashSet<&str>; 7] = std::array::from_fn(|_| FxHashSet::default());
    let mut depth = 1;
    for heading in &doc.headings {
        let raw = prefix(ctx.raw(heading.span)).trim_end();
        let atx = raw.starts_with('#');
        let current = if !atx {
            "setext"
        } else if raw.ends_with('#') && raw.trim_end_matches('#').ends_with(char::is_whitespace) {
            "atx_closed"
        } else {
            "atx"
        };
        if previous > 0 && heading.level > previous + 1 {
            ctx.report(1, heading.span);
        }
        previous = heading.level;
        if style.is_empty() {
            style = current;
        }
        let required = ctx.settings.string(3, "style", "consistent");
        let valid = match required {
            "consistent" => style == current,
            "setext_with_atx" => {
                if heading.level <= 2 {
                    current == "setext"
                } else {
                    current == "atx"
                }
            }
            "setext_with_atx_closed" => {
                if heading.level <= 2 {
                    current == "setext"
                } else {
                    current == "atx_closed"
                }
            }
            _ => required == current,
        };
        if !valid {
            ctx.report(3, heading.span);
        }
        let line = ctx.source.line_index(heading.span.start as usize);
        let before_frontmatter = ctx.source.line_index(frontmatter_end(ctx.source.text));
        let above = if line == before_frontmatter
            && !ctx.settings.boolean(22, "include_front_matter", false)
        {
            0
        } else {
            blanks(ctx, "lines_above", heading.level)
        };
        let below = blanks(ctx, "lines_below", heading.level);
        ctx.around(22, heading.span, above, below);
        let whole_line = ctx.line(line);
        if whole_line.starts_with([' ', '\t']) && !whole_line.trim_start().starts_with('>') {
            ctx.report(23, heading.span);
        }
        let siblings = ctx.settings.boolean(24, "siblings_only", false);
        let level = usize::from(heading.level);
        if siblings {
            for set in seen.iter_mut().skip(level + 1) {
                set.clear();
            }
            if level > depth {
                seen[level].clear();
            }
            depth = level;
        }
        let index = if siblings { level } else { 0 };
        if ctx.settings.on(24) && !seen[index].insert(heading.text.as_str()) {
            ctx.report(24, heading.span);
        }
        if is_title && level == top_level {
            if single {
                ctx.report(25, heading.span);
            }
            single = true;
        }
        let punctuation = ctx.settings.string(26, "punctuation", ".,;:!。，；：！");
        let content = ctx.raw(heading.content).trim_end();
        if content.chars().last().is_some_and(|c| punctuation.contains(c))
            && !PUNCTUATION_ESCAPE.as_ref().is_some_and(|re| re.is_match(content))
        {
            let trimmed = content.trim_end_matches(|c| punctuation.contains(c)).trim_end();
            ctx.report(
                26,
                ox_content_ast::Span::new(
                    heading.content.start + trimmed.len() as u32,
                    heading.content.start + content.len() as u32,
                ),
            );
        }
    }
    if ctx.settings.on(41) && !title(ctx, 41) {
        let first = first_content;
        if let Some(first) = first {
            let required = ctx.settings.number(41, "level", 1);
            let preamble = ctx.settings.boolean(41, "allow_preamble", false);
            let valid = doc.headings.first().is_some_and(|h| {
                usize::from(h.level) == required
                    && (preamble || ctx.source.line_index(h.span.start as usize) == first)
            });
            let raw = ctx.line(first).trim_start();
            let tag = format!("<h{required}");
            let html_heading = raw.get(..tag.len()).is_some_and(|v| v.eq_ignore_ascii_case(&tag))
                && raw
                    .as_bytes()
                    .get(tag.len())
                    .is_some_and(|c| *c == b'>' || c.is_ascii_whitespace());
            if !valid && !html_heading {
                ctx.report(41, ctx.line_span(first));
            }
        }
    }
    required_headings(ctx);
}

fn title(ctx: &Context<'_, '_>, rule: usize) -> bool {
    let end = frontmatter_end(ctx.source.text);
    end > 0
        && ctx.settings.regexes[rule]
            .as_ref()
            .is_some_and(|re| ctx.source.text[..end].lines().skip(1).any(|line| re.is_match(line)))
}
fn first_content(ctx: &Context<'_, '_>) -> Option<usize> {
    let mut comment = false;
    for line in ctx.source.line_index(frontmatter_end(ctx.source.text))..ctx.line_count() {
        let mut text = ctx.line(line).trim();
        loop {
            if comment {
                let Some(end) = text.find("-->") else {
                    break;
                };
                text = text[end + 3..].trim_start();
                comment = false;
            } else if let Some(rest) = text.strip_prefix("<!--") {
                text = rest;
                comment = true;
            } else {
                if !text.is_empty() {
                    return Some(line);
                }
                break;
            }
        }
    }
    None
}
fn blanks(ctx: &Context<'_, '_>, key: &str, level: u8) -> usize {
    let value = &ctx.settings.rules[22][key];
    let value = if value.is_array() { &value[usize::from(level) - 1] } else { value };
    value.as_i64().map_or(1, |v| usize::try_from(v.max(0)).unwrap_or(usize::MAX))
}
fn required_headings(ctx: &mut Context<'_, '_>) {
    let doc = ctx.document;
    let Some(pattern) = ctx.settings.strings(43, "headings") else {
        return;
    };
    let case = ctx.settings.boolean(43, "match_case", false);
    let matches = |text: &str, expected: &str| {
        if case { text == expected } else { text.to_lowercase() == expected.to_lowercase() }
    };
    let mut expected = 0;
    let mut wildcard = false;
    for heading in &doc.headings {
        let text = format!("{} {}", "#".repeat(heading.level.into()), heading.text);
        let current = pattern.get(expected).and_then(serde_json::Value::as_str).unwrap_or("[None]");
        expected += 1;
        match current {
            "*" => {
                let next =
                    pattern.get(expected).and_then(serde_json::Value::as_str).unwrap_or("[None]");
                if matches(&text, next) {
                    expected += 1;
                } else {
                    wildcard = true;
                }
            }
            "+" => wildcard = true,
            "?" => {}
            _ if matches(&text, current) => wildcard = false,
            _ if wildcard => expected -= 1,
            _ => {
                ctx.report(43, heading.span);
                return;
            }
        }
    }
    let remaining = pattern.len().saturating_sub(expected);
    if (remaining > 1 || (remaining == 1 && pattern[expected] != "*"))
        && (!doc.headings.is_empty() || !pattern.iter().all(|v| v == "*"))
    {
        ctx.report(43, ctx.line_span(ctx.source.lines.len().saturating_sub(1)));
    }
}
