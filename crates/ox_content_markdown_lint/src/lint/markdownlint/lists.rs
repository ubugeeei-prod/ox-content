use super::{Context, prefix};
use ox_content_ast::Span;

pub(super) fn check(ctx: &mut Context<'_, '_>) {
    let doc = ctx.document;
    let mut bullet = "";
    let mut sublist_styles = Vec::new();
    let mut parents: Vec<&super::syntax::List> = Vec::new();
    for (list_index, list) in doc.lists.iter().enumerate() {
        while parents.last().is_some_and(|parent| parent.span.end < list.span.end) {
            parents.pop();
        }
        let first_line = ctx.source.line_index(list.span.start as usize);
        let indent = indentation(ctx.line(first_line));
        let all_unordered = !list.ordered && parents.iter().all(|p| !p.ordered);
        let depth = parents.len();
        let expected_indent = depth * ctx.settings.number(7, "indent", 2)
            + if ctx.settings.boolean(7, "start_indented", false) {
                ctx.settings.number(7, "start_indent", ctx.settings.number(7, "indent", 2))
            } else {
                0
            };
        if all_unordered && indent != expected_indent {
            ctx.report(7, ctx.line_span(first_line));
        }
        let items = &doc.list_items[list.items.clone()];
        let mut expected_number = 1;
        let first_number = items.first().and_then(|span| marker(prefix(ctx.raw(*span))).1);
        let ordered_style = ctx.settings.string(29, "style", "one_or_ordered");
        let second_number = items.get(1).and_then(|span| marker(prefix(ctx.raw(*span))).1);
        let incrementing = second_number.is_some_and(|n| n != 1 || first_number == Some(0));
        let ones = ordered_style == "one" || (ordered_style == "one_or_ordered" && !incrementing);
        if ordered_style == "zero" {
            expected_number = 0;
        } else if ordered_style == "ordered" || ordered_style == "one_or_ordered" {
            expected_number = usize::from(!(incrementing && first_number == Some(0)));
        }
        let multiline =
            ctx.source.line_index(list.span.end.saturating_sub(1) as usize) - first_line + 1
                != items.len();
        for &span in items {
            let line_index = ctx.source.line_index(span.start as usize);
            let line = ctx.line(line_index);
            let raw = prefix(ctx.raw(span));
            let (length, number) = marker(raw);
            if length == 0 {
                continue;
            }
            let marker_offset = line.find(&raw[..length]).unwrap_or(0);
            let marker_span = Span::new(
                ctx.line_span(line_index).start + marker_offset as u32,
                ctx.line_span(line_index).start + (marker_offset + length) as u32,
            );
            let item_indent = indentation(line);
            if item_indent != indent {
                ctx.report(5, marker_span);
            }
            if list.ordered {
                let desired = if ones { 1 } else { expected_number };
                if number != Some(desired) {
                    ctx.report(29, marker_span);
                }
                if !ones && ordered_style != "zero" {
                    expected_number += 1;
                }
            } else {
                let current = match raw.as_bytes()[0] {
                    b'*' => "asterisk",
                    b'+' => "plus",
                    _ => "dash",
                };
                if bullet.is_empty() {
                    bullet = current;
                }
                let required = ctx.settings.string(4, "style", "consistent");
                let mismatch = if required == "sublist" {
                    if sublist_styles.len() <= depth {
                        sublist_styles.resize(depth + 1, "");
                    }
                    if sublist_styles[depth].is_empty() {
                        sublist_styles[depth] = current;
                    }
                    current != sublist_styles[depth]
                        || depth > 0 && current == sublist_styles[depth - 1]
                } else {
                    current != if required == "consistent" { bullet } else { required }
                };
                if mismatch {
                    ctx.report(4, marker_span);
                }
            }
            let name = match (list.ordered, multiline) {
                (true, true) => "ol_multi",
                (true, false) => "ol_single",
                (false, true) => "ul_multi",
                (false, false) => "ul_single",
            };
            let spaces = raw[length..]
                .bytes()
                .take_while(|c| c.is_ascii_whitespace() && *c != b'\n' && *c != b'\r')
                .count();
            if spaces != ctx.settings.number(30, name, 1) && !raw[length..].trim().is_empty() {
                ctx.report(30, marker_span);
            }
        }
        if parents.is_empty() {
            ctx.around(32, list.span, 1, 1);
        }
        parents.push(&doc.lists[list_index]);
    }
}

fn indentation(line: &str) -> usize {
    let mut raw = line;
    while let Some(at) = raw.find('>') {
        if !raw[..at].trim().is_empty() {
            break;
        }
        raw = &raw[at + 1..];
        raw = raw.strip_prefix(' ').unwrap_or(raw);
    }
    raw.bytes().take_while(|c| *c == b' ').count()
}
fn marker(line: &str) -> (usize, Option<usize>) {
    if line.starts_with(['*', '+', '-']) {
        return (1, None);
    }
    let digits = line.bytes().take_while(u8::is_ascii_digit).count();
    if digits > 0 && line.as_bytes().get(digits).is_some_and(|c| matches!(c, b'.' | b')')) {
        (digits + 1, line[..digits].parse().ok())
    } else {
        (0, None)
    }
}
