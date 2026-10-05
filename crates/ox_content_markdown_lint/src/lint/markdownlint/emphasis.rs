use super::Context;
use ox_content_ast::Span;

pub(super) fn spaces(ctx: &mut Context<'_, '_>) {
    if !ctx.settings.on(37) {
        return;
    }
    let mut pending = [None; 6];
    let mut block = None;
    for &span in &ctx.document.text {
        let paragraph = ctx.document.paragraphs.partition_point(|p| p.start <= span.start);
        let current = paragraph
            .checked_sub(1)
            .filter(|&i| span.end <= ctx.document.paragraphs[i].end)
            .or_else(|| {
                let heading = ctx.document.headings.partition_point(|h| h.span.start <= span.start);
                heading
                    .checked_sub(1)
                    .filter(|&i| span.end <= ctx.document.headings[i].span.end)
                    .map(|i| i + ctx.document.paragraphs.len())
            });
        if current != block {
            pending.fill(None);
            block = current;
        }
        let raw = ctx.raw(span);
        let bytes = raw.as_bytes();
        let mut at = 0;
        while at < bytes.len() {
            let marker = bytes[at];
            if !matches!(marker, b'*' | b'_') {
                at += 1;
                continue;
            }
            let count = bytes[at..].iter().take_while(|&&c| c == marker).count();
            let escaped = bytes[..at].iter().rev().take_while(|&&c| c == b'\\').count() % 2 != 0;
            if count <= 3 && !escaped {
                let kind = count - 1 + usize::from(marker == b'_') * 3;
                let position = span.start + at as u32;
                if let Some(start) = pending[kind].take() {
                    edge(ctx, start + count as u32, true);
                    edge(ctx, position, false);
                } else {
                    pending[kind] = Some(position);
                }
            }
            at += count;
        }
    }
}

fn edge(ctx: &mut Context<'_, '_>, position: u32, leading: bool) {
    let index = ctx.source.line_index(position as usize);
    let line = ctx.line(index);
    let at = position as usize - ctx.source.lines[index];
    let text = if leading { &line[at..] } else { &line[..at] };
    let trimmed = if leading {
        text.trim_start_matches([' ', '\t'])
    } else {
        text.trim_end_matches([' ', '\t'])
    };
    let count = text.len() - trimmed.len();
    if count > 0 && !trimmed.is_empty() {
        let start = if leading { position } else { position - count as u32 };
        ctx.report(37, Span::new(start, start + count as u32));
    }
}
