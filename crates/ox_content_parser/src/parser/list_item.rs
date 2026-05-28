use super::Parser;

pub(super) struct ParsedListItem<'a> {
    pub(super) ordered: bool,
    pub(super) start: Option<u32>,
    pub(super) content: &'a str,
    pub(super) content_offset: usize,
    pub(super) checked: Option<bool>,
}

impl<'a> Parser<'a> {
    pub(super) fn try_parse_list(&self) -> bool {
        Self::try_parse_list_line(self.current_line().trim_start())
    }

    /// Line-cached variant of [`Self::try_parse_list`]. Callers that
    /// already produced the trimmed line via `parse_block` /
    /// `line_starts_block` reuse that slice instead of re-scanning the
    /// source for a newline and re-running `trim_start`.
    pub(super) fn try_parse_list_line(trimmed: &str) -> bool {
        let trimmed = trimmed.as_bytes();

        // Unordered list: starts with -, *, or + followed by space.
        if trimmed.len() >= 2 && matches!(trimmed[0], b'-' | b'*' | b'+') && trimmed[1] == b' ' {
            return true;
        }

        // Ordered list: digit(s) followed by `.` or `)` and a space.
        let mut i = 0;
        while i < trimmed.len() && trimmed[i].is_ascii_digit() {
            i += 1;
        }
        i > 0
            && i + 1 < trimmed.len()
            && matches!(trimmed[i], b'.' | b')')
            && trimmed[i + 1] == b' '
    }

    /// Calculates the indentation level (number of spaces) of the current line.
    pub(super) fn calc_indentation(&self, start: usize) -> usize {
        let mut indent = 0;
        let bytes = self.source.as_bytes();
        for byte in bytes.iter().skip(start) {
            match byte {
                b' ' => indent += 1,
                b'\t' => indent += 4, // Assume tab is 4 spaces
                _ => break,
            }
        }
        indent
    }

    pub(super) fn parse_task_list_prefix(&self, content: &'a str) -> Option<(bool, usize)> {
        if !self.options.task_lists || content.len() < 3 {
            return None;
        }

        if (content.starts_with("[x]") || content.starts_with("[X]"))
            && (content.len() == 3 || content.starts_with("[x] ") || content.starts_with("[X] "))
        {
            return Some((true, usize::from(content.len() > 3) + 3));
        }

        if content.starts_with("[ ]") && (content.len() == 3 || content.starts_with("[ ] ")) {
            return Some((false, usize::from(content.len() > 3) + 3));
        }

        None
    }

    pub(super) fn parse_list_item_line(&self, line_start: usize) -> Option<ParsedListItem<'a>> {
        let remaining = &self.source[line_start..];
        let line = remaining.lines().next().unwrap_or("");
        self.parse_list_item_line_from_line(line_start, line)
    }

    pub(super) fn parse_list_item_line_from_line(
        &self,
        line_start: usize,
        line: &'a str,
    ) -> Option<ParsedListItem<'a>> {
        let trimmed = line.trim_start();
        let trimmed_offset = line_start + (line.len() - trimmed.len());

        if trimmed.starts_with("- ") || trimmed.starts_with("* ") || trimmed.starts_with("+ ") {
            let mut content = &trimmed[2..];
            let mut content_offset = trimmed_offset + 2;
            let mut checked = None;

            if let Some((done, consumed)) = self.parse_task_list_prefix(content) {
                checked = Some(done);
                content = &content[consumed..];
                content_offset += consumed;
            }

            return Some(ParsedListItem {
                ordered: false,
                start: None,
                content,
                content_offset,
                checked,
            });
        }

        let bytes = trimmed.as_bytes();
        let mut marker_end = 0;
        while marker_end < bytes.len() && bytes[marker_end].is_ascii_digit() {
            marker_end += 1;
        }

        if marker_end == 0 || marker_end + 1 >= bytes.len() {
            return None;
        }

        let marker = bytes[marker_end];
        if !matches!(marker, b'.' | b')') || bytes[marker_end + 1] != b' ' {
            return None;
        }

        Some(ParsedListItem {
            ordered: true,
            start: trimmed[..marker_end].parse().ok(),
            content: &trimmed[marker_end + 2..],
            content_offset: trimmed_offset + marker_end + 2,
            checked: None,
        })
    }

    pub(super) fn strip_indent_columns(line: &str, columns: usize) -> &str {
        let mut consumed = 0;
        let mut byte_index = 0;

        for ch in line.chars() {
            let width = if ch == ' ' {
                1
            } else if ch == '\t' {
                4
            } else {
                break;
            };

            if consumed + width > columns {
                break;
            }

            consumed += width;
            byte_index += ch.len_utf8();
        }

        &line[byte_index..]
    }
}
