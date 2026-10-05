use super::{Context, prefix};
use ox_content_ast::Span;

pub(super) fn check(ctx: &mut Context<'_, '_>) {
    let doc = ctx.document;
    let mut blanks = 0;
    let base = crate::lint::source::frontmatter_end(ctx.source.text);
    for line_index in ctx.source.line_index(base)..ctx.line_count() {
        let line = ctx.line(line_index);
        let span = ctx.line_span(line_index);
        let code = ctx.code_at(span.end.saturating_sub(1) as usize);
        let in_code = code.is_some();
        let trailing = line.len() - line.trim_end_matches(char::is_whitespace).len();
        let body = code.is_some_and(|code| {
            !code.fenced
                || (line_index > ctx.source.line_index(code.span.start as usize)
                    && line_index < ctx.source.line_index(code.span.end.saturating_sub(1) as usize))
        });
        if trailing > 0 && (!body || ctx.settings.boolean(9, "code_blocks", false)) {
            let breaks = ctx.settings.number(9, "br_spaces", 2);
            let units = line[line.len() - trailing..].encode_utf16().count();
            let allowed = breaks >= 2 && units == breaks;
            let strict = ctx.settings.boolean(9, "strict", false);
            let list_blank = line.trim().is_empty()
                && ctx.settings.boolean(9, "list_item_empty_lines", false)
                && doc
                    .lists
                    .iter()
                    .any(|list| list.span.start <= span.start && span.end <= list.span.end);
            let paragraph = doc.paragraphs.partition_point(|p| p.start <= span.end);
            let next = paragraph.checked_sub(1).is_some_and(|i| {
                line_index < ctx.source.line_index(doc.paragraphs[i].end.saturating_sub(1) as usize)
                    && span.start < doc.paragraphs[i].end
            }) && !doc.inline_code.iter().any(|p| {
                p.start <= span.end
                    && line_index < ctx.source.line_index(p.end.saturating_sub(1) as usize)
            });
            if !list_blank && !(allowed && (!strict || next)) {
                let edit = Span::new(span.end - trailing as u32, span.end);
                let hard_break = doc.breaks.iter().any(|break_| break_.start == span.end);
                let preserve = if hard_break && breaks >= 2 && line.ends_with("  ") {
                    2.min(trailing)
                } else {
                    0
                };
                ctx.replace(9, edit, Span::new(edit.start, edit.end - preserve as u32), "");
            }
        }
        let ignore_tabs = in_code
            && (!ctx.settings.boolean(10, "code_blocks", true)
                || code.is_some_and(|code| {
                    ctx.settings.strings(10, "ignore_code_languages").is_some_and(|languages| {
                        languages.iter().any(|v| {
                            v.as_str()
                                .is_some_and(|v| v.eq_ignore_ascii_case(code.language.as_str()))
                        })
                    })
                }));
        if !ignore_tabs && ctx.settings.on(10) {
            for (at, _) in line.match_indices('\t') {
                let position = span.start + at as u32;
                let inline = doc
                    .inline_code
                    .iter()
                    .any(|code| code.start <= position && position < code.end);
                if !inline || ctx.settings.boolean(10, "code_blocks", true) {
                    ctx.report(10, Span::new(position, position + 1));
                }
            }
        }
        if !in_code && line.trim().is_empty() {
            blanks += 1;
            if blanks > ctx.settings.number(12, "maximum", 1) {
                let next =
                    ctx.source.lines.get(line_index + 1).copied().unwrap_or(ctx.source.text.len());
                ctx.replace(12, span, Span::new(span.start, next as u32), "");
            }
        } else {
            blanks = 0;
        }
        long_line(ctx, line_index, in_code);
        if !in_code {
            hashes(ctx, line_index);
        }
    }
    if !ctx.source.text.is_empty() && !ctx.source.text.ends_with('\n') {
        let end = ctx.source.text.len() as u32;
        let at = ctx.source.text.char_indices().next_back().map_or(end, |(i, _)| i as u32);
        ctx.replace(47, Span::new(at, end), Span::new(end, end), ctx.new_line());
    }
}

fn long_line(ctx: &mut Context<'_, '_>, index: usize, code: bool) {
    if !ctx.settings.on(13) || (code && !ctx.settings.boolean(13, "code_blocks", true)) {
        return;
    }
    let doc = ctx.document;
    let span = ctx.line_span(index);
    let point = span.end.saturating_sub(1);
    let heading = super::at(&doc.headings, point, |h| h.span).is_some();
    let table = super::contains(&doc.tables, point);
    if (heading && !ctx.settings.boolean(13, "headings", true))
        || (table && !ctx.settings.boolean(13, "tables", true))
    {
        return;
    }
    if super::at(&doc.definitions, point, |s| s.span).is_some() {
        return;
    }
    let link_start = doc.links.partition_point(|link| link.span.start < span.start);
    if !ctx.settings.boolean(13, "strict", false)
        && doc.links[link_start..].iter().take_while(|link| link.span.start <= span.end).any(
            |link| {
                link.span.start >= span.start
                    && link.span.end <= span.end
                    && ctx
                        .raw(Span::new(span.start, link.span.start))
                        .trim_matches([' ', '*', '_'])
                        .is_empty()
                    && ctx
                        .raw(Span::new(link.span.end, span.end))
                        .trim_matches([' ', '*', '_'])
                        .is_empty()
            },
        )
    {
        return;
    }
    let line = ctx.line(index);
    if !ctx.settings.boolean(13, "strict", false)
        && line.trim_start().starts_with("<!--")
        && line.trim_end().ends_with("-->")
    {
        return;
    }
    let default = ctx.settings.number(13, "line_length", 80);
    let maximum = if heading {
        ctx.settings.number(13, "heading_line_length", default)
    } else if code {
        ctx.settings.number(13, "code_block_line_length", default)
    } else {
        default
    };
    let length = if line.is_ascii() { line.len() } else { line.encode_utf16().count() };
    if length <= maximum {
        return;
    }
    let at = if line.is_ascii() {
        maximum
    } else {
        let mut units = 0;
        line.char_indices()
            .find_map(|(at, c)| {
                if units >= maximum {
                    Some(at)
                } else {
                    units += c.len_utf16();
                    None
                }
            })
            .unwrap_or(line.len())
    };
    let strict = ctx.settings.boolean(13, "strict", false);
    let stern = ctx.settings.boolean(13, "stern", false);
    if strict
        || (stern && line.trim().contains(char::is_whitespace))
        || (!stern && line[at..].contains(char::is_whitespace))
    {
        ctx.report(13, Span::new(span.start + at as u32, span.end));
    }
}

fn hashes(ctx: &mut Context<'_, '_>, index: usize) {
    let line = ctx.line(index);
    let raw = prefix(line).trim_end();
    let start = ctx.line_span(index).start + (line.len() - prefix(line).len()) as u32;
    let count = raw.bytes().take_while(|&c| c == b'#').count();
    if !(1..=6).contains(&count) || raw.len() <= count {
        return;
    }
    let closing = raw.bytes().rev().take_while(|&c| c == b'#').count();
    let spaces = raw[count..].bytes().take_while(u8::is_ascii_whitespace).count();
    let closed = closing > 0
        && !raw.ends_with("\\#")
        && raw.len() > count + closing
        && (spaces == 0 || raw.as_bytes()[raw.len() - closing - 1].is_ascii_whitespace());
    if spaces == 0 {
        ctx.report(if closed { 20 } else { 18 }, Span::new(start, start + raw.len() as u32));
    }
    if spaces > 1 {
        let edit = Span::new(start + count as u32, start + (count + spaces) as u32);
        ctx.replace(if closed { 21 } else { 19 }, edit, edit, " ");
    }
    if closed {
        let before = &raw[..raw.len() - closing];
        let spaces = before.len() - before.trim_end_matches([' ', '\t']).len();
        if spaces == 0 && raw[count..].starts_with(char::is_whitespace) {
            ctx.report(20, Span::new(start, start + raw.len() as u32));
        }
        if spaces > 1 {
            let end = start + before.len() as u32;
            ctx.replace(
                21,
                Span::new(end - spaces as u32, end),
                Span::new(end - spaces as u32, end),
                " ",
            );
        }
    }
}
