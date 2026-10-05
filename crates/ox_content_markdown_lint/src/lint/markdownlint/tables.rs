use super::{Context, prefix};
use ox_content_ast::Span;
use unicode_width::UnicodeWidthStr;

pub(super) fn check(ctx: &mut Context<'_, '_>) {
    let doc = ctx.document;
    let mut pipes = Vec::new();
    let mut aligned = Vec::new();
    let mut compact = Vec::new();
    let mut tight = Vec::new();
    let mut header_columns = Vec::new();
    let mut matched = Vec::new();
    for &table in &doc.tables {
        ctx.around(58, table, 1, 1);
        let first = ctx.source.line_index(table.start as usize);
        let last = ctx.source.line_index(table.end.saturating_sub(1) as usize);
        let header = content(ctx.line(first)).trim_end();
        dividers(header, &mut pipes);
        let expected_count = columns(header, &pipes);
        let expected_pipes = edges(header, &pipes);
        header_columns.clear();
        let mut previous = 0;
        let mut width = 0;
        for &at in &pipes {
            width += header[previous..at].width();
            header_columns.push(width);
            previous = at;
        }
        aligned.clear();
        compact.clear();
        tight.clear();
        for index in first..=last {
            let line = ctx.line(index);
            let raw = content(line).trim_end();
            let offset = ctx.line_span(index).start + (line.len() - content(line).len()) as u32;
            dividers(raw, &mut pipes);
            if columns(raw, &pipes) != expected_count {
                ctx.report(56, ctx.line_span(index));
            }
            let current = edges(raw, &pipes);
            let desired = match ctx.settings.string(55, "style", "consistent") {
                "leading_and_trailing" => (true, true),
                "leading_only" => (true, false),
                "trailing_only" => (false, true),
                "no_leading_or_trailing" => (false, false),
                _ => expected_pipes,
            };
            if current.0 != desired.0 {
                ctx.report(55, ctx.line_span(index));
            }
            if current.1 != desired.1 {
                ctx.report(55, ctx.line_span(index));
            }
            matched.clear();
            matched.resize(header_columns.len(), false);
            let mut remaining = header_columns.len();
            let mut column = 0;
            let mut previous = 0;
            for &at in &pipes {
                let span = Span::new(offset + at as u32, offset + at as u32 + 1);
                column += raw[previous..at].width();
                previous = at;
                let matches = header_columns.binary_search(&column).ok().filter(|&i| !matched[i]);
                if let Some(index) = matches {
                    matched[index] = true;
                    remaining -= 1;
                }
                if matches.is_none() && remaining > 0 && index > first {
                    aligned.push(span);
                    if index == first + 1 && ctx.settings.boolean(60, "aligned_delimiter", false) {
                        compact.push(span);
                        tight.push(span);
                    }
                }
                if at > 0 {
                    let before = &raw[..at];
                    let spaces = before.len() - before.trim_end_matches([' ', '\t']).len();
                    if spaces != 1 {
                        compact.push(span);
                    }
                    if spaces != 0 {
                        tight.push(span);
                    }
                }
                if at + 1 < raw.len() {
                    let after = &raw[at + 1..];
                    let spaces = after.len() - after.trim_start_matches([' ', '\t']).len();
                    if spaces != 1 {
                        compact.push(span);
                    }
                    if spaces != 0 {
                        tight.push(span);
                    }
                }
            }
        }
        let style = ctx.settings.string(60, "style", "any");
        let best = match style {
            "aligned" => &aligned,
            "compact" => &compact,
            "tight" => &tight,
            _ => {
                [&aligned, &compact, &tight].into_iter().min_by_key(|v| v.len()).unwrap_or(&aligned)
            }
        };
        for &span in best {
            ctx.report(60, span);
        }
    }
}

fn dividers(line: &str, output: &mut Vec<usize>) {
    output.clear();
    let mut escaped = false;
    for (at, byte) in line.bytes().enumerate() {
        if escaped {
            escaped = false;
            continue;
        }
        if byte == b'\\' {
            escaped = true;
        } else if byte == b'|' {
            output.push(at);
        }
    }
}
fn columns(line: &str, pipes: &[usize]) -> usize {
    let edges = edges(line, pipes);
    (pipes.len() + 1).saturating_sub(usize::from(edges.0) + usize::from(edges.1))
}

fn edges(line: &str, pipes: &[usize]) -> (bool, bool) {
    (pipes.first() == Some(&0), pipes.last().is_some_and(|&at| at + 1 == line.len()))
}
fn content(line: &str) -> &str {
    let raw = prefix(line);
    if raw.starts_with(['-', '*', '+'])
        && raw.as_bytes().get(1).is_some_and(u8::is_ascii_whitespace)
    {
        return raw[1..].trim_start();
    }
    let digits = raw.bytes().take_while(u8::is_ascii_digit).count();
    if digits > 0
        && raw.as_bytes().get(digits).is_some_and(|c| matches!(c, b'.' | b')'))
        && raw.as_bytes().get(digits + 1).is_some_and(u8::is_ascii_whitespace)
    {
        return raw[digits + 1..].trim_start();
    }
    raw
}
