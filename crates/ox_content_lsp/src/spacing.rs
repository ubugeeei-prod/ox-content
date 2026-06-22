use serde::{Deserialize, Serialize};
use tower_lsp::lsp_types::{Diagnostic, DiagnosticSeverity, NumberOrString, Range, TextEdit};

use crate::document::TextDocumentState;
use crate::frontmatter::FrontmatterBlock;

pub const SOURCE: &str = "ox-content-spacing";
pub const DATA_KIND: &str = "ox-content-spacing-fix";
const CODE_FORBID: &str = "space-between-half-and-full-width";
const CODE_REQUIRE: &str = "require-space-between-half-and-full-width";

#[derive(Clone, Copy, Debug, Default, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub enum SpaceBetweenHalfAndFullWidth {
    Off,
    #[default]
    Forbid,
    Require,
}

impl SpaceBetweenHalfAndFullWidth {
    #[must_use]
    pub fn parse(value: &str) -> Option<Self> {
        match value {
            "off" | "false" | "0" => Some(Self::Off),
            "forbid" | "remove" | "never" => Some(Self::Forbid),
            "require" | "insert" | "always" => Some(Self::Require),
            _ => None,
        }
    }
}

#[derive(Clone, Copy, Debug, Default)]
pub struct SpacingConfig {
    pub between_half_and_full_width: SpaceBetweenHalfAndFullWidth,
    pub auto_fix_on_save: bool,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SpacingFixData {
    pub kind: String,
    pub new_text: String,
}

#[must_use]
pub fn diagnostics(
    document: &TextDocumentState,
    block: Option<&FrontmatterBlock>,
    config: SpacingConfig,
) -> Vec<Diagnostic> {
    edits(document, block, config)
        .into_iter()
        .map(|edit| diagnostic_for_edit(edit, config.between_half_and_full_width))
        .collect()
}

#[must_use]
pub fn formatting_edits(
    document: &TextDocumentState,
    block: Option<&FrontmatterBlock>,
    config: SpacingConfig,
) -> Vec<TextEdit> {
    edits(document, block, config).into_iter().map(|edit| edit.text_edit).collect()
}

#[must_use]
pub fn fix_edit_from_diagnostic(diagnostic: &Diagnostic) -> Option<TextEdit> {
    if diagnostic.source.as_deref() != Some(SOURCE) {
        return None;
    }
    let data = serde_json::from_value::<SpacingFixData>(diagnostic.data.clone()?).ok()?;
    (data.kind == DATA_KIND)
        .then_some(TextEdit { range: diagnostic.range, new_text: data.new_text })
}

fn diagnostic_for_edit(edit: SpacingEdit, mode: SpaceBetweenHalfAndFullWidth) -> Diagnostic {
    let (code, message) = match mode {
        SpaceBetweenHalfAndFullWidth::Forbid => {
            (CODE_FORBID, "Remove the space between half-width and full-width text.")
        }
        SpaceBetweenHalfAndFullWidth::Require => {
            (CODE_REQUIRE, "Insert a space between half-width and full-width text.")
        }
        SpaceBetweenHalfAndFullWidth::Off => ("", ""),
    };

    Diagnostic {
        range: edit.text_edit.range,
        severity: Some(DiagnosticSeverity::ERROR),
        code: Some(NumberOrString::String(code.to_string())),
        source: Some(SOURCE.to_string()),
        message: message.to_string(),
        data: serde_json::to_value(SpacingFixData {
            kind: DATA_KIND.to_string(),
            new_text: edit.text_edit.new_text,
        })
        .ok(),
        ..Default::default()
    }
}

#[derive(Clone, Debug)]
struct SpacingEdit {
    text_edit: TextEdit,
}

fn edits(
    document: &TextDocumentState,
    block: Option<&FrontmatterBlock>,
    config: SpacingConfig,
) -> Vec<SpacingEdit> {
    match config.between_half_and_full_width {
        SpaceBetweenHalfAndFullWidth::Off => Vec::new(),
        SpaceBetweenHalfAndFullWidth::Forbid => forbid_edits(document, block),
        SpaceBetweenHalfAndFullWidth::Require => require_edits(document, block),
    }
}

fn forbid_edits(
    document: &TextDocumentState,
    block: Option<&FrontmatterBlock>,
) -> Vec<SpacingEdit> {
    let mut edits = Vec::new();
    scan_body_lines(document, block, |line, line_start, ignored| {
        let bytes = line.as_bytes();
        let mut index = 0usize;
        while index < bytes.len() {
            if bytes[index] != b' ' || ignored.contains(index) {
                index += 1;
                continue;
            }

            let space_start = index;
            while index < bytes.len() && bytes[index] == b' ' && !ignored.contains(index) {
                index += 1;
            }
            let space_end = index;
            let Some(left) = previous_char(line, space_start, ignored) else {
                continue;
            };
            let Some(right) = next_char(line, space_end, ignored) else {
                continue;
            };
            if should_separate(left, right) {
                edits.push(SpacingEdit {
                    text_edit: TextEdit {
                        range: document
                            .range_from_offsets(line_start + space_start, line_start + space_end),
                        new_text: String::new(),
                    },
                });
            }
        }
    });
    edits
}

fn require_edits(
    document: &TextDocumentState,
    block: Option<&FrontmatterBlock>,
) -> Vec<SpacingEdit> {
    let mut edits = Vec::new();
    scan_body_lines(document, block, |line, line_start, ignored| {
        let chars = line.char_indices().collect::<Vec<_>>();
        for window in chars.windows(2) {
            let (left_offset, left) = window[0];
            let (right_offset, right) = window[1];
            if ignored.contains(left_offset) || ignored.contains(right_offset) {
                continue;
            }
            if left.is_whitespace() || right.is_whitespace() {
                continue;
            }
            if should_separate(left, right) {
                let insert_offset = line_start + right_offset;
                let position = document.offset_to_position(insert_offset);
                edits.push(SpacingEdit {
                    text_edit: TextEdit {
                        range: Range { start: position, end: position },
                        new_text: " ".to_string(),
                    },
                });
            }
        }
    });
    edits
}

fn scan_body_lines(
    document: &TextDocumentState,
    block: Option<&FrontmatterBlock>,
    mut visit: impl FnMut(&str, usize, &IgnoredLineOffsets),
) {
    let body_start = block.map_or(0, |block| block.block_end_offset);
    let mut in_fence: Option<&str> = None;

    for line_index in 0..document.line_count() {
        let line_start = document.line_start_offset(line_index);
        let line_end = document.line_end_offset(line_index);
        if line_end <= body_start {
            continue;
        }

        let line = &document.text()[line_start..line_end];
        let content = line.trim_end_matches(['\r', '\n']);
        let trimmed = content.trim_start();
        if let Some(marker) = in_fence {
            if trimmed.starts_with(marker) {
                in_fence = None;
            }
            continue;
        }
        if trimmed.starts_with("```") {
            in_fence = Some("```");
            continue;
        }
        if trimmed.starts_with("~~~") {
            in_fence = Some("~~~");
            continue;
        }

        let ignored = inline_code_offsets(content);
        visit(content, line_start, &ignored);
    }
}

#[derive(Default)]
struct IgnoredLineOffsets {
    ranges: Vec<std::ops::Range<usize>>,
}

impl IgnoredLineOffsets {
    fn contains(&self, offset: usize) -> bool {
        self.ranges.iter().any(|range| range.contains(&offset))
    }
}

fn inline_code_offsets(line: &str) -> IgnoredLineOffsets {
    let mut ranges = Vec::new();
    let mut open: Option<usize> = None;
    for (offset, ch) in line.char_indices() {
        if ch != '`' {
            continue;
        }
        if let Some(start) = open.take() {
            ranges.push(start..offset + ch.len_utf8());
        } else {
            open = Some(offset);
        }
    }
    IgnoredLineOffsets { ranges }
}

fn previous_char(line: &str, before: usize, ignored: &IgnoredLineOffsets) -> Option<char> {
    line[..before]
        .char_indices()
        .rev()
        .find(|(offset, ch)| !ignored.contains(*offset) && !ch.is_whitespace())
        .map(|(_, ch)| ch)
}

fn next_char(line: &str, after: usize, ignored: &IgnoredLineOffsets) -> Option<char> {
    line[after..]
        .char_indices()
        .find(|(offset, ch)| !ignored.contains(after + offset) && !ch.is_whitespace())
        .map(|(_, ch)| ch)
}

fn should_separate(left: char, right: char) -> bool {
    (is_half_width_text(left) && is_full_width_text(right))
        || (is_full_width_text(left) && is_half_width_text(right))
}

fn is_half_width_text(ch: char) -> bool {
    ch.is_ascii_alphanumeric()
}

fn is_full_width_text(ch: char) -> bool {
    matches!(
        ch as u32,
        0x3040..=0x30ff // Hiragana + Katakana
            | 0x3400..=0x9fff // CJK ideographs
            | 0xac00..=0xd7af // Hangul syllables
            | 0xff01..=0xff60 // Fullwidth ASCII variants and punctuation
            | 0xffe0..=0xffee
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::frontmatter::parse_frontmatter;

    fn doc(source: &str) -> TextDocumentState {
        TextDocumentState::new(source.to_string())
    }

    #[test]
    fn forbid_reports_spaces_between_ascii_and_japanese_text() {
        let document = doc("Rust と TypeScript\n");
        let diagnostics = diagnostics(&document, None, SpacingConfig::default());
        assert_eq!(diagnostics.len(), 2);
        assert_eq!(diagnostics[0].source.as_deref(), Some(SOURCE));
        assert_eq!(diagnostics[0].range.start.character, 4);
        assert_eq!(fix_edit_from_diagnostic(&diagnostics[0]).unwrap().new_text, "");
    }

    #[test]
    fn require_reports_missing_spaces_between_ascii_and_japanese_text() {
        let document = doc("RustとTypeScript\n");
        let diagnostics = diagnostics(
            &document,
            None,
            SpacingConfig {
                between_half_and_full_width: SpaceBetweenHalfAndFullWidth::Require,
                auto_fix_on_save: false,
            },
        );
        assert_eq!(diagnostics.len(), 2);
        assert_eq!(diagnostics[0].range.start, diagnostics[0].range.end);
        assert_eq!(fix_edit_from_diagnostic(&diagnostics[0]).unwrap().new_text, " ");
    }

    #[test]
    fn ignores_frontmatter_fences_and_inline_code() {
        let source = concat!(
            "---\n",
            "title: Rust と TypeScript\n",
            "---\n",
            "\n",
            "`Rust と TypeScript`\n",
            "```txt\n",
            "Rust と TypeScript\n",
            "```\n",
            "Rust と TypeScript\n",
        );
        let document = doc(source);
        let frontmatter = parse_frontmatter(&document);
        let diagnostics =
            diagnostics(&document, frontmatter.block.as_ref(), SpacingConfig::default());
        assert_eq!(diagnostics.len(), 2);
        assert_eq!(diagnostics[0].range.start.line, 8);
    }
}
