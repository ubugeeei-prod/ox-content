use std::path::Path;

use tower_lsp::lsp_types::{Position, Range};

#[derive(Clone, Debug)]
pub struct TextDocumentState {
    text: String,
    line_offsets: Vec<usize>,
}

impl TextDocumentState {
    #[must_use]
    pub fn new(text: String) -> Self {
        let mut line_offsets = vec![0];
        for (index, ch) in text.char_indices() {
            if ch == '\n' {
                line_offsets.push(index + 1);
            }
        }
        Self { text, line_offsets }
    }

    #[must_use]
    pub fn text(&self) -> &str {
        &self.text
    }

    #[must_use]
    pub fn line_count(&self) -> usize {
        self.line_offsets.len()
    }

    #[must_use]
    pub fn line_start_offset(&self, line: usize) -> usize {
        self.line_offsets.get(line).copied().unwrap_or(self.text.len())
    }

    #[must_use]
    pub fn line_end_offset(&self, line: usize) -> usize {
        if line + 1 < self.line_offsets.len() {
            self.line_offsets[line + 1]
        } else {
            self.text.len()
        }
    }

    #[must_use]
    pub fn line_text(&self, line: u32) -> &str {
        let line = line as usize;
        if line >= self.line_offsets.len() {
            return "";
        }
        let start = self.line_start_offset(line);
        let end = self.line_end_offset(line);
        &self.text[start..end]
    }

    #[must_use]
    pub fn position_to_offset(&self, position: Position) -> usize {
        if self.line_offsets.is_empty() {
            return 0;
        }

        let line = (position.line as usize).min(self.line_offsets.len().saturating_sub(1));
        let start = self.line_start_offset(line);
        let end = self.line_end_offset(line);
        let line_text = &self.text[start..end];

        let mut utf16_offset = 0usize;
        let mut byte_offset = start;

        for (index, ch) in line_text.char_indices() {
            let width = ch.len_utf16();
            if utf16_offset + width > position.character as usize {
                return start + index;
            }
            utf16_offset += width;
            byte_offset = start + index + ch.len_utf8();
        }

        byte_offset.min(self.text.len())
    }

    #[must_use]
    pub fn offset_to_position(&self, offset: usize) -> Position {
        let clamped = offset.min(self.text.len());
        let line_index = self
            .line_offsets
            .partition_point(|line_offset| *line_offset <= clamped)
            .saturating_sub(1);

        let start = self.line_start_offset(line_index);
        let character = self.text[start..clamped].encode_utf16().count() as u32;

        Position { line: line_index as u32, character }
    }

    #[must_use]
    pub fn range_from_offsets(&self, start: usize, end: usize) -> Range {
        Range { start: self.offset_to_position(start), end: self.offset_to_position(end) }
    }

    #[must_use]
    pub fn word_range_at(&self, position: Position, predicate: fn(char) -> bool) -> Range {
        let offset = self.position_to_offset(position);
        let line_start = self.line_start_offset(position.line as usize);
        let line_end = self.line_end_offset(position.line as usize);
        let line_text = &self.text[line_start..line_end];
        let local_offset = offset.saturating_sub(line_start).min(line_text.len());

        let mut start = local_offset;
        for (index, ch) in line_text[..local_offset].char_indices().rev() {
            if predicate(ch) {
                start = index;
            } else {
                break;
            }
        }

        let mut end = local_offset;
        for (index, ch) in line_text[local_offset..].char_indices() {
            if predicate(ch) {
                end = local_offset + index + ch.len_utf8();
            } else {
                break;
            }
        }

        self.range_from_offsets(line_start + start, line_start + end)
    }
}

#[must_use]
pub fn is_markdown_path(path: &Path) -> bool {
    matches!(
        path.extension().and_then(|ext| ext.to_str()),
        Some("md" | "markdown" | "mdown" | "mdc")
    )
}
