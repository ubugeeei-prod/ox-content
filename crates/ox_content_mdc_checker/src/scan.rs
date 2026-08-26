use super::{Diagnostic, Severity};

pub struct LineIndex {
    starts: Vec<usize>,
}

impl LineIndex {
    pub fn new(source: &str) -> Self {
        let mut starts = vec![0];
        for (index, ch) in source.char_indices() {
            if ch == '\n' {
                starts.push(index + 1);
            }
        }
        Self { starts }
    }

    pub fn position(&self, offset: usize) -> (u32, u32) {
        let line = self.starts.partition_point(|start| *start <= offset).saturating_sub(1);
        let column = offset.saturating_sub(self.starts[line]);
        (line as u32 + 1, column as u32 + 1)
    }
}

pub fn masked_fence_ranges(source: &str) -> Vec<(usize, usize)> {
    let mut ranges = Vec::new();
    let mut cursor = 0;
    let mut fence: Option<(u8, usize)> = None;

    for line in source.split_inclusive('\n') {
        let line_start = cursor;
        let trimmed = line.trim_start();
        let indent = line.len() - trimmed.len();
        if indent <= 3 && (trimmed.starts_with("```") || trimmed.starts_with("~~~")) {
            let marker = trimmed.as_bytes()[0];
            if let Some((open_marker, start)) = fence {
                if marker == open_marker {
                    ranges.push((start, line_start + line.len()));
                    fence = None;
                }
            } else {
                fence = Some((marker, line_start));
            }
        }
        cursor += line.len();
    }

    if let Some((_, start)) = fence {
        ranges.push((start, source.len()));
    }

    ranges
}

pub fn is_masked(offset: usize, ranges: &[(usize, usize)]) -> bool {
    ranges.iter().any(|(start, end)| offset >= *start && offset < *end)
}

pub fn diagnostic(
    line_index: &LineIndex,
    start: usize,
    end: usize,
    code: &'static str,
    message: String,
    component: Option<String>,
) -> Diagnostic {
    let (line, column) = line_index.position(start);
    let (end_line, end_column) = line_index.position(end);
    Diagnostic {
        severity: Severity::Error,
        code: code.to_string(),
        message,
        line,
        column,
        end_line,
        end_column,
        component,
    }
}
