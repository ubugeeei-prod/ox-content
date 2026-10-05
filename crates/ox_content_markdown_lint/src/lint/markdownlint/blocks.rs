use super::{Context, prefix};
use ox_content_ast::Span;

pub(super) fn check(ctx: &mut Context<'_, '_>) {
    let doc = ctx.document;
    let mut block_style = "";
    let mut fence_style = "";
    for code in &doc.codes {
        let raw = ctx.raw(code.span);
        let first = prefix(raw.lines().next().unwrap_or(""));
        let marker = first.bytes().next().unwrap_or(0);
        let fenced = code.fenced;
        let current = if fenced { "fenced" } else { "indented" };
        if block_style.is_empty() {
            block_style = current;
        }
        let style = ctx.settings.string(46, "style", "consistent");
        if current != if style == "consistent" { block_style } else { style } {
            ctx.report(46, code.span);
        }
        if !fenced {
            continue;
        }
        let current = if marker == b'`' { "backtick" } else { "tilde" };
        if fence_style.is_empty() {
            fence_style = current;
        }
        let style = ctx.settings.string(48, "style", "consistent");
        if current != if style == "consistent" { fence_style } else { style } {
            ctx.report(48, code.span);
        }
        let first_line = ctx.source.line_index(code.span.start as usize);
        let info = first.trim_start_matches(['`', '~']).trim();
        let language = info.split_whitespace().next().unwrap_or("");
        if language.is_empty()
            || ctx.settings.strings(40, "allowed_languages").is_some_and(|allowed| {
                !allowed.is_empty() && !allowed.iter().any(|v| v.as_str() == Some(language))
            })
        {
            ctx.report(40, ctx.line_span(first_line));
        }
        if ctx.settings.boolean(40, "language_only", false) && info != language {
            ctx.report(40, ctx.line_span(first_line));
        }
        let list_item = super::contains(&doc.list_ranges, code.span.start);
        if !list_item || ctx.settings.boolean(31, "list_items", true) {
            ctx.around(31, code.span, 1, 1);
        }
        let body_start = first_line + 1;
        let commands_only = (body_start..body_start + code.body_lines).all(|i| {
            let line = prefix(ctx.line(i)).trim();
            line.is_empty() || line.starts_with("$ ")
        });
        if commands_only {
            for i in body_start..body_start + code.body_lines {
                if prefix(ctx.line(i)).trim().starts_with("$ ") {
                    ctx.report(14, ctx.line_span(i));
                }
            }
        }
    }
    let mut rule_style = "";
    for &span in &doc.thematic {
        let raw = prefix(ctx.raw(span)).trim();
        if rule_style.is_empty() {
            rule_style = raw;
        }
        let style = ctx.settings.string(35, "style", "consistent");
        if raw != if style == "consistent" { rule_style } else { style } {
            ctx.report(35, span);
        }
    }
    if ctx.settings.on(27) {
        for i in 0..ctx.line_count() {
            let line = ctx.line(i);
            if ctx
                .code_at(ctx.line_span(i).end.saturating_sub(1) as usize)
                .is_some_and(|code| !code.fenced)
            {
                continue;
            }
            let in_list = super::contains(&doc.list_ranges, ctx.line_span(i).end.saturating_sub(1));
            if in_list && !ctx.settings.boolean(27, "list_items", true) {
                continue;
            }
            for (at, _) in line.match_indices('>') {
                if !line[..at].chars().all(|c| matches!(c, ' ' | '\t' | '>')) {
                    break;
                }
                let after = &line[at + 1..];
                let spaces = after.bytes().take_while(|&c| c == b' ').count();
                if spaces > 1 && !after.trim().is_empty() {
                    let start = ctx.line_span(i).start + at as u32 + 1;
                    ctx.report(27, Span::new(start, start + spaces as u32));
                }
            }
        }
    }
    for pair in doc.quotes.windows(2) {
        let end = ctx.source.line_index(pair[0].end.saturating_sub(1) as usize);
        let start = ctx.source.line_index(pair[1].start as usize);
        if start > end + 1 && (end + 1..start).all(|i| ctx.line(i).trim().is_empty()) {
            for i in end + 1..start {
                ctx.report(28, ctx.line_span(i));
            }
        }
    }
    emphasis_heading(ctx);
}

fn emphasis_heading(ctx: &mut Context<'_, '_>) {
    let doc = ctx.document;
    for &span in &doc.paragraphs {
        if super::contains(&doc.list_ranges, span.start) {
            continue;
        }
        let raw = ctx.raw(span).trim();
        if raw.contains('\n') {
            continue;
        }
        let index = doc.emphasis.partition_point(|&(candidate, _)| candidate.start < span.start);
        if let Some(&(emphasis, _)) =
            doc.emphasis.get(index).filter(|&&(candidate, _)| candidate.end <= span.end)
        {
            let text = ctx.raw(emphasis);
            let label = text.trim_matches(['*', '_']);
            if text == raw
                && !label.is_empty()
                && label.chars().last().is_some_and(|c| {
                    !ctx.settings.string(36, "punctuation", ".,;:!?。，；：！？").contains(c)
                })
            {
                ctx.report(36, span);
            }
        }
    }
}
