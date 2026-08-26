use std::path::Path;

use tower_lsp::lsp_types::{Position, Range, TextDocumentContentChangeEvent};

#[derive(Clone, Debug)]
pub struct TextDocumentState {
    text: String,
    line_offsets: Vec<usize>,
    version: i32,
}

impl TextDocumentState {
    #[must_use]
    pub fn new(text: String) -> Self {
        Self::with_version(text, 0)
    }

    #[must_use]
    pub fn with_version(text: String, version: i32) -> Self {
        let line_offsets = line_offsets(&text);
        Self { text, line_offsets, version }
    }

    #[must_use]
    pub fn text(&self) -> &str {
        &self.text
    }

    #[must_use]
    pub fn version(&self) -> i32 {
        self.version
    }

    pub fn apply_changes(&mut self, changes: &[TextDocumentContentChangeEvent], version: i32) {
        for change in changes {
            self.apply_change(change);
        }
        self.version = version;
    }

    fn apply_change(&mut self, change: &TextDocumentContentChangeEvent) {
        match change.range {
            Some(range) => {
                let start = self.position_to_offset(range.start);
                let end = self.position_to_offset(range.end).max(start);
                self.text.replace_range(start..end, &change.text);
            }
            None => self.text.clone_from(&change.text),
        }
        self.line_offsets = line_offsets(&self.text);
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
    path.extension().and_then(|extension| extension.to_str()).is_some_and(|extension| {
        ["md", "markdown", "mdown", "mdc", "mdx"]
            .iter()
            .any(|candidate| extension.eq_ignore_ascii_case(candidate))
    })
}

fn line_offsets(text: &str) -> Vec<usize> {
    let mut offsets = vec![0];
    for (index, ch) in text.char_indices() {
        if ch == '\n' {
            offsets.push(index + 1);
        }
    }
    offsets
}

#[must_use]
pub fn is_mdx_path(path: &Path) -> bool {
    path.extension()
        .and_then(|extension| extension.to_str())
        .is_some_and(|extension| extension.eq_ignore_ascii_case("mdx"))
}

#[cfg(test)]
mod path_tests {
    use super::*;

    #[test]
    fn markdown_paths_include_mdx_case_insensitively() {
        assert!(is_markdown_path(Path::new("guide.mdx")));
        assert!(is_markdown_path(Path::new("guide.MDX")));
        assert!(is_mdx_path(Path::new("guide.MDX")));
        assert!(!is_mdx_path(Path::new("guide.md")));
    }

    #[test]
    fn incremental_change_updates_text_and_version() {
        let mut document = TextDocumentState::with_version("See [missing](./a.md).\n".into(), 1);
        document.apply_changes(
            &[TextDocumentContentChangeEvent {
                range: Some(Range {
                    start: Position { line: 0, character: 14 },
                    end: Position { line: 0, character: 20 },
                }),
                range_length: None,
                text: "./b.md".into(),
            }],
            2,
        );
        assert_eq!(document.version(), 2);
        assert_eq!(document.text(), "See [missing](./b.md).\n");
    }

    #[test]
    fn full_change_replaces_the_document() {
        let mut document = TextDocumentState::with_version("old\n".into(), 1);
        document.apply_changes(
            &[TextDocumentContentChangeEvent {
                range: None,
                range_length: None,
                text: "# New\n".into(),
            }],
            3,
        );
        assert_eq!(document.version(), 3);
        assert_eq!(document.text(), "# New\n");
        assert_eq!(document.line_count(), 2);
    }
}
