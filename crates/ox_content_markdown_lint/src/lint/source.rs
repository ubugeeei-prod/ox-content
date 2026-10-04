use super::{MarkdownLintDiagnostic, create_diagnostic};

/// Indexed once per document; Unicode positions never scan from document start.
pub(super) struct Source<'s> {
    pub text: &'s str,
    pub lines: Vec<usize>,
    unicode: Vec<(usize, usize)>,
}

impl<'s> Source<'s> {
    pub fn new(text: &'s str) -> Self {
        let mut lines = vec![0];
        lines.extend(text.bytes().enumerate().filter_map(|(i, b)| (b == b'\n').then_some(i + 1)));
        let mut extra = 0;
        let unicode = text
            .char_indices()
            .filter_map(|(i, c)| {
                if c.is_ascii() {
                    return None;
                }
                extra += c.len_utf8() - c.len_utf16();
                Some((i + c.len_utf8(), extra))
            })
            .collect();
        Self { text, lines, unicode }
    }

    fn extra(&self, offset: usize) -> usize {
        let index = self.unicode.partition_point(|(end, _)| *end <= offset);
        index.checked_sub(1).map_or(0, |i| self.unicode[i].1)
    }

    pub fn position(&self, offset: usize) -> (usize, usize) {
        let offset = offset.min(self.text.len());
        let index = self.lines.partition_point(|start| *start <= offset).saturating_sub(1);
        let start = self.lines[index];
        (index + 1, offset - start - (self.extra(offset) - self.extra(start)) + 1)
    }

    pub fn diagnostic(
        &self,
        rule: &str,
        message: String,
        start: usize,
        end: usize,
    ) -> MarkdownLintDiagnostic {
        let (line, column) = self.position(start);
        let (end_line, end_column) = self.position(end);
        let mut value = create_diagnostic(rule, message, line, column, end_column, None, None);
        value.end_line = end_line as u32;
        value
    }
}

pub(super) fn frontmatter_end(source: &str) -> usize {
    let mut lines = source.split_inclusive('\n');
    let Some(first) = lines.next() else {
        return 0;
    };
    if first.trim_start_matches('\u{feff}').trim() != "---" {
        return 0;
    }
    let mut offset = first.len();
    for line in lines {
        offset += line.len();
        if matches!(line.trim(), "---" | "...") {
            return offset;
        }
    }
    source.len()
}
