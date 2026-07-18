use super::Parser;

pub(super) struct ParsedListItem<'a> {
    pub(super) ordered: bool,
    /// Marker byte identifying the list flavor: the bullet (`-`, `*`,
    /// `+`) for unordered items, the delimiter (`.`, `)`) for ordered
    /// ones. Lists only continue across items with the same marker.
    pub(super) marker: u8,
    pub(super) start: Option<u32>,
    pub(super) content: &'a str,
    pub(super) content_offset: usize,
    /// Column (relative to the marker line's start) where continuation
    /// lines must be indented to belong to this item: marker indent +
    /// marker width + following spaces (one column when the item is
    /// empty or starts with indented code).
    pub(super) content_indent: usize,
    pub(super) checked: Option<bool>,
}

impl<'a> Parser<'a> {
    /// Checks whether a trimmed line opens a list item. Callers that
    /// already produced the trimmed line via `parse_block` /
    /// `line_starts_block` reuse that slice instead of re-scanning the
    /// source for a newline and re-running `trim_start`.
    pub(super) fn try_parse_list_line(trimmed: &str) -> bool {
        let bytes = trimmed.as_bytes();

        // Unordered: bullet followed by space/tab or end of line.
        if matches!(bytes.first(), Some(b'-' | b'*' | b'+')) {
            return matches!(bytes.get(1), None | Some(b' ' | b'\t'));
        }

        // Ordered: up to nine digits, `.` or `)`, then space/tab or EOL.
        let mut i = 0;
        while i < bytes.len() && bytes[i].is_ascii_digit() {
            i += 1;
        }
        (1..=9).contains(&i)
            && matches!(bytes.get(i), Some(b'.' | b')'))
            && matches!(bytes.get(i + 1), None | Some(b' ' | b'\t'))
    }

    /// Variant for paragraph interruption: only non-empty items can
    /// interrupt a paragraph, and ordered ones only when numbered 1.
    pub(super) fn try_parse_list_interrupt(trimmed: &str) -> bool {
        let bytes = trimmed.as_bytes();
        if matches!(bytes.first(), Some(b'-' | b'*' | b'+')) {
            return matches!(bytes.get(1), Some(b' ' | b'\t')) && !trimmed[1..].trim().is_empty();
        }
        let mut i = 0;
        while i < bytes.len() && bytes[i].is_ascii_digit() {
            i += 1;
        }
        i > 0
            && trimmed[..i] == *"1"
            && matches!(bytes.get(i), Some(b'.' | b')'))
            && matches!(bytes.get(i + 1), Some(b' ' | b'\t'))
            && !trimmed[i + 1..].trim().is_empty()
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
        let bytes = trimmed.as_bytes();

        let (ordered, marker, marker_width, start) =
            if matches!(bytes.first(), Some(b'-' | b'*' | b'+')) {
                (false, bytes[0], 1, None)
            } else {
                let mut digits = 0;
                while digits < bytes.len() && bytes[digits].is_ascii_digit() {
                    digits += 1;
                }
                if !(1..=9).contains(&digits) || !matches!(bytes.get(digits), Some(b'.' | b')')) {
                    return None;
                }
                (true, bytes[digits], digits + 1, trimmed[..digits].parse().ok())
            };

        // Content begins after 1–4 spaces (or a tab). More than four
        // spaces means the item starts with indented code: content is
        // taken to start one column after the marker, keeping the extra
        // spaces. A bare marker at end of line is an empty item.
        let after_marker = &bytes[marker_width..];
        let spaces = after_marker.iter().take_while(|&&byte| byte == b' ').count();
        let content_skip = match after_marker.first() {
            None => 0,
            Some(b'\t') => 1,
            Some(b' ') if spaces <= 4 && spaces < after_marker.len() => spaces,
            Some(b' ') if spaces >= after_marker.len() => spaces, // marker + trailing blanks
            Some(b' ') => 1,
            Some(_) => return None,
        };

        let marker_indent = line.len() - trimmed.len();
        let ws_run = after_marker.iter().take_while(|&&byte| matches!(byte, b' ' | b'\t')).count();
        if after_marker[..ws_run].contains(&b'\t') {
            // Tabs after the marker expand from the marker's original
            // column; everything beyond the single separator column
            // becomes content spaces so alignment survives the item
            // re-parse (`-\t\tfoo` is an item holding two-space-indented
            // code).
            let marker_end_col = marker_indent + marker_width;
            let mut end_col = marker_end_col;
            for &byte in &after_marker[..ws_run] {
                end_col = if byte == b'\t' { (end_col / 4 + 1) * 4 } else { end_col + 1 };
            }
            let extra_columns = end_col.saturating_sub(marker_end_col + 1);
            let rest = &trimmed[marker_width + ws_run..];
            let mut expanded = self.allocator.new_string();
            for _ in 0..extra_columns {
                expanded.push(' ');
            }
            expanded.push_str(rest);
            let content: &'a str = expanded.into_bump_str();
            return Some(ParsedListItem {
                ordered,
                marker,
                start,
                content,
                content_offset: trimmed_offset + marker_width + 1,
                content_indent: marker_end_col + 1,
                checked: None,
            });
        }

        let mut content = &trimmed[marker_width + content_skip..];
        let mut content_offset = trimmed_offset + marker_width + content_skip;
        // Continuation indent counts the marker's own indent plus the
        // marker and its separating spaces; empty items count one column.
        let content_indent = marker_indent
            + marker_width
            + if content.trim().is_empty() { 1 } else { content_skip.max(1) };
        let mut checked = None;

        if let Some((done, consumed)) = self.parse_task_list_prefix(content) {
            checked = Some(done);
            content = &content[consumed..];
            content_offset += consumed;
        }

        Some(ParsedListItem {
            ordered,
            marker,
            start,
            content,
            content_offset,
            content_indent,
            checked,
        })
    }

    /// Pushes `line` minus its first `columns` columns onto `out`,
    /// expanding the whole leading whitespace run to spaces so tab stops
    /// keep their original alignment through item/quote re-parsing.
    pub(super) fn push_line_without_indent(
        out: &mut ox_content_allocator::String<'a>,
        line: &str,
        columns: usize,
    ) {
        let bytes = line.as_bytes();
        let mut col = 0usize;
        let mut i = 0usize;
        while i < bytes.len() {
            match bytes[i] {
                b' ' => {
                    col += 1;
                    i += 1;
                }
                b'\t' => {
                    col = (col / 4 + 1) * 4;
                    i += 1;
                }
                _ => break,
            }
        }
        for _ in columns..col {
            out.push(' ');
        }
        out.push_str(&line[i..]);
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
